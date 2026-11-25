# Hela-TRM

Hela is a terminal arsenal: a keyboard-driven TUI for cryptographic transforms, network checks, file triage, and machine facts. It opens with an animated Hela wordmark and a custom half-block portrait, then hands you a workspace of focused tools.

Built in Rust with [Ratatui](https://ratatui.rs). Version 0.2.0. MIT licensed.

## What it does

- Boots through a short animated sequence with a custom "H" wordmark, boot log, and progress bar.
- Introduces an operator panel with live system facts and a custom portrait (full or compact), revealed row by row with a travelling scanline.
- Exposes 12 tools in a keyboard-navigable arsenal list. Each tool keeps its own inputs, focus, output, scroll position, and running state.
- Runs tools on background threads so the interface stays responsive while output streams in.
- Adapts to the terminal. The portrait switches to a compact drawing when space is tight, and the toolbox falls back to a readable notice on very small screens.

## Tools

| Tool | Group | Fields and defaults | What it does |
| --- | --- | --- | --- |
| `hash-forge` | crypto | text: `the gate is open` | MD5, SHA-1, SHA-256, and SHA-512 of the input text. |
| `hash-id` | crypto | digest: empty | Guesses the digest family from hex width and recognises a few known sample digests. Identification, not cracking. |
| `base64` | crypto | text: `the gate is open` | Encodes to standard and URL-safe base64, then tries padded and unpadded decoding and previews the result. |
| `caesar` | crypto | text, shift: `3` | Rotates ASCII letters by the shift and lists all 25 non-zero brute-force candidates. |
| `jwt-peel` | crypto | token: bundled sample | Decodes JWT header and payload as pretty JSON. The signature segment is reported by length only and never verified. |
| `wordlist` | crypto | pattern: `hela`, limit: `256` | Generates case permutations of the seed word. The limit is clamped to 1-4096. |
| `password-forge` | crypto | length: `20`, count: `5` | Generates random passwords containing lower case, upper case, digits, and symbols. Length is clamped to 8-128, count to 1-50. |
| `port-scan` | network | host: `127.0.0.1`, ports: `22,80,443,8080`, timeout: `300` ms | Concurrent TCP connect scan over comma-separated ports and ranges. Up to 64 workers, 2,048 unique ports, and a 50-5000 ms timeout. |
| `dns` | network | host: `github.com` | Resolves a host through the system resolver and lists every unique IPv4/IPv6 address. |
| `http-headers` | network | url: `https://github.com` | Sends a HEAD request with a 12 second timeout and prints status plus response headers. Retries with GET only when the server rejects HEAD with 405. |
| `autopsy` | files | path: empty | Reads up to the first 32 MiB of a file and reports magic bytes, Shannon entropy, SHA-256, MD5, the first 256 bytes as a hexdump, and a text preview. |
| `sysinfo` | system | none | Reports operator, host, OS, kernel, architecture, CPU, memory, uptime, shell, terminal, local IP, and working directory. |

### Notes per tool

- `autopsy` hashes, calculates entropy for, and previews the sampled bytes. When a file is larger than 32 MiB, the report says so; the hashes are of the sample, not the whole file.
- `port-scan` is a plain TCP connect scan. It is not stealthy and does not identify services beyond a small built-in port label table.
- `base64` treats any valid base64-looking input as decodable. A short plain word can be valid base64 too, so the decode section may show binary data.
- `hash-id` can only infer a family from digest length or match a tiny set of known fingerprints. It cannot recover the original input.
- `jwt-peel` accepts a raw token or one prefixed with `Bearer `.
- `sysinfo` reads Linux `/proc` when available and falls back gracefully on other systems.

## Requirements

- A recent stable Rust toolchain with Cargo. Tested with Rust 1.96.
- A native C toolchain and linker for dependencies. On Debian/Ubuntu, install `build-essential` and get Rust from [rustup.rs](https://rustup.rs). On Alpine Linux, the app was built and tested with `rust`, `cargo`, `build-base`, `cmake`, and `perl` installed; the exact native prerequisites can vary with the TLS dependency build.
- A Unicode-capable terminal. Truecolor and a monospace font are recommended for the intended palette, portrait, and block glyphs.

The app was developed and tested on Linux aarch64 (Alpine). It uses cross-platform Ratatui/Crossterm APIs but is not currently tested on Windows.

## Install and run

From source:

```bash
git clone https://github.com/prettycashless/Hela-TRM.git
cd Hela-TRM
cargo run --release
```

Build a release binary:

```bash
cargo build --release
./target/release/hela
```

Install the `hela` binary into Cargo's bin directory:

```bash
cargo install --path .
hela
```

There are no command-line tool flags. Everything is driven from the TUI. Hela does not write configuration or state files; sessions live only for the lifetime of the process.

## Controls

| Screen | Keys |
| --- | --- |
| Global | `Ctrl+Q` or `Ctrl+C` quit. |
| Boot | Any key skips the boot sequence. `Ctrl+Q`/`Ctrl+C` still quit. |
| Home | `Enter`, `Space`, `Tab`, `Down`, or `j` opens the arsenal. `Esc` or `q` quits. |
| Toolbox | `Up`/`Down` select a tool (wraps). `Tab`/`Shift+Tab` cycle fields. `Enter` runs or reruns the selected tool. `Esc` returns home. |
| Toolbox fields | Printable characters append to the focused field. `Backspace` deletes one character, `Ctrl+U` clears the field, `Ctrl+W` deletes the previous word. |
| Output | `PageUp` scrolls back, `PageDown` scrolls toward live output. Starting a run resets the view to live. |

In the toolbox, `q` is a normal character and does not quit; use `Ctrl+Q` or `Ctrl+C`. Running a tool does not block navigation, and switching tools does not cancel a running job. The header shows an open channel while any tool is working.

Each tool's output is kept in memory, capped at the most recent 4,000 lines.

## Terminal layout

- About 100x34 or larger: full portrait plus the operator panel.
- 80x24: compact portrait and a complete toolbox workspace.
- Narrow and short terminals: the boot screen compacts, the home screen falls back to a wordmark and summary, and the toolbox shows a resize notice rather than a broken layout.

For the intended visual experience, use a terminal at least 80 columns wide and 24 rows tall. The full portrait needs a taller window.

## Safety and limits

Hela is a local convenience tool, not a security boundary.

- Only scan systems you own or have explicit permission to test.
- JWT decoding is not signature verification. Never treat decoded claims as authenticated without verifying the token properly.
- File magic, entropy, and text detection are heuristics; they do not prove what a file is or that it is safe.
- Generated passwords are printed in the terminal only. Hela does not copy them to a clipboard. Clear your scrollback if the output is sensitive.
- Network tools make direct outbound connections. There is no proxy configuration UI, retry policy, or scanning scheduler.
- A tool that runs in the background keeps running until it finishes. There is no cancel key.

## Development

```bash
cargo fmt
cargo test
cargo build --release
```

The test suite covers boot timing, colour blending, wordmark and portrait geometry, crypto primitives and generation limits, file magic/entropy/hexdump logic, port parsing, and app-level input and worker flow. One ignored test renders real frames to stdout through Ratatui's `TestBackend` for visual inspection:

```bash
cargo test preview_frames -- --ignored --nocapture
```

Module map:

```text
src/main.rs        terminal lifecycle, event loop, panic/cleanup handling
src/app.rs         app state, tool registry, input editing, worker channels
src/ui/            header/footer chrome plus boot, home, and toolbox screens
src/art.rs         animated "H" wordmark and half-block portrait rendering
src/art_data.rs    portrait source data
src/tools/         crypto, files, network, and sysinfo implementations
src/system.rs      platform facts with graceful fallbacks
src/theme.rs       dark palette and colour blending
```

## License

MIT. See [LICENSE](LICENSE).
