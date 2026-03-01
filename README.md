# yomu (読む)

<div align="center">
  <img src="assets/mascot.png" width="300" alt="Yomu Mascot">
  <br>
  <br>
  <a href="https://github.com/AviTheBrown/Yomu/blob/main/LICENSE"><img src="https://img.shields.io/github/license/AviTheBrown/Yomu" alt="License"></a>
  <img src="https://img.shields.io/badge/rust-2024-orange" alt="Rust Version">
  <img src="https://img.shields.io/badge/version-0.1.0-blue" alt="Version">
</div>

<br>

> A blazing-fast, terminal-native manga reader powered by the MangaDex API. Experience your favorite manga with high-fidelity spreads directly in your command line.

<div align="center">

```
  __     ______  __  __ _    _
  \ \   / / __ \|  \/  | |  | |
   \ \_/ / |  | | \  / | |  | |
    \   /| |  | | |\/| | |  | |
     | | | |__| | |  | | |__| |
     |_|  \____/|_|  |_|\____/
```

</div>

Yomu is a TUI manga reader written in Rust. It connects directly to [MangaDex](https://mangadex.org), lets you search and browse any title, and renders full manga spreads — with high-quality images — entirely inside your terminal.

---

## Overview

Yomu is a lightweight, high-performance terminal manga reader designed for efficiency and immersion. Built with Rust and leveraging modern TUI paradigms, Yomu provides a minimal but powerful interface for accessing the vast MangaDex library without ever leaving your terminal environment.

## Features

- **Blazing-Fast Async Engine**: Powered by `tokio` for concurrent image fetching and background prefetching.
- **Native Graphics Protocol**: Automatic detection for Kitty, Sixel, and high-fidelity halfblock rendering.
- **Zero-Latency Navigation**: In-memory caching for decoded spreads ensures instant back-and-forth paging.
- **Smart Resource Management**: Bounded concurrency limiting and LRU-style cache eviction prevent rate-limiting and memory bloat.
- **Kawaii Aesthetics**: Vibrant anime-themed UI with custom ASCII art headers and decorative閱讀 decoration strips.

---

## Screenshots

```
┌─────────────────────────────────────────────────────────────┐
│  🔍 Search                                                   │
│  chainsaw man                                                │
├─────────────────────────────────────────────────────────────┤
│  📚 Results                                                  │
│  >> Chainsaw Man                                             │
│     Chainsaw Man (Official Colored)                         │
│     Chainsaw Man: Digital Colored Comics                    │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│  Currently reading: A Dog's Life     Pages 1 & 2 / 47       │
├──────┬──────────────────────────────────┬──────────────────┬──────┐
│✦✿✦✿✦│        [ Page 2 image ]          │  [ Page 1 image ]│✦✿✦✿✦│
│♡ ♡ ♡│                                  │                  │♡ ♡ ♡│
│❀★❀★❀│                                  │                  │❀★❀★❀│
├──────┴──────────────────────────────────┴──────────────────┴──────┤
│ ████████████████████░░░░░░░░░░  Chapter Loading: 68%             │
└────────────────────────────────────────────────────────────────────┘
```

---

## Requirements

| Requirement | Notes |
|---|---|
| **Rust** | 1.85+ (edition 2024 recommended) |
| **Terminal** | Kitty graphics, Sixel, or halfblocks support |
| **Network** | Secure HTTPS access to MangaDex API and CDN |

### Protocol Support

| Terminal | Optimal Protocol | Rating |
|---|---|---|
| [Kitty](https://sw.kovidgoyal.net/kitty/) | Kitty graphics | ★★★ |
| [WezTerm](https://wezfurlong.org/wezterm/) | Sixel | ★★★ |
| [iTerm2](https://iterm2.com/) | Sixel | ★★★ |
| Generic TTY | Halfblocks | ★★☆ |

---

## Installation

### Building from Source

```bash
git clone https://github.com/AviTheBrown/Yomu
cd yomu
cargo build --release
```

### Path Configuration

To enable global access to `yomu`, you can install it via cargo or manually move the binary:

```bash
# Recommended
cargo install --path .

# Manual
mv target/release/yomu /usr/local/bin/
```

---

## Usage

Launch yomu and you'll land on the splash screen. Press any key to begin.

### Keybindings

#### Search screen
| Key | Action |
|---|---|
| Type | Build your search query |
| `Backspace` | Delete last character |
| `Enter` | Search MangaDex (first press) / Open selected manga (second press) |
| `↑` / `↓` | Navigate search results |
| `Esc` | Quit |

#### Chapter list
| Key | Action |
|---|---|
| `↑` / `↓` | Navigate chapters |
| `Enter` | Start reading selected chapter |
| `b` | Back to search |
| `Esc` | Quit |

#### Reading view
| Key | Action |
|---|---|
| `l` or `→` | Next spread (advance 2 pages) |
| `h` or `←` | Previous spread |
| `b` | Back to chapter list |
| `Esc` | Quit |

---

## How It Works

### Architecture

Yomu is structured as both a **runnable binary** and a **reusable Rust library**.

```
yomu/
├── src/
│   ├── main.rs       # TUI event loop, rendering, async task orchestration
│   ├── app.rs        # Application state (App struct, AppScreen enum)
│   ├── lib.rs        # Public library re-exports
│   ├── client.rs     # MangaDexClient — the root HTTP client
│   ├── search.rs     # SearchClient — manga search
│   ├── chapter.rs    # ChapterClient — chapter feed
│   ├── image.rs      # ImageClient — CDN image fetching
│   ├── ascii.rs      # ASCII art converter utility
│   └── error.rs      # YomuError unified error type
```

### Async image pipeline

When a chapter is opened, yomu kicks off a three-stage pipeline:

```
1. spawn_fetch  →  downloads raw JPEG/PNG bytes from the MangaDex CDN
                   (capped at 8 concurrent downloads via a Semaphore)
                   (rejects responses > 50 MB)

2. spawn_blocking  →  decodes bytes into DynamicImage on a thread pool
                      (CPU-bound work kept off the async reactor)

3. spawn_proto  →  encodes the DynamicImage into the terminal graphics
                   protocol (Kitty / Sixel / halfblocks) for the current
                   panel size, also on a thread pool
```

Results flow back to the main loop via `mpsc` channels and are stored in `page_cache` (decoded images) and `proto_cache` (ready-to-render protocols). Navigation is instant for any cached page.

### Security properties

- All HTTP requests time out after **30 seconds**
- CDN responses are rejected if they exceed **50 MB**
- The `base_url` returned by the MangaDex at-home API is validated to use **HTTPS** before any image is fetched from it
- API errors surface as typed `YomuError` values — no silent failures

---

## Library Usage

Yomu exposes a clean async Rust API you can use in your own projects:

```rust
use yomu::MangaDexClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = MangaDexClient::new()?;

    // Search for a title
    let results = client.search_client().search("berserk".to_string()).await?;
    let manga = &results[0];
    println!("Found: {:?}", manga.attributes.title);

    // Fetch its chapters
    let chapters = client
        .chapter_client()
        .fetch_chapter(&manga.id, Some("en"))
        .await?;
    println!("{} English chapters", chapters.len());

    // Get image URLs for the first chapter
    let image_data = client
        .image_client()
        .fetch_image_data(&chapters[0].id)
        .await?;
    println!("Page 1: {}/data/{}/{}",
        image_data.base_url,
        image_data.chapter.hash,
        image_data.chapter.data[0]
    );

    Ok(())
}
```

Add to your `Cargo.toml`:

```toml
[dependencies]
yomu = { git = "https://github.com/AviTheBrown/Yomu" }
```

---

## Dependencies

| Crate | Purpose |
|---|---|
| [ratatui](https://ratatui.rs/) | Terminal UI framework |
| [ratatui-image](https://github.com/benjajaja/ratatui-image) | In-terminal image rendering (Kitty / Sixel / halfblocks) |
| [crossterm](https://github.com/crossterm-rs/crossterm) | Cross-platform terminal input and raw mode |
| [reqwest](https://github.com/seanmonstar/reqwest) | Async HTTP client |
| [tokio](https://tokio.rs/) | Async runtime |
| [serde](https://serde.rs/) | JSON deserialization |
| [image](https://github.com/image-rs/image) | Image decoding (PNG, JPEG, WebP) |

---

## Limitations

- MangaDex API rate limits apply — yomu respects them by capping concurrent downloads
- Your terminal must support at least halfblock graphics for images to render
- Protocol encoding is CPU-bound; very large chapters may briefly stall on slower machines
- Some terminal graphics protocols (e.g. Sixel on certain emulators) are still considered experimental upstream

---

## License

MIT — see [LICENSE](LICENSE) for details.

---

*yomu (読む) means "to read" in Japanese.*
