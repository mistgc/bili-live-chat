# bili-live-chat

A bilibili streaming chat tool using TUI written in Rust.

## Installaion

### Linux

```bash
git clone https://github.com/zaiic/bili-live-chat.git
cd bili-live-chat
cargo build --release
sudo cp target/release/bili-live-chat /usr/local/bin/bili-live-chat
bili-live-chat --version
```

### Windows

```bash
git clone https://github.com/zaiic/bili-live-chat.git
cd bili-live-chat
cargo build --release
# put the path of the directory to the PATH of USER in the environment variable for global use
bili-live-chat.exe --version
```

## Configuration

The path of config file is `$HOME/.config/bili-live-chat/config.yml`, and the format of config file is **YAML**.
All fields of `credential` come from Cookies from [BiLiBiLi](https://www.bilibili.com).

Notice: In **Windows**, the config file is located at `C:\Users\your_username\.config\bili-live-chat\config.yml`.

```yaml
credential:
  session_data: "<Your session_data>"
  bili_jct: "<Your bili_jct>"
  buvid3: "<Your buvid3>"
```

## Usage

**Press `Q` to quit programe**

**Press `E` into Editing Mode**

**Press `Esc` to exit Editing Mode**

**Press `Tab` to switch the tab**

```
Usage: bili-live-chat [OPTIONS] -d <ROOM_ID>

Options:
  -c, --config <FILE>        Sets loading path for config file
  -d <ROOM_ID>               Specify a live room
  -s, --sessdata <SESSDATA>  
  -b, --bili_jct <BILI_JCT>  
  -u, --buvid3 <BUVID3>      
  -h, --help                 Print help
  -V, --version              Print version
```
```bash
# use with config file
bili-live-chat -d <ROOM_ID>

# use without config file or pass fields of the credential directly (if the config file does not exist)
bili-live-chat -d <ROOM_ID> -s "sessdata" -b "bili_jct" -u "buvid3"
```

Notice: If the config file exists, and some of the fields of the credential are passed by CLI, the latter will overwrite the fields of the credential from the config file.

## Requirements

On Linux:
- OpenSSL 1.0.1, 1.0.2, 1.1.0, or 1.1.1 with headers (see https://github.com/sfackler/rust-openssl)

## Similar Projects

[yaocccc/bilibili_live_tui](https://github.com/yaocccc/bilibili_live_tui): 终端下使用的bilibili弹幕获取和弹幕发送服务 
