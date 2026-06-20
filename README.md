# Speek2Words

按住热键说话，松手文字喷到光标所在输入框。

Hold hotkey to talk, release to type text at cursor.

## 功能

- 🎤 按住热键录音，松开自动语音识别
- ⌨️ 识别结果自动输入到光标位置
- 🍎 macOS 离线识别（SFSpeechRecognizer，无需 API Key）
- 🪟 Windows 在线识别（Groq/OpenAI Whisper API）
- 🔧 可配置热键、语言等
- 🔔 系统托盘常驻后台

## 安装

从 [Releases](https://github.com/JaikenWong/Speek2Words/releases) 下载对应平台安装包。

## 开发

```bash
# 安装前端依赖
npm install

# macOS: 编译 Swift STT helper
cd src-tauri && swiftc -O -o bin/s2w_stt bin/s2w_stt.swift -framework Speech -framework Foundation && cd ..

# 开发模式
npx tauri dev

# 构建发布
npx tauri build
```

## 配置

启动后在 Settings 页面配置：

### macOS
无需 API Key，使用系统内置 SFSpeechRecognizer（离线）。

首次使用需授权：
- **麦克风**：系统设置 → 隐私与安全 → 麦克风
- **语音识别**：系统设置 → 隐私与安全 → 语音识别
- **辅助功能**：系统设置 → 隐私与安全 → 辅助功能（输入文字必需）

### Windows
| 配置项 | 默认值 | 说明 |
|--------|--------|------|
| API Key | - | Groq / OpenAI 或其他 Whisper 兼容 API Key |
| Base URL | `https://api.groq.com/openai` | ASR 服务地址 |
| Model | `whisper-large-v3-turbo` | 语音模型 |

### 通用配置

| 配置项 | 默认值 | 说明 |
|--------|--------|------|
| Hotkey | `CmdOrCtrl+Shift+KeyK` | 录音热键 |
| Language | `zh` | 语言（zh/en/auto） |

### Windows 推荐的 ASR 服务

| 服务 | Base URL | Model | 说明 |
|------|----------|-------|------|
| [Groq](https://console.groq.com) | `https://api.groq.com/openai` | `whisper-large-v3-turbo` | 免费，速度快 |
| [OpenAI](https://platform.openai.com) | `https://api.openai.com` | `whisper-1` | 官方 Whisper |
| [SiliconFlow](https://siliconflow.cn) | `https://api.siliconflow.cn/v1` | `FunAudioLLM/SenseVoiceSmall` | 国内，中文优 |

## 技术栈

- [Tauri v2](https://tauri.app/) - 桌面应用框架
- [Rust](https://rust-lang.org/) - 后端（音频录制、键盘模拟）
- [SFSpeechRecognizer](https://developer.apple.com/documentation/speech) - macOS 离线语音识别
- [TypeScript](https://typescriptlang.org/) + [Vite](https://vitejs.dev/) - 前端

## License

MIT
