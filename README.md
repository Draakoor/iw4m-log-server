# IW4M Log Server

This allows IW4M to send the game logs over a REST-ful API. The original Python code and insperation can be found [here](https://github.com/RaidMax/IW4MAdmin-GameLogServer/tree/master). The reason I did this was to:
- Practice my Rust skills
- To make a more efficient version

# Usage

- Download the executable for your OS from the [latest release](https://github.com/Stefanuk12/iw4m-log-server/releases/latest)
- Simply run the executable

```
Allows IW4M Admin to retrieve server game logs

Usage: iw4m-log-server.exe [OPTIONS]

Options:
  -H, --host <HOST>  The host of the IW4M server [default: 0.0.0.0]
  -p, --port <PORT>  Specify a custom port to bind to [default: 1625]
  -h, --help         Print help
  -V, --version      Print version
```