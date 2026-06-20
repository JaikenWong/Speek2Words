# Speek2Words

按住热键说话，松手文字喷到光标所在输入框。

Hold hotkey to talk, release to type text at cursor.

## 功能

- 🎤 按住热键录音，松开自动语音识别
- ⌨️ 识别结果自动输入到光标位置
- 🔧 可配置 API Key、热键、语言等
- 🖥️ macOS + Windows 双平台支持
- 🔔 系统托盘常驻后台

## 安装

从 [Releases](https://github.com/JaikenWong/Speek2Words/releases) 下载对应平台安装包。

## 开发

```bash
# 安装前端依赖
npm install

# 开发模式
npx tauri dev

# 构建发布
npx tauri build
```

## 配置

启动后在 Settings 页面配置：

| 配置项 | 默认值 | 说明 |
|--------|--------|------|
| API Key | - | MiniMax 或 Whisper 兼容 API Key |
| Base URL | `https://api.minimaxi.com` | ASR 服务地址 |
| Model | `speech-01` | 语音模型 |
| Hotkey | `CommandOrControl+Shift+K` | 录音热键 |
| Language | `zh` | 语言（zh/en/auto） |

## 权限

### macOS
- **麦克风**：系统设置 → 隐私与安全 → 麦克风
- **辅助功能**：系统设置 → 隐私与安全 → 辅助功能
- **输入监控**：部分系统需要

### Windows
- **麦克风**：系统会自动请求权限

## 技术栈

- [Tauri v2](https://tauri.app/) - 桌面应用框架
- [Rust](https://rust-lang.org/) - 后端（音频录制、ASR、键盘模拟）
- [TypeScript](https://typescriptlang.org/) + [Vite](https://vitejs.dev/) - 前端

## License

MIT
