# Hela-TRM

A Rust terminal-based tools client with hacking utilities, built with Ratatui UI.

## Overview

Hela is a terminal-based tools client featuring:
- **Ratatui UI** - Modern terminal interface with hacking aesthetic
- **Custom "H" logo** - Prominent branding
- **ASCII/terminal portrait avatar** - Anime girl character rendered in half-block cells
- **Multiple cryptographic tools** - MD5, SHA1, SHA256, SHA512, base64, Caesar cipher, JWT parsing
- **Network utilities** - Port scanning, DNS lookup, HTTP headers
- **File analysis** - Magic bytes, entropy, hexdump, text reporting
- **Password generation** and wordlist tools

## Features

### Cryptography
- Hash computation (MD5, SHA1, SHA256, SHA512)
- Base64 encoding/decoding
- Caesar cipher brute-force
- JWT parsing and pretty-printing
- Wordlist generation

### Network
- Port scanning with multiple threads
- DNS resolution
- HTTP HEAD requests

### File Analysis
- File type detection (magic bytes)
- Entropy calculation
- SHA256 hashing
- Hex dump generation
- Text content extraction

### Utilities
- Password generation with configurable options
- System information display

## Installation

```bash
# Using cargo
cargo install hela-trm

# Or build from source
git clone https://github.com/yourusername/Hela-TRM.git
cd Hela-TRM
cargo build --release
```

## Usage

```bash
./target/debug/hela
```

Or with specific tool:

```bash
./target/debug/hela --tool md5
```

## Key Bindings

- **j/k** - Navigate up/down in toolbox
- **Enter** - Run selected tool
- **Tab** - Cycle through input fields
- **Ctrl+Q** / **q** - Force quit
- **Arrow keys** - Navigate within panels

## License

MIT License

Copyright (c) 2024 Hela Project

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.", "encoding": "utf-8