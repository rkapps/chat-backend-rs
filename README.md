# chat-backend-rs

Backend service for multi-provider LLM chat interactions with persistent storage.

## Overview

REST API that provides unified chat interface across multiple LLM providers (Gemini, Claude, OpenAI) with file-based conversation storage.

**Built with:**

- [agentic-core-rs](../agentic-core-rs) - Multi-provider LLM abstraction
- [storage-core-rs](../storage-core-rs) - File-based storage with repository pattern

## Features

- 🤖 **Multi-Provider Support** - Gemini, Claude (Anthropic), OpenAI
- 💾 **Persistent Storage** - File-based chat history
- 🔄 **Stateful & Stateless** - Supports both API patterns
- ⚡ **Async & Concurrent** - Built on tokio with Arc<Mutex>
- 🎯 **System Prompts** - Customize AI behavior per chat

## Architecture

agentic-rs (Backend)
├── agentic-core-rs (LLM abstraction)
│   ├── GeminiClient
│   ├── AnthropicClient
│   └── OpenAIClient
├── storage-core-rs (Storage)
│   └── FsRepository (File storage)
└── Chat Handlers
    └── Arc<Mutex<ChatStorage>>

**Data Flow:**

1. HTTP request → Chat handler
2. Handler retrieves chat history from storage
3. Handler selects LLM client based on provider
4. Client sends to LLM API
5. Response stored to file
6. Response returned to client

## Storage Structure

./dbname/
└── chats/
├   ├── chat_123.json
├   ├── chat_456.json
├   └── chat_789.json
└───dgname.json

## Environment Variables

| Variable            | Required   | Description       |
|---------------------|------------|-------------------|
| `GEMINI_API_KEY`    | For Gemini | Gemini API key    |
| `ANTHROPIC_API_KEY` | For Claude | Anthropic API key |
| `OPENAI_API_KEY`    | For OpenAI | OpenAI API key    |

## Quick Start

### Prerequisites

- Rust 1.70+
- API keys for desired providers

### Installation

```bash
git clone https://github.com/yourusername/chat-backend-rs.git
cd agentic-rs
cargo build --release
```

## License

MIT

## Contributing

1. Fork the repository
2. Create feature branch
3. Commit changes
4. Submit pull request

---

Part of the **agentic** project suite