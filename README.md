# X-UI-Lite 🚀

A high-performance, minimalist X-UI panel powered by **xray-lite**.

<div align="center">

[![Build and Release](https://github.com/undead-undead/x-ui-rs/actions/workflows/release.yml/badge.svg)](https://github.com/undead-undead/x-ui-rs/actions/workflows/release.yml)
![Memory](https://img.shields.io/badge/RAM-~50MB-green)
![Bilingual](https://img.shields.io/badge/Language-ZH/EN-blue)

[**One-Click Install**](#-installation) | [**Features**](#-features) | [**Supporting Project**](#-sponsorship)

</div>

---

## ⚡ Quick Start

Run the following command to install/update:

```bash
bash <(curl -Ls https://raw.githubusercontent.com/undead-undead/x-ui-lite-v2/main/install.sh)
```

---

## ✨ Features

- **Ultra High Performance**: Powered by **xray-lite**, a pure Rust implementation of VLESS+Reality.
  - Backend: Rust (Axum + SQLx) - ~50MB RAM
  - Core: xray-lite (Pure Rust) - ~10MB RAM
  - Total system footprint: ~60MB RAM
- **Bilingual Support**: Complete Chinese (Simplified) and English support for both Installer and Web UI.
- **Secure**: Built-in JWT authentication with token freshness validation.
- **Reality Validation**: Built-in Reality target domain reachability check to ensure connectivity.
- **BBR Support**: One-click BBR enablement.
- **Built-in Management**: Simple `x-ui` command to manage your panel from the terminal.

---

## 🔧 Technical Stack

- **Backend**: Rust (Axum framework) + SQLite (SQLx)
- **Frontend**: React + TypeScript + Vite
- **Core**: [xray-lite](https://github.com/undead-undead/xray-lite) - Pure Rust VLESS+Reality implementation

### Why xray-lite?

- 🚀 **Pure Rust**: No Go runtime, zero GC overhead
- 🪶 **Lightweight**: ~1.5MB binary, ~10MB memory
- 🔒 **Secure**: Built-in anti-probing with strict SNI validation
- ⚡ **Fast**: Raw VLESS over H2 for minimum latency

---

## ☕ Sponsorship

If you like this project, you can buy me a coffee to support the development!

<a href="https://buymeacoffee.com/undeadundead" target="_blank">
  <img src="https://cdn.buymeacoffee.com/buttons/v2/default-yellow.png" alt="Buy Me A Coffee" style="height: 60px !important;width: 217px !important;" >
</a>

---

## 📜 License

This project is licensed under the **MIT License with Additional Terms**.

### For Users
You are free to use, modify, and distribute this software.

### For Fork Creators
If you fork or redistribute this project, you **MUST**:

1. ✅ **Keep original sponsor links intact** - Do not remove or replace the "☕ 赞助项目" button or any sponsor links
2. ✅ **Clearly indicate it's a fork** - State that your version is derived from [x-ui-rs](https://github.com/undead-undead/x-ui-rs)
3. ✅ **Credit the original author** - Maintain attribution in README and UI
4. ✅ **No misrepresentation** - Do not claim your fork is the official version

You may add your own sponsor links **alongside** the original ones, but **cannot remove** the original.

**Violation of these terms may result in license termination and DMCA takedown.**

See [LICENSE](./LICENSE) for full details.

