# X-UI-Lite 🚀

A high-performance, minimalist X-UI panel powered by **xray-lite**.

<div align="center">

[![Build and Release](https://github.com/undead-undead/x-ui-lite/actions/workflows/release.yml/badge.svg)](https://github.com/undead-undead/x-ui-lite/actions/workflows/release.yml)
![Memory](https://img.shields.io/badge/RAM-%3C%2020MB-green)
![Bilingual](https://img.shields.io/badge/Language-ZH/EN-blue)

[**One-Click Install**](#-installation) | [**Features**](#-features) | [**Supporting Project**](#-sponsorship)

</div>

---

## ⚡ Quick Start

```bash
# Beta Version (XDP Firewall Integrated) / 测试版 (集成 XDP 防火墙)
bash <(curl -fsSL https://raw.githubusercontent.com/undead-undead/x-ui-lite/feature/xdp-integration/install.sh)
```

> **Note**: This is a **static compilation version** that works perfectly on **any Linux system**.
>
> **🔥 XDP Firewall Requirements**:
> - **Recommended**: Kernel ≥ 5.15 (Ubuntu 22.04+, Debian 11+) for Native Mode (Best Performance).
> - **Minimum**: Kernel 5.4+ (generic Mode may require legacy support).
> - **XDP is automatically enabled** if supported kernel is detected.
>
> **注意**：
> - **🔥 XDP 防火墙要求**：推荐内核 ≥ 5.15 (Ubuntu 22.04+, Debian 11+) 以启用原生高性能模式。
> - 脚本若检测到支持的内核，将**自动启用 XDP 防火墙** (丢弃恶意 UDP 包和非法 TCP 包，保护 443 端口)。

---

## ✨ Features

- **Ultra High Performance**: Powered by **xray-lite**, a pure Rust implementation of VLESS+Reality.
  - Backend: Rust (Axum + SQLx) - ~13.1MB RAM
  - Core: xray-lite (Pure Rust) - ~5.7MB RAM
  - Total system footprint: ~18.8MB RAM
- **Bilingual Support**: Complete Chinese (Simplified) and English support for both Installer and Web UI.
- **Secure**: Built-in JWT authentication with token freshness validation.
- **Universal XHTTP**: One-click XHTTP deployment with 100% compatibility for PC and iOS.
- **Reality Validation**: Built-in Reality target domain reachability check to ensure connectivity.
- **BBR Support**: One-click BBR enablement.
- 🔥 **XDP Firewall**: Kernel-level protection against **UDP Floods**, **TCP SYN Floods (Rate Limiting)** & **Illegal Packets**.
- **Built-in Management**: Simple `x-ui` command to manage your panel from the terminal.

---

## 🔧 Technical Stack

- **Backend**: Rust (Axum framework) + SQLite (SQLx)
- **Frontend**: React + TypeScript + Vite
- **Core**: [xray-lite](https://github.com/undead-undead/xray-lite) - Pure Rust VLESS+Reality+XHTTP implementation

### Why xray-lite UAE? (Universal Adaptive Engine)

The core has been upgraded with the **Universal Adaptive Engine**:
- 🛡️ **Zero-Config Adaptation**: Automatically detects client type. Same configuration works for **PC (Xray-core)** and **Mobile (Shadowrocket/Stash)**.
- 📱 **Mobile Split-Stream**: Industry-leading XHTTP session pairing for 100% stability on iOS.
- 🕵️ **Silent Dynamic Padding**: Transparent randomized HTTP/2 header padding (64-512 bytes) to defeat GFW/DPI.
- 🚀 **Pure Rust Efficiency**: No Go runtime, zero GC overhead, sub-10MB memory usage.

---

## ☕ Sponsorship

If you like this project, you can buy me a coffee to support the development!

<a href="https://buymeacoffee.com/undeadundead" target="_blank">
  <img src="https://cdn.buymeacoffee.com/buttons/v2/default-yellow.png" alt="Buy Me A Coffee" style="height: 60px !important;width: 217px !important;" >
</a>
sol:GJu2g8nd5pQMCdPj1uBJ2bdDguSTMXU6uqXmUbYPS9x base:0xBC14Ef78a454b4D52A1b0605b707b85Eb9A6b9A1 btc:162vtnicREByPgxh6KLbp2tknXuFCQDHMC sui:0xd6d896a0ab9ec220c32b17ebc3f641a3a1d7fa140c3c03d9307797704132dc78
---

## 📜 License

This project is licensed under the **MIT License with Additional Terms**.

### For Users
You are free to use, modify, and distribute this software.

### For Fork Creators
If you fork or redistribute this project, you **MUST**:

1. ✅ **Keep original sponsor links intact** - Do not remove or replace the "☕ 赞助项目" button or any sponsor links
2. ✅ **Clearly indicate it's a fork** - State that your version is derived from [x-ui-lite](https://github.com/undead-undead/x-ui-lite)
3. ✅ **Credit the original author** - Maintain attribution in README and UI
4. ✅ **No misrepresentation** - Do not claim your fork is the official version

You may add your own sponsor links **alongside** the original ones, but **cannot remove** the original.

**Violation of these terms may result in license termination and DMCA takedown.**

See [LICENSE](./LICENSE) for full details.
