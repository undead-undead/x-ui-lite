# X-UI-Lite v2.5.9 - Critical Performance Patch 🚀

This release includes **xray-lite v0.2.78** which introduces a critical performance optimization: **Smart Write Buffering**.

### 🛠 Fixes & Improvements
- **100% Speed Recovery**: Solved the issue where YouTube/Netflix speeds were capped at ~70-80% of bandwidth.
- **Smart Write Buffer**: Added a 14KB intelligent buffer to the Reality TLS layer.
  - **Before**: Every small packet (e.g., 50 bytes) triggered a full encryption block + syscall.
  - **After**: Small packets are aggregated into fewer, larger blocks, reducing CPU load by up to **90%** and maximizing throughput.

### 📦 Upgrade Now
Run the one-click script to upgrade automatically:

```bash
bash <(curl -Ls https://raw.githubusercontent.com/undead-undead/x-ui-lite/main/install.sh)
```

**Full Changelog**: [CHANGELOG.md](https://github.com/undead-undead/x-ui-lite/blob/v2.5.9/CHANGELOG.md)
