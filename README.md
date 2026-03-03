# BARYCENTER

**A zero-bloat, Wayland-native application launcher and command hub written in Rust.**

[![Rust](https://img.shields.io/badge/Rust-2024_Edition-f74c00?logo=rust&logoColor=white)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Wayland](https://img.shields.io/badge/Wayland-Native-yellow?logo=wayland&logoColor=white)](https://wayland.freedesktop.org)
[![Maintenance](https://img.shields.io/badge/Maintained-Actively-brightgreen.svg)](https://github.com/cptdawn/BaryCenter/commits/main)

![Demo](assets/demo.gif)

---

## Philosophy

Most application launchers are relics of the X11 era — ported forward with compatibility shims, dynamically linked against half of GTK, and polling the filesystem on every keystroke. Barycenter exists because the modern Wayland desktop deserves better.

Built from scratch in Rust with `iced` and `iced_layershell`, Barycenter talks directly to the Wayland compositor via the `wlr-layer-shell` protocol. There is no X11 fallback, no GLib main loop, and no runtime garbage collector. The binary is statically optimized with LTO and ships at ~12 MB stripped — compositor blur, exclusive keyboard grab, and sub-millisecond fuzzy matching included.

The architecture follows a strict **modular Runner pattern**: each capability (application search, calculator, system commands) is an isolated module behind a single trait. Adding a new runner is adding a file, not touching a framework.

> [!IMPORTANT]
> Barycenter requires a Wayland compositor that implements the **`wlr-layer-shell-unstable-v1`** protocol. This includes **Hyprland**, **Sway**, **river**, and **labwc**. GNOME and KDE Plasma do not support this protocol natively.

---

## Features

- **Wayland layer-shell native** — renders on `Layer::Overlay` via `wlr-layer-shell`, no X11/XWayland dependency
- **Spawns on the active monitor** — `StartMode::Active` places the window where your focus already is
- **Exclusive keyboard grab** — `KeyboardInteractivity::Exclusive` captures input immediately on launch; no click-to-focus
- **Sub-millisecond fuzzy search** — powered by [`nucleo-matcher`](https://github.com/helix-editor/nucleo), the same engine behind the Helix editor
- **Modular Runner architecture** — pluggable `Runner` trait with three built-in modules:
  - `AppRunner` — parses `.desktop` files at startup, caches in memory, deduplicates, fuzzy-matches with scored ranking
  - `CalcRunner` — real-time math evaluation via [`meval`](https://crates.io/crates/meval); result copies to clipboard with `wl-copy`
  - `SysRunner` — system commands (lock, logout, reboot, shutdown, suspend, hibernate) via `loginctl`/`systemctl`
- **Embedded config bootstrap** — a beautifully commented `config.toml` is baked into the binary at compile time and written to disk on first run via the XDG base directory spec
- **"Cosmic Dawn" theme** — deep purple borders, translucent dark background with compositor blur passthrough, soft off-white text, vivid accent highlights — fully configurable via hex RGBA
- **Zero-polling** — event-driven architecture; the process is idle until you type
- **Minimal dependency surface** — no GTK, no Qt, no Electron; just `iced`, `iced_layershell`, and a handful of focused crates
- **Release-optimized** — `opt-level = 3`, LTO, single codegen unit, stripped symbols

---

## Installation

### Building from Source

Requires a working Rust toolchain (1.85+ recommended). The Wayland development libraries (`wayland-client`, `wayland-protocols`) must be present on the system.

```bash
git clone https://github.com/cptdawn/BaryCenter.git
cd BaryCenter
cargo build --release
```

The binary is written to `target/release/barycenter`. Copy it to a location on your `$PATH`:

```bash
sudo install -Dm755 target/release/barycenter /usr/local/bin/barycenter
```

### Arch Linux (AUR)

```bash
yay -S barycenter
```

> [!NOTE]
> The AUR package name is a placeholder and will be updated once the package is published.

---

## Configuration

Barycenter uses an **embedded asset bootloader** pattern:

1. A fully commented default configuration is compiled into the binary via `include_str!` at build time.
2. On first launch, if `~/.config/barycenter/config.toml` does not exist, the embedded default is written to disk.
3. On every subsequent launch, the on-disk file is read and parsed. User edits are always respected.
4. To reset to defaults, simply delete the file — it will be regenerated on the next run.

### Config Location

```
~/.config/barycenter/config.toml
```

### Default Configuration

```toml
# ─── Window Geometry ──────────────────────────────────────────────────────────
width = 680
height = 480

# ─── Colors (hex RRGGBBAA — last two digits are alpha) ────────────────────────
border_color     = "#7B2FBEff"   # Vivid deep purple border
background_color = "#0D0B1Fcc"   # Near-black, translucent (compositor blur)
text_color       = "#E8E0F0ff"   # Soft off-white
accent_color     = "#A855F7ff"   # Selected result highlight

# ─── Typography ───────────────────────────────────────────────────────────────
font_family = "Inter"            # System font (fallback: iced default sans-serif)
font_size   = 22.0               # Base size for the search input (px)

# ─── Behavior ─────────────────────────────────────────────────────────────────
max_results   = 8                # Maximum visible results
border_width  = 2.0              # Window border thickness (px)
border_radius = 12.0             # Corner rounding (px)
```

| Key | Type | Description |
|---|---|---|
| `width` / `height` | `u32` | Window dimensions in pixels. Centered on the active monitor. |
| `border_color` | `String` | `#RRGGBB` or `#RRGGBBAA` hex color for the window border. |
| `background_color` | `String` | Window background. Set the alpha channel below `ff` for compositor blur passthrough. |
| `text_color` | `String` | Primary text color for input and result titles. |
| `accent_color` | `String` | Highlight color for the selected result row and badges. |
| `font_family` | `String` | Preferred font family name. Must be installed on the system. |
| `font_size` | `f32` | Base font size in pixels. Result text scales proportionally. |
| `max_results` | `usize` | Maximum number of search results rendered at once. |
| `border_width` | `f32` | Border thickness around the launcher window. |
| `border_radius` | `f32` | Corner radius for the window and result row containers. |

---

## Usage

Barycenter is designed to be launched on-demand via a compositor keybind. It captures the keyboard, runs the query, and exits after execution or dismissal.

### Hyprland

Add to `~/.config/hypr/hyprland.conf`:

```ini
bind = $mainMod, SPACE, exec, barycenter
```

### Sway

Add to `~/.config/sway/config`:

```
bindsym $mod+space exec barycenter
```

### Keyboard Shortcuts

| Key | Action |
|---|---|
| *Any text* | Fuzzy search across applications, calculator, and system commands |
| `↑` / `↓` | Navigate results |
| `Tab` | Move to next result |
| `Enter` | Execute the selected result |
| `Escape` | Dismiss the launcher |

### Runner Behavior

- **Applications** — Always active. Searches `.desktop` entries from `/usr/share/applications`, `/usr/local/share/applications`, and `~/.local/share/applications`. Deduplicates by name (user entries override system). Launches the `Exec` command directly.
- **Calculator** — Activates when input contains digits and a math operator (`+`, `-`, `*`, `/`, `^`, `%`, `(`). Pressing `Enter` on a calc result copies the value to the Wayland clipboard via `wl-copy`.
- **System** — Activates on fuzzy match against keywords: `lock`, `logout`, `reboot`, `shutdown`, `suspend`, `hibernate`. Executes via `loginctl` / `systemctl`.

### Runtime Dependencies

| Dependency | Required By | Purpose |
|---|---|---|
| `wl-copy` | CalcRunner | Clipboard access for calculator results |
| `loginctl` | SysRunner | Session locking, logout |
| `systemctl` | SysRunner | Reboot, shutdown, suspend, hibernate |

---

## Architecture

```
src/
├── main.rs           # Entrypoint: logging → config boot → runner init → layer-shell launch
├── config.rs         # Embedded asset bootloader, XDG path resolution, TOML parsing
├── search.rs         # nucleo-matcher wrapper (fuzzy_score, fuzzy_rank)
├── runner/
│   ├── mod.rs        # Runner trait + RunnerResult struct
│   ├── app.rs        # .desktop file parser, in-memory cache, fuzzy search, Command launch
│   ├── calc.rs       # Math expression detection + meval evaluation + wl-copy clipboard
│   └── sys.rs        # System command keywords + loginctl/systemctl execution
└── ui/
    ├── mod.rs        # iced application: state machine, message handling, view rendering
    └── theme.rs      # Cosmic Dawn: container, text input, scrollable, result row styles
```

---

## Contributing

Contributions are welcome. Please open an issue before submitting large changes to discuss the approach.

1. Fork the repository
2. Create a feature branch (`git checkout -b feat/your-feature`)
3. Commit with clear messages
4. Open a Pull Request against `main`

All code must pass `cargo check`, `cargo clippy`, and `cargo fmt --check` with no warnings.

---

## License

[MIT](LICENSE) © 2026 Swastik Patel