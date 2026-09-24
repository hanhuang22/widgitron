<p align="center">
  <img src="icons/widgitron.png" alt="Widgitron Logo" width="120" style="border-radius: 24px; box-shadow: 0 8px 30px rgba(0,0,0,0.15);" />
</p>

<h1 align="center">Widgitron</h1>

<p align="center">
  <strong>A high-performance, modular desktop widget framework for researchers and developers.</strong>
</p>

<p align="center">
  <a href="https://github.com/starkmomo/widgitron/releases">
    <img src="https://img.shields.io/badge/Version-v0.2.6-8B5CF6?style=flat-square&labelColor=2E1065&logo=github&logoColor=white" alt="Version" />
  </a>
  <a href="https://www.rust-lang.org/">
    <img src="https://img.shields.io/badge/Rust-1.75%2B-F97316?style=flat-square&logo=rust&logoColor=white&labelColor=431407" alt="Rust" />
  </a>
  <a href="https://tauri.app/">
    <img src="https://img.shields.io/badge/Tauri-2.0-24C6C1?style=flat-square&logo=tauri&logoColor=white&labelColor=083344" alt="Tauri" />
  </a>
  <a href="https://react.dev/">
    <img src="https://img.shields.io/badge/React-19-61DAFB?style=flat-square&logo=react&logoColor=white&labelColor=172554" alt="React" />
  </a>
  <a href="LICENSE">
    <img src="https://img.shields.io/badge/License-MIT-10B981?style=flat-square&labelColor=022C22" alt="License" />
  </a>
</p>

> [!TIP]
> Windows users can download the pre-compiled standalone executable directly from the [Releases](https://github.com/starkmomo/widgitron/releases) page.

Widgitron is a modern, cross-platform dashboard built with **Tauri**, **Rust**, and **React**. It provides a premium, glassmorphic UI for monitoring GPUs, conference deadlines, and arxiv research papers.

<p align="center">
  <img src="assets/quota_monitor.png" width="50%" />
  <img src="assets/gpu_monitor.png" width="46%" />
</p>
<p align="center">
  <img src="assets/deadline_demo.gif" width="49%" />
  <img src="assets/arxiv_radar_demo.gif" width="47%" />
</p>

## 🗺️ Roadmap

### ✅ Completed
- [x] GPU monitoring (Persistent SSH)
- [x] Slurm integration & Job ID tracking
- [x] Paper deadline countdown widget
- [x] Advanced widget theme customization
- [x] Arxiv Radar: paper card with swipe gestures
- [x] Agent quota monitor widget (Codex, Cursor, etc.)
- [x] Dockable sidebar hub


## 🚀 Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/starkmomo/widgitron.git
cd widgitron

# Install dependencies
pnpm install
```

### Run

```bash
# Development mode
pnpm tauri dev

# Build production executable
pnpm tauri build
```

### macOS build

Install the Xcode Command Line Tools, Node.js, and pnpm, then run the commands
above. The Rust build compiles its own OpenSSL for SSH support, so a separate
Homebrew OpenSSL installation is not required. The generated app and disk image
are under `src-tauri/target/release/bundle/`.

On macOS, Widgitron starts with the main dashboard. Open the sidebar from the
dashboard's **Open Sidebar** button or the menu bar icon. The sidebar stays
visible until you close it; **Pin Display** also opens it automatically at the
next launch. Use **Independent Widgets** on the dashboard to show individual
floating windows as needed, and **Hide All Widgets** to clear the desktop.
The menu bar icon also reopens the dashboard and hides all floating widgets.

The macOS default is Simplified Chinese; change it in **Settings → General →
Interface Language**. Existing macOS profiles receive a one-time display
migration: the legacy four-widget auto-open layout is cleared (explicitly
pinned widgets stay open), and legacy transparent widget themes switch to
readable presets. Later user choices are retained.

Windows-only edge reveal and global hotkey are not available on macOS.
Desktop embedding is Windows-only; macOS widgets remain regular windows and
can optionally be kept above other windows.
When a macOS update is available, the app opens its disk image for manual installation.

## 🤝 Contributing

Contributions welcome! Here's how:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-widget`)
3. Commit your changes (`git commit -m 'Add amazing widget'`)
4. Push to the branch (`git push origin feature/amazing-widget`)
5. Open a Pull Request

## 📄 License

MIT License - see [LICENSE](LICENSE) file for details.
