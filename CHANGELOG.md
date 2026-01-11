# Changelog

All notable changes to X-UI-Lite will be documented in this file.

## [2.0.0] - 2026-01-11

### 🚀 Major Changes

**Switched from Xray-Core to xray-lite**

This is a complete rewrite of the core proxy engine, replacing the official Go-based Xray-Core with pure Rust xray-lite implementation.

### ✨ New Features

- **Pure Rust Core**: xray-lite is written entirely in Rust, providing better performance and lower memory usage
- **Ultra-Low Memory**: Total system footprint reduced to ~60MB (Backend 50MB + xray-lite 10MB)
- **Zero GC Overhead**: No Go runtime, no garbage collection pauses
- **Built-in Anti-Probing**: Strict SNI validation prevents active server detection
- **Raw VLESS over H2**: Minimum latency with raw pipe transport

### 🔧 Technical Changes

- **Configuration Simplified**: Removed API, Stats, and Policy configurations (not needed for xray-lite)
- **Traffic Statistics Disabled**: xray-lite doesn't provide gRPC API for statistics
- **Version Detection**: Updated to support both `--version` and `-version` flags
- **Download Source**: Changed from XTLS/Xray-core to undead-undead/xray-lite

### 📝 Configuration Format

xray-lite uses a simplified configuration format:
- No `api` section
- No `stats` section
- No `policy` section
- Only core inbound/outbound/routing configuration

### 🔄 Migration Notes

When upgrading from v1.x to v2.x:

1. Your existing inbound configurations will be automatically converted
2. Traffic statistics will no longer update (limitation of xray-lite)
3. All other panel features remain the same

### ⚠️ Breaking Changes

- Traffic quota enforcement still works but traffic counters won't increase (API limitation)
- If you need traffic statistics, please continue using v1.x with Xray-Core

### 🙏 Credits

- [xray-lite](https://github.com/undead-undead/xray-lite) - Pure Rust VLESS+Reality implementation
- [Xray-core](https://github.com/XTLS/Xray-core) - Original Reality protocol design

---

## [1.1.88] - Previous Version

See [RELEASE_NOTES_v1.1.88.md](./RELEASE_NOTES_v1.1.88.md) for details about the previous version.
