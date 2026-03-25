# Tauri AI Assistant

A customizable context-aware AI assistant built with Tauri (Rust + WebView).

## Features

- **AI Chat** - Powered by OpenRouter with multiple model support
- **Memory System** - Remembers user preferences and facts
- **Weather** - Get current weather for any city
- **Time** - Display current time
- **Dictionary** - Look up word definitions
- **Calculator** - Built-in calculator
- **Customizable Personality** - Fully customizable bot name, greeting, responses, and more

## Getting Started

### Prerequisites

- [OpenRouter API Key](https://openrouter.ai) - Free tier available

### Installation

1. Download the latest release from the releases page
2. Run the application
3. On first launch, enter your OpenRouter API key
4. Customize your bot's personality in Settings

## Customization

Click the **Settings** tab to customize:

- **Bot Name** - Your assistant's name
- **Greeting Message** - Welcome message for users
- **System Prompt** - How your AI behaves
- **Speaking Style** - Toggle emojis, casual speech, etc.
- **Catchphrases** - Fun phrases your bot might use
- **Idle Messages** - Messages when user is away
- **Emotional Responses** - Custom responses for different emotions

## Commands

| Command | Description |
|---------|-------------|
| `/weather <city>` | Get weather for a city |
| `/time` | Show current time |
| `/define <word>` | Look up a word definition |
| `/calculator` | Open calculator |
| `/remember <fact>` | Store a fact in memory |
| `/forget <fact>` | Remove a fact from memory |
| `/memories` | List all stored memories |

## Tech Stack

- **Backend**: Rust + Tauri
- **Frontend**: Vanilla HTML/CSS/JavaScript
- **AI**: OpenRouter API (free models available)

## Building from Source

```bash
cd multibot
cargo build --release
```

The executable will be in `target/release/multibot.exe`

## License

MIT
