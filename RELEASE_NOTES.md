# X-UI-Lite v2.5.8 - High Performance Update

This release integrates the latest **xray-lite v0.2.77** core, bringing massive performance improvements.

### 🚀 Performance Boost
- **4x Throughput**: Increased internal buffer size from 16KB to 64KB. This solves the speed throttling issue on high-speed connections (e.g. YouTube 4K/8K).
- **Zero-Copy**: Implemented `BytesMut` for XHTTP transport to reduce CPU usage and memory copying overhead.
- **H2 Optimization**: Tuned HTTP/2 window sizes to match standard Xray-core behavior.

### 🛡️ Universal Compatibility
- Still fully static (Musl). Runs on any Linux distro out of the box.

### 🛠 Installation & Upgrade

```bash
bash <(curl -Ls https://raw.githubusercontent.com/undead-undead/x-ui-lite/main/install.sh)
```

**Full Changelog**: [CHANGELOG.md](https://github.com/undead-undead/x-ui-lite/blob/v2.5.8/CHANGELOG.md)
