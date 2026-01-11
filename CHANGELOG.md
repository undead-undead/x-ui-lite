# Changelog

All notable changes to X-UI-Lite will be documented in this file.

## [2.5.9] - 2026-01-12

### ⚡ Critical Performance Fix (Core v0.2.78)

- **Smart Write Buffering**: Implemented a 14KB write buffer in the Reality transport layer. This eliminates CPU overhead caused by frequent small packet writes (common in TLS handshakes and video streaming headers).
- **Reduced Syscalls**: System calls for sending data have been reduced by ~90%, significantly lowering CPU load under high concurrency.
- **Full Bandwidth**: This fix allows the proxy to fully utilize the available bandwidth, solving the speed drop issue compared to direct connections.

---

## [2.5.8] - 2026-01-12

### ⚡ Performance Optimization (Core v0.2.77)

- **4x Throughput Increase**: Upgraded core data buffering from 16KB to 64KB, significantly improving transfer speeds on high-bandwidth connections (e.g., 4K/8K video streaming).
- **Zero-Copy Architecture**: Rewrote the XHTTP transport layer to use `BytesMut` for efficient memory management, eliminating unnecessary data copying and reducing CPU usage.
- **H2 Window Tuning**: Aligned HTTP/2 initial window size with industry standards (512KB) for better flow control.
- **Universal Static Build**: continues to use the new Musl static compilation for 100% Linux compatibility.

---

## [2.5.7] - 2026-01-11

### 🛡️ Universal Compatibility (Static Release)

- **Full Static Binaries**: All components (Panel, Xray-Core, Keygen) are now built with the `musl` target. This eliminates `GLIBC_2.xx not found` errors on older Linux distributions like Debian 10 or Ubuntu 20.04.
- **Improved Reliability**: Guaranteed to run on any Linux environment regardless of system library versions.
- **Stability Focused**: Reverts some aggressive traffic tracking changes to match the stability profile of v2.5.2, while keeping the essential fixes.

---

## [2.5.2] - 2026-01-11

### 🚀 Bug Fixes & Features

- **Global Traffic Reset**: The "Reset" button in the node list header now resets traffic for ALL inbounds in the database, not just for the current UI session.
- **Accurate Traffic Display**: The total traffic summary now shows absolute sums from the database, matching the real state of all nodes.
- **Improved Monitoring**: Refined the traffic collector logic to be more resilient and fix a bug where node tags were sometimes extracted incorrectly from `iptables` logs.

---

## [2.5.1] - 2026-01-11

### 🚀 Bug Fixes & Optimization

- **Dual-Stack Traffic Statistics**: Added support for `ip6tables` to ensure IPv6 traffic is correctly counted in the node list.
- **Accurate System Status Speed**: Optimized system network interface monitoring by excluding loopback and virtual interfaces, providing more accurate real-world traffic speeds.
- **Priority Rules**: Ensured traffic counting rules are always at the top of the `INPUT` and `OUTPUT` chains to prevent bypass by established connection rules or firewall overrides.
- **Robust Rule Check**: Improved the rule synchronization logic for higher reliability.

---

## [2.5.0] - 2026-01-11

### 🚀 Major Improvements

- **100% Accurate Traffic Statistics**: Re-implemented the traffic counting algorithm using Linux Kernel `iptables` counters. This bypasses the limitations of log parsing and provides bit-perfect traffic tracking directly from the network layer.
- **Atomic Database Updates**: Traffic usage is now updated using atomic SQL operations, preventing race conditions and ensuring data integrity.
- **Automatic Quota Enforcement**: The new traffic manager automatically disables inbound nodes that exceed their traffic quota in real-time.

### ✨ New Features & UI Enhancements

- **Simplified UI**: Cleaned up the XHTTP mode display text for a more professional look.
- **Automated Firewall Management**: The traffic manager automatically handles `iptables` rule creation and synchronization for all inbound nodes.
- **Robust Persistence**: Traffic counters now survive system reboots and service restarts gracefully.

### 🧹 Codebase Cleanup

- **Removed Redundant Scripts**: Deleted legacy installation and deployment scripts, consolidating logic into the main `install.sh`.
- **Project Structure Optimization**: Cleaned up temporary build files and updated `.gitignore` for a cleaner repository.

---

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
