// Dependencies
use lazy_static::lazy_static;
use std::{collections::HashMap, time::SystemTime, fs::{self, File}, io::{SeekFrom, Seek, Read}};
use regex::Regex;
use rand::Rng;

// Structs
pub struct LogFile {
    pub content: Option<String>,
    pub next_key: Option<String>
}
#[derive(Clone)]
pub struct LogFileSize {
    pub size: Option<u64>,
    pub read: Option<u64>,
    pub next_key: Option<String>,
    pub previous_key: Option<String>
}
pub struct LogReader {
    pub log_file_sizes: HashMap<String, LogFileSize>,
    pub max_file_time_change: u64
}

// Constants
const EMPTY_LOG: LogFile = LogFile {
    content: None,
    next_key: None
};

// Funcs
impl LogReader {
    // Initialise the default log reader
    pub fn default() -> LogReader {
        Self {
            log_file_sizes: HashMap::default(),
            max_file_time_change: 30
        }
    }

    // Attempt to clear old logs
    pub fn clear_old_logs(&mut self) {
        // Grab current time and only retain new logs
        let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
        self.log_file_sizes.retain(|_, v| v.read.is_some() && (current_time - v.read.unwrap()) < self.max_file_time_change)
    }

    // Gets the length of a file
    pub fn file_length(path: &str) -> Option<u64> {
        // Get the file's metadata and check for errors
        let file_md = fs::metadata(path);
        if let Err(_e) = file_md {
            println!("could not get the size of the log file at {}", path);
            return Some(0);
        }

        // Return the length of the file
        Some(file_md.unwrap().len())
    }

    // Generates a random string, used as a key
    pub fn generate_key() -> String {
        let mut rng = rand::thread_rng();
        (0..8)
            .map(|_| rng.gen_range(0..36))
            .map(|n| if n < 26 { (n + b'A') as char } else { (n - 26 + b'0') as char })
            .collect()
    }

    // Attempts to read a file's lines (valid input for start_position and length_to_read are assumed)
    pub fn get_file_lines(path: String, start_position: u64, length_to_read: usize) -> Option<String> {
        // Attempt to open the file
        let file_handle_r = File::open(path);
        if let Err(_e) = file_handle_r {
            return None;
        }
        let mut file_handle = file_handle_r.unwrap();

        // Read data within the range
        file_handle.seek(SeekFrom::Start(start_position)).unwrap();
        let mut file_data = vec![0; length_to_read];
        file_handle.read_exact(&mut file_data).unwrap();

        // Convert buffer to a string
        return Some(String::from_utf8_lossy(&file_data).to_string());
    }

    // Read a log file
    pub fn read_file(&mut self, path: String, retrieval_key: String) -> LogFile {
        // Static RE
        lazy_static! {
            static ref AUNIX_PATH_SUB_RE: Regex = Regex::new(r"^[A-Z]\:").unwrap();
            static ref BUNIX_PATH_SUB_RE: Regex = Regex::new(r"\\+").unwrap();
            static ref TRAVERSE_DIR_RE: Regex = Regex::new(r"^.+\.\.\\.+$").unwrap();
            static ref LOG_PATH_RE: Regex = Regex::new(r"^.+\.\.\\.+$").unwrap();
        }
    
        // Attempt to remove old logs
        self.clear_old_logs();

        // Add support for other os (linux)
        if cfg!(unix) {
            AUNIX_PATH_SUB_RE.replace(&path, "");
            BUNIX_PATH_SUB_RE.replace(&path, "/");
        }

        // Make sure is valid
        if !LOG_PATH_RE.is_match(&path) || TRAVERSE_DIR_RE.is_match(&path) {
            return EMPTY_LOG;
        }

        // Check the file size
        let file_size_r = LogReader::file_length(&path);
        if file_size_r.is_none() {
            return EMPTY_LOG;
        }
        let file_size = file_size_r.unwrap();

        // Default the last_log
        let next_retrieval_key = LogReader::generate_key();
        let mut last_log_info = LogFileSize {
            size: Some(file_size),
            previous_key: None,

            read: None,
            next_key: None
        };

        // Set if already have a valid log file
        let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
        if self.log_file_sizes.contains_key(&next_retrieval_key) || (current_time - self.log_file_sizes.get(&next_retrieval_key).unwrap().read.unwrap()) < self.max_file_time_change {
            last_log_info = self.log_file_sizes.get(&next_retrieval_key).unwrap().clone();
        }

        // Save for later...
        let expired_key = last_log_info.previous_key.unwrap();

        // Grab previous value
        let last_size = last_log_info.size.unwrap();
        let file_size_difference = file_size - last_size;

        // Update the new size
        self.log_file_sizes.entry(next_retrieval_key.clone()).and_modify(|x| {
            x.size = Some(file_size);
            x.read = Some(current_time);
            x.next_key = Some(next_retrieval_key.clone());
            x.previous_key = Some(retrieval_key)
        });

        // Remove expired key
        self.log_file_sizes.remove(&expired_key);

        // Grab the new content and return
        let new_log_content = LogReader::get_file_lines(path, last_size, file_size_difference.try_into().unwrap());
        LogFile {
            content: new_log_content,
            next_key: Some(next_retrieval_key)
        }
    }
}