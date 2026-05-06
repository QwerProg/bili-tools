# bt

B站直播开播工具，命令行一键开播/下播。

## 安装

### 预编译

从 [Release](https://github.com/QwerProg/bili-tools/releases) 下载对应平台二进制。

### 自行编译

```bash
git clone https://github.com/QwerProg/bili-tools.git --depth=1
cd bili-tools
cargo install --path .
```

编译安装到 `~/.cargo/bin/bt`，确认该目录在 PATH 中即可直接使用。

## 使用

```bash
bt          # 开播（默认，交互式）
bt -s       # 同上
bt -c       # 下播
bt --status # 查看直播状态
bt --restart# 清除登录，重新扫码
```

### 开播参数

| 参数 | 说明 |
|---|---|
| `--area <ID>` | 指定分区 ID，跳过交互式选择 |
| `--title <标题>` | 指定直播标题，跳过输入 |
| `--show` | 显示完整推流码（默认打码） |

```bash
# 非交互式一键开播
bt --area 398 --title "晚上随便播会儿" --show
```

### 完整参数

```
Usage: bt [OPTIONS]

Options:
  -s, --start          开播（默认行为）
  -c, --close          下播
      --status         查看直播状态
      --restart        重新登录（清除 Cookie）
      --area <AREA>    指定直播分区 ID，跳过交互式选择
      --title <TITLE>  指定直播标题，跳过输入
      --show           显示完整推流码（不打码）
  -h, --help           Print help
  -V, --version        Print version
```

## 交互流程

```
# 首次使用 — 扫码登录
✅ 需要登录，开始登录流程...
[终端弹出二维码]

# 登录后 — 正常开播
✅ 登录状态正常
📺 是否使用上次直播的分区？(回车/y=使用默认，n=选择新分区)
✅ 使用上次的分区: 自习室 - 372
📺 请输入新标题（直接回车保留原标题）:
ℹ️ 开始直播！
✅ RTMP地址: rtmp://live-push.bilivideo.com/live-bvc/
✅ 推流码: ?strea************...g=13
✅ 推流信息已写入 stream_info.txt
✅ 直播已开启，程序退出

# 已在播时运行 — 询问下播
✅ 登录状态正常
ℹ️ 检测到当前正在直播中
📺 是否关闭直播？(y/n): y
ℹ️ 准备关闭直播...
✅ 成功关闭直播
ℹ️ 直播统计信息:
ℹ️ 新增粉丝 : 3
ℹ️ 弹幕数 : 42
ℹ️ 直播时长 : 7200
...

# 查看状态 — 显示 B 站电视机 Logo
bt --status
```

## 分区选择快捷键

| 键 | 功能 |
|---|---|
| `j` `↓` | 下移 |
| `k` `↑` | 上移 |
| `l` `→` | 右移 |
| `h` `←` | 左移 / 二级菜单返回上级 |
| `Enter` | 确认 / 进入子分区 |
| `Backspace` | 二级菜单返回上级 |
| `q` `Esc` | 退出 |

## 注意事项

- `cookies.json` 包含敏感凭证（SESSDATA、bili_jct），请勿泄露
- 推流信息写入 `stream_info.txt`，第一行 RTMP 地址，第二行完整推流码
- 高频调用接口可能触发风控，请合理使用
- 本项目仅供学习交流，禁止用于违反 B站用户协议的行为
