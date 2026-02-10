# X-UI-Lite

A high-performance, minimalist X-UI panel powered by **xray-lite**.

[One-Click Install](#installation) | [Features](#features) | [Supporting Project](#sponsorship)

---

## Quick Start

### Installation / 安装

> **Current Version: v2.9.8**
>
> **Included Kernels (Switchable in Panel):**
> - **Stable**: v0.4.6 (Tokio) - Recommended for production
> - **XDP Firewall**: v0.6.0-xdp - Anti-DDoS/Flood protection (Kernel 5.4+)

```bash
bash <(curl -fsSL https://raw.githubusercontent.com/undead-undead/x-ui-lite/main/install.sh)
```

> **Note**: This is a **static compilation version** that works perfectly on **any Linux system**.
>
> **注意**：
> - 脚本默认安装**稳定版**内核。
> - 如需 **XDP 防火墙**模式，请在面板的【系统状态】->【切换版本】中选择对应版本。

### XDP Firewall (Optional) / XDP 防火墙（可选）

If you switch to **v0.6.0-xdp** in the panel:

*   **XDP Firewall**: Kernel-level protection against **UDP Floods**, **TCP SYN Floods (Rate Limit)** & **Illegal Flags**.
*   **Anti-Probe**: Instantly drops **UDP Floods** and **Illegal TCP Packets** (e.g., Null Scan, SYN+FIN).
*   **Adaptive Egress Pacing (TC eBPF)**: Microsecond-precision traffic smoothing to eliminate proxy-burst timing fingerprints.
*   **Performance**: Drop malicious packets at driver level, saving CPU.

> **Requirements for XDP**: Linux Kernel ≥ 5.4, Root Privileges.
> **XDP is automatically enabled** if supported kernel is detected and XDP version is selected.

---

## Features

- **Ultra High Performance**: Powered by **xray-lite**, a pure Rust implementation of VLESS+Reality.
  - Backend: Rust (Axum + SQLx) - ~13.1MB RAM
  - Core: xray-lite (Pure Rust) - ~5.7MB RAM
  - Total system footprint: ~18.8MB RAM
- **Bilingual Support**: Complete Chinese (Simplified) and English support for both Installer and Web UI.
- **Secure**: Built-in JWT authentication with token freshness validation.
- **Reality Validation**: Built-in Reality target domain reachability check to ensure connectivity.
- **BBR Support**: One-click BBR enablement.
- **XDP Firewall**: Kernel-level protection against **UDP Floods**, **TCP SYN Floods (Rate Limiting)** & **Illegal Packets**.
- **Built-in Management**: Simple `x-ui` command to manage your panel from the terminal.

---

## Technical Stack

- **Backend**: Rust (Axum framework) + SQLite (SQLx)
- **Frontend**: React + TypeScript + Vite
- **Core**: [xray-lite](https://github.com/undead-undead/xray-lite) - Pure Rust VLESS+Reality+XHTTP implementation

### Why xray-lite UAE? (Universal Adaptive Engine)

The core has been upgraded with the **Universal Adaptive Engine**:
- **Zero-Config Adaptation**: Automatically detects client type. Same configuration works for **PC (Xray-core)** and **Mobile (Shadowrocket/Stash)**.
- **Mobile Split-Stream**: Industry-leading XHTTP session pairing for 100% stability on iOS.
- **Silent Dynamic Padding**: Transparent randomized HTTP/2 header padding (64-512 bytes) to defeat GFW/DPI.
- **Pure Rust Efficiency**: No Go runtime, zero GC overhead, sub-10MB memory usage.

---

## Sponsorship

If you think the project is good, you can support the developers.

https://buymeacoffee.com/undeadundead

crypto:

Sol: 9QFKQ3jpBSuNPLZQH1uq5GrJm4RDKue82zeVaXwazcmj

Base：0x4cf0b79aea1c229dfb1df9e2b40ea5dd04f37969

---

## License

This project is licensed under the **MIT License with Additional Terms**.

### For Users
You are free to use, modify, and distribute this software.

### For Fork Creators
If you fork or redistribute this project, you **MUST**:

1. **Keep original sponsor links intact** - Do not remove or replace the sponsorship button or any sponsor links
2. **Clearly indicate it's a fork** - State that your version is derived from [x-ui-lite](https://github.com/undead-undead/x-ui-lite)
3. **Credit the original author** - Maintain attribution in README and UI
4. **No misrepresentation** - Do not claim your fork is the official version

You may add your own sponsor links **alongside** the original ones, but **cannot remove** the original.

**Violation of these terms may result in license termination and DMCA takedown.**

See [LICENSE](./LICENSE) for full details.
