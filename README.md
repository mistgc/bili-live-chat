# bili-live-chat

一个由Rust写的bilibili直播聊天TUI工具。

## 安装

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
# 将当前目录加入至用户的PATH目录后，以便全局使用
bili-live-chat.exe --version
```

## 配置

配置文件在 `$HOME/.config/bili-live-chat/config.yml` ，格式为 **YAML** 。
`credential` 中所有字段全来源于 [BiLiBiLi](https://www.bilibili.com) 的Cookies。

注意：**Windows**下，配置文件默认在`C:\Users\用户名\.config\bili-live-chat\config.yml`。

```yaml
credential:
  session_data: "<Your session_data>"
  bili_jct: "<Your bili_jct>"
  buvid3: "<Your buvid3>"
```

## 使用

**按 `Q` 退出**

**按 `E` 进入 Editing Mode**

**按 `Esc` 退出 Editing Mode**

**按 `Tab` 切换标签**

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
# 使用配置文件
bili-live-chat -d <ROOM_ID>

# 不使用配置文件或者直接传递所有凭证(credential)的字段 (如果没有配置文件的话)
bili-live-chat -d <ROOM_ID> -s "sessdata" -b "bili_jct" -u "buvid3"
```

注意：如果有配置文件，且使用命令行传递了凭证(credential)的字段，后者将覆盖配置文件中的凭证(credential)的字段。

## 要求

On Linux:
- OpenSSL 1.0.1, 1.0.2, 1.1.0, or 1.1.1 with headers (see https://github.com/sfackler/rust-openssl)

## 类似项目

[yaocccc/bilibili_live_tui](https://github.com/yaocccc/bilibili_live_tui): 终端下使用的bilibili弹幕获取和弹幕发送服务 
