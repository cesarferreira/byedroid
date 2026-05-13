<div align="center">
  <h1>byedroid</h1>

  <p><strong>Build, run, and debug Android apps from the terminal — no Android Studio required.</strong></p>

  <p>
    <a href="https://github.com/cesarferreira/byedroid/actions/workflows/rust-tests.yml"><img alt="CI" src="https://github.com/cesarferreira/byedroid/actions/workflows/rust-tests.yml/badge.svg"></a>
    <a href="https://crates.io/crates/byedroid"><img alt="Crates.io" src="https://img.shields.io/crates/v/byedroid"></a>
    <img alt="License" src="https://img.shields.io/badge/license-MIT-green">
  </p>

  <p>
    <a href="#install">Install</a>
    &nbsp;·&nbsp;
    <a href="#quickstart">Quickstart</a>
    &nbsp;·&nbsp;
    <a href="#keybindings">Keybindings</a>
    &nbsp;·&nbsp;
    <a href="#configuration">Configuration</a>
  </p>

  <br>

  <img src="assets/screnshot.png" width="880" alt="byedroid in action">
</div>

---

## Why byedroid

You don't need Android Studio eating 4 GB of RAM just to read logcat and tap "Run".

In the age of AI agents and terminal-first workflows, your editor is Cursor/Neovim/VS Code and your build system is Gradle on the command line. The only thing keeping Android Studio open was the run button and the log window. **byedroid** replaces both with a keystroke-driven TUI that starts in under a second.

- **One keystroke to run.** `n` builds, installs, and launches the app. That's it.
- **Instant startup.** A single Rust binary. No JVM, no IDE, no 30-second splash screen.
- **Crash-aware.** Crash blocks are highlighted in red, counted in the info bar, and `y` opens a full detail popup with copy, export, and agent-prompt shortcuts.
- **Smart Gradle inference.** Reads `app/build.gradle(.kts)`, resolves `productFlavors` and `flavorDimensions`, and picks the right variant automatically.
- **scrcpy-native.** `m` mirrors the device. If you use `scrcpy --new-display`, the app launches straight into the virtual display — no config needed.

## Install

<a id="install"></a>

```bash
cargo install byedroid
```

<details>
<summary><strong>Other installation methods</strong> — prebuilt binaries, from source</summary>

### Prebuilt binaries

