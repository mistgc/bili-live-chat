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

## Configuration

The path of config file is `$HOME/.config/bili-live-chat/config.yml`, and the format of config file is **YAML**.
All field of `credential` come from Cookies from [BiLiBiLi](https://www.bilibili.com).

```yaml
credential:
  session_data: "<Your session_data>"
  bili_jct: "<Your bili_jct>"
  buvid3: "<Your buvid3>"
```

## Usage
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

# use without config file or pass fields of the credential directly
bili-live-chat -d <ROOM_ID> -s "sessdata" -b "bili_jct" -u "buvid3"
```

## Requirements

On Linux:
- OpenSSL 1.0.1, 1.0.2, 1.1.0, or 1.1.1 with headers (see https://github.com/sfackler/rust-openssl)
