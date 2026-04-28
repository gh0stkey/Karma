<div align="center">
<img src="src-tauri/icons/logo.png" style="width: 20%" />
<h4><a href="https://github.com/gh0stkey/Karma">On-device PII Detection & Redaction, Powered by MLX.</a></h4>
<h5>Author: <a href="https://github.com/gh0stkey">EvilChen</a></h5>
</div>

README Version: \[[English](README.md) | [简体中文](README_CN.md)\]

## Project Introduction

**Karma** is a privacy-first desktop application for detecting and redacting Personally Identifiable Information (PII). Powered by Apple's MLX framework, all AI inference runs entirely on-device — your data never leaves your machine.

Karma identifies **9 types of PII** in text and replaces them with labeled placeholders:

| PII Type | Placeholder |
|----------|-------------|
| Person Name | `[PERSON]` |
| Email Address | `[EMAIL]` |
| Phone Number | `[PHONE]` |
| Physical Address | `[ADDRESS]` |
| Date | `[DATE]` |
| URL | `[URL]` |
| Account Number | `[ACCOUNT_NUMBER]` |
| Secret / Password | `[SECRET]` |

## Features

- **Real-time PII Detection**: Token-level classification powered by MLX on Apple Silicon
- **Text Redaction**: One-click redaction with auto-copy to clipboard
- **HTTP API Server**: Built-in REST API server (`/health`, `/redact`) for integration with other tools
- **Redaction History**: SQLite-backed history with infinite scroll browsing
- **Global Shortcut**: Quick access via configurable keyboard shortcut (default: `Cmd+Shift+K`)
- **Bilingual UI**: English and Simplified Chinese
- **Zero Cloud Dependency**: All processing happens locally for complete privacy

## Tech Stack

| Component | Technology |
|-----------|------------|
| Desktop Framework | Tauri 2 (Rust) |
| Frontend | React + TypeScript + Tailwind CSS |
| State Management | Zustand |
| AI Inference | MLX + MLX Embeddings (Python Sidecar) |
| HTTP Server | Axum |
| Database | SQLite (rusqlite) |
| Build Tool | Vite |

## Installation

Download the latest `.dmg` from the [Releases](https://github.com/gh0stkey/Karma/releases) page.

> **Requirements**: macOS 11.0+ with Apple Silicon (M1/M2/M3/M4)

## Usage

### Quick Start

1. Open Karma and navigate to the **Model** page
2. Select your local MLX token-classification model directory
3. Wait for the model to load (status indicator turns green)
4. Switch to the **Redactor** page, paste your text, and click **Redact**

### Global Shortcut

Press `Cmd+Shift+K` (configurable in Settings) to instantly open Karma and jump to the Redactor page.

### HTTP API

Enable the built-in HTTP server in the **Server** page to expose the redaction API:

```bash
# Health check
curl http://127.0.0.1:8000/health

# Redact text
curl -X POST http://127.0.0.1:8000/redact \
  -H "Content-Type: application/json" \
  -d '{"text": "My name is John and my email is john@example.com"}'
```

## Build from Source

### Prerequisites

- Node.js >= 20
- Rust (stable)
- [uv](https://github.com/astral-sh/uv) (Python package manager)

### Steps

```bash
# Install frontend dependencies
npm install

# Build sidecar binary (PyInstaller)
make sidecar

# Build Tauri app (DMG)
make app
```

Or run the full pipeline:

```bash
make all
```

## Interface

| Page | Description |
|------|-------------|
| Redactor | Input text, detect PII, view redacted output and detected spans |
| Model | Configure model path, view loaded model metadata |
| Server | Enable/configure HTTP API server, view API reference and request logs |
| History | Browse past redaction records with search and pagination |
| Settings | Global shortcut, language, auto-copy, history limit, and more |
| About | Version info and tech stack credits |