Download the latest binary from [GitHub Releases](https://github.com/cesarferreira/byedroid/releases):

```bash
# macOS (Apple Silicon)
curl -fsSL https://github.com/cesarferreira/byedroid/releases/latest/download/byedroid-aarch64-apple-darwin.tar.gz | tar xz
# macOS (Intel)
curl -fsSL https://github.com/cesarferreira/byedroid/releases/latest/download/byedroid-x86_64-apple-darwin.tar.gz | tar xz
# Linux (x86_64)
curl -fsSL https://github.com/cesarferreira/byedroid/releases/latest/download/byedroid-x86_64-unknown-linux-gnu.tar.gz | tar xz
# Linux (arm64)
curl -fsSL https://github.com/cesarferreira/byedroid/releases/latest/download/byedroid-aarch64-unknown-linux-gnu.tar.gz | tar xz

mkdir -p ~/.local/bin
mv bd ~/.local/bin/
# Ensure ~/.local/bin is on your PATH:
# echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
```

### Build from source

```bash
cargo install --path . --locked
```

</details>

Verify the install:

```bash
bd --version
```

<a id="quickstart"></a>
## Quickstart

Run `bd` from your Android project root:

```bash
cd ~/projects/my-android-app
bd
```

`bd` opens immediately. If a device is connected, logcat starts automatically and follows the project's package(s). Hit `n` to build, install, and launch.

```bash
# Point at a project from anywhere
bd --project /path/to/my/android/app

# Scaffold a .byedroid.toml config
bd init

# Check your local toolchain and project setup
bd doctor
```

## Highlights

### Smart variant inference

`bd` reads `app/build.gradle(.kts)` on startup and resolves everything automatically:

```groovy
flavorDimensions "track", "environment"
productFlavors {
    canary { dimension "track" }
    stable { dimension "track" }
    dev    { dimension "environment"; applicationIdSuffix ".dev" }
    prod   { dimension "environment" }
}
```

Result: default variant **`canaryDevDebug`**, tasks **`assembleCanaryDevDebug`** / **`installCanaryDevDebug`**, packages `["com.example.app", "com.example.app.dev"]`. Switch variants at runtime with `v`.

### Crash & ANR detection

Crash blocks are highlighted in red as they stream in. The info bar shows a running count. Press `y` to open the crash detail popup:

- `c` — copy the full stack trace
- `a` — copy an agent prompt (paste straight into your AI assistant)
- `w` — export to `crash-<timestamp>.log`
- `s` — Google search the exception

### scrcpy virtual display

Start `scrcpy --new-display` to get a separate virtual display (keeps your phone screen free), then press `n` in byedroid — the app launches directly into the virtual display. Display ID is detected automatically via `dumpsys display`.

### Live filter + exclude

`f` opens an include filter across tag + message. `x` opens an exclude filter. Both update the viewport in real time. Persist noisy exclusions in `.byedroid.toml`:

```toml
exclude_filters = ["chatty", "ViewRootImpl", "ImeTracker"]
```

<a id="keybindings"></a>
## Keybindings

| Key | Action |
|-----|--------|
| `b` | Build (assemble only) |
| `i` | Install (assemble + install) |
| `n` | **Run** — install then launch the app |
| `v` | Variant picker |
| `d` | Device picker |
| `p` | Package filter picker |
| `l` | Toggle logcat on/off |
| `L` | Log level picker |
| `a` | Toggle all-logs / package-filter mode |
| `f` | Include filter |
| `x` | Exclude filter |
| `w` | Export visible log to `byedroid-<timestamp>.log` |
| `y` | Crash detail popup (`c` copy · `a` agent prompt · `w` export · `s` search) |
| `H` | Build history overlay |
| `Space` | Pause / resume log streaming |
| `↑` `↓` `j` `k` | Scroll logcat |
| `PageUp` `PageDown` | Scroll 20 lines |
| `End` `G` | Jump to tail |
| `e` | Expand / collapse build output |
| `c` | Clear log buffer |
| `m` | Launch scrcpy |
| `s` | Stop current Gradle / logcat process |
| `r` | Refresh device list |
| `q` | Quit |

<a id="configuration"></a>
## Configuration

| File | Purpose |
|------|---------|
| `~/.config/byedroid/config.toml` | Global: preferred device serial, default log level |
| `.byedroid.toml` | Per-project overrides |

**`.byedroid.toml` example:**

```toml
# Explicit package list (skips inference)
packages = ["com.example.app", "com.example.app.dev"]

# Override inferred Gradle tasks
assemble_task = "assembleCanaryDevDebug"
install_task  = "installCanaryDevDebug"

# Logcat
log_level       = "D"
log_filters     = ["OkHttp", "MyApp"]
exclude_filters = ["chatty", "ViewRootImpl"]

# scrcpy extra flags
scrcpy_args = ["--window-title", "MyApp Mirror"]
```

## Requirements

| Tool | Required | Notes |
|------|----------|-------|
| `adb` | Yes | Android SDK Platform Tools |
| `gradlew` | For build/install | Any standard Android project |
| `scrcpy` | No | For `m` screen mirror and `--new-display` support |

## Contributing

```bash
cargo test
```

To cut a release, push a `v*` tag — the release workflow builds and publishes binaries automatically.

## Related

- [rustycat](https://github.com/cesarferreira/rustycat) — logcat rendering style and parsing
- [dab](https://github.com/cesarferreira/dab) — ADB client helpers

## License

MIT &copy; Cesar Ferreira
