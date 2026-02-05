# rust-pi

Pure Rust implementation of core `pi-mono` concepts.

This workspace mirrors the high-level package boundaries from `badlogic/pi-mono` and provides a functional baseline:

- `crates/pi-ai`: provider-agnostic message and tool model + OpenAI/Anthropic/Google adapters
- `crates/pi-agent-core`: event-driven agent loop with tool execution
- `crates/pi-tui`: minimal differential terminal rendering primitives
- `crates/pi-coding-agent`: CLI harness with built-in `read`, `write`, `edit`, and `bash` tools

## Current Scope

Implemented now:

- Rust-first core architecture (no Node/TypeScript runtime)
- Tool-call loop (`assistant -> tool -> assistant`) in `pi-agent-core`
- Multi-provider model routing: `openai/*`, `anthropic/*`, `google/*`
- Interactive CLI and one-shot prompt mode
- Persistent JSONL sessions with branch/resume support
- Built-in filesystem and shell tools
- Unit tests for serialization, tool loop, renderer diffing, and tool behaviors

Not implemented yet:

- Streaming token-by-token UI updates
- Advanced session tree UX/compaction
- Extensions/skills/themes package system
- Full TUI parity with overlays/images/editor

## Build & Test

```bash
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Usage

Set an API key for your provider:

```bash
# OpenAI-compatible
export OPENAI_API_KEY=...your-key...

# Anthropic
export ANTHROPIC_API_KEY=...your-key...

# Google Gemini
export GEMINI_API_KEY=...your-key...
```

Run interactive mode:

```bash
cargo run -p pi-coding-agent -- --model openai/gpt-4o-mini
```

Use Anthropic:

```bash
cargo run -p pi-coding-agent -- --model anthropic/claude-sonnet-4-20250514
```

Use Google Gemini:

```bash
cargo run -p pi-coding-agent -- --model google/gemini-2.5-pro
```

Run one prompt:

```bash
cargo run -p pi-coding-agent -- --prompt "Summarize src/lib.rs"
```

Use a custom base URL (OpenAI-compatible):

```bash
cargo run -p pi-coding-agent -- --api-base http://localhost:11434/v1 --model openai/qwen2.5-coder
```

Session branching and resume:

```bash
# Persist to the default session file (.pi/sessions/default.jsonl)
cargo run -p pi-coding-agent -- --model openai/gpt-4o-mini

# Resume latest branch (default behavior), inspect session state
/session
/branches

# Switch to an older entry and fork a new branch
/branch 12

# Jump back to latest head
/resume
```
