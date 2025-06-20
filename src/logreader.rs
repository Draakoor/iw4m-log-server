// Dependenciess
use lazy_static::lazy_static;
use std::{collections::HashMap, time::SystemTime, fs::{self, File}, io::{SeekFrom, Seek, Read}};
use regex::Regex;
use rand::Rng;

// Structs
pub struct LogFile {
    pub content: Option<String>,
    pub next_key: Option<String>
}
#[derive(Clone, Debug)]
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
        self.log_file_sizes.retain(|_, v| (current_time - v.read.unwrap()) < self.max_file_time_change)
    }

    // Gets the length of a file
    pub fn file_length(path: &str) -> Option<u64> {
        // Get the file's metadata and check for errors
        let file_md = fs::metadata(path);
        if let Err(_e) = file_md {
            return None;
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
    pub fn get_file_lines(path: String, start_position: u64, length_to_read: usize, verbose: bool) -> Option<String> {
        // Attempt to open the file
        let file_handle_r = File::open(path);
        if let Err(_e) = file_handle_r {
            return None;
        }
        let mut file_handle = file_handle_r.unwrap();

        // Read data within the range
        if verbose { println!("iw4m-log-server: reading from {} for {} characters", start_position, length_to_read); }
        file_handle.seek(SeekFrom::Start(start_position)).unwrap();
        let mut file_data = vec![0; length_to_read];
        file_handle.read_exact(&mut file_data).unwrap();

        // Convert buffer to a string
        return Some(String::from_utf8_lossy(&file_data).to_string());
    }

    // Read a log file
    pub fn read_file(&mut self, path: String, retrieval_key: String, verbose: bool) -> LogFile {
        // Static RE
        lazy_static! {
            static ref AUNIX_PATH_SUB_RE: Regex = Regex::new(r"^[A-Z]\:").unwrap();
            static ref BUNIX_PATH_SUB_RE: Regex = Regex::new(r"\\+").unwrap();
            static ref TRAVERSE_DIR_RE: Regex = Regex::new(r"^.+\.\.\\.+$").unwrap();
            static ref LOG_PATH_RE: Regex = Regex::new(r"^.+[\\|/](.+)[\\|/].+.log$").unwrap();
        }
    
        // Attempt to remove old logs
        self.clear_old_logs();

        // Add support for other os (linux)
        if cfg!(unix) {
            AUNIX_PATH_SUB_RE.replace(&path, "");
            BUNIX_PATH_SUB_RE.replace(&path, "/");
        }

        // Prevent traversing directories and check if valid log file
        if TRAVERSE_DIR_RE.is_match(&path) || !LOG_PATH_RE.is_match(&path) {
            if verbose { println!("iw4m-log-server: invalid path - traversing directory"); }
            return EMPTY_LOG;
        }

        // Check the file size
        let file_size_r = LogReader::file_length(&path);
        if file_size_r.is_none() {
            if verbose { println!("iw4m-log-server: log file cannot be read?"); }
            return EMPTY_LOG;
        }

        // Default the last_log
        let mut last_log_info = LogFileSize {
            size: file_size_r,
            previous_key: None,

            read: None,
            next_key: None
        };

        // Set if already have a valid log file
        let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
        match self.log_file_sizes.get(&retrieval_key) {
            Some(log_file_size) => {
                if (current_time - log_file_size.read.unwrap()) < self.max_file_time_change {
                    last_log_info = log_file_size.clone();
                    if verbose { println!("iw4m-log-server: loaded old key"); } 
                }
            },
            None => {
                if verbose { println!("iw4m-log-server: new key"); } 
            },
        }

        // Save for later...
        let expired_key_r = last_log_info.previous_key.clone();

        // Grab previous value
        let last_size = last_log_info.size.unwrap();
        let file_size_difference = file_size_r.unwrap() - last_size;

        // Update the new size
        let next_retrieval_key = LogReader::generate_key();
        let new_log_file_sizes = LogFileSize {
            size: file_size_r,
            read: Some(current_time),
            next_key: Some(next_retrieval_key.clone()),
            previous_key: Some(retrieval_key)
        };
        self.log_file_sizes.insert(next_retrieval_key.clone(), new_log_file_sizes.clone());
        if verbose { println!("iw4m-log-server: added log file size -> {:?}", new_log_file_sizes.clone()); }

        // Remove expired key, if there is one
        match expired_key_r {
            Some(expired_key) => { self.log_file_sizes.remove(&expired_key); },
            None => {},
        }

        // Grab the new content
        let new_log_content = if file_size_difference > 0 {
            LogReader::get_file_lines(path, last_size, file_size_difference.try_into().unwrap(), verbose)
        } else {
            Some(String::from(""))
        };
        if verbose { println!("iw4m-log-server: got log file: {:?}", new_log_content); }

        // Done!
        LogFile {
            content: new_log_content,
            next_key: Some(next_retrieval_key)
        }
    }
}
