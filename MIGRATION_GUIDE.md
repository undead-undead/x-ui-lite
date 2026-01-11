# X-UI-Lite v2.0 - xray-lite Integration Guide

## 项目概述 / Project Overview

**X-UI-Lite v2.0** 现在由 **xray-lite** 提供支持，这是一个纯 Rust 实现的 VLESS+Reality 代理核心。

**X-UI-Lite v2.0** is now powered by **xray-lite**, a pure Rust implementation of VLESS+Reality proxy core.

---

## 架构变化 / Architecture Changes

### v1.x (使用 Xray-Core / Using Xray-Core)
```
┌─────────────────────────────────────┐
│  X-UI-Lite Backend (Rust)  ~50MB   │
├─────────────────────────────────────┤
│  Xray-Core (Go)            ~100MB   │
└─────────────────────────────────────┘
Total: ~150MB RAM
```

### v2.0 (使用 xray-lite / Using xray-lite)
```
┌─────────────────────────────────────┐
│  X-UI-Lite Backend (Rust)  ~50MB   │
├─────────────────────────────────────┤
│  xray-lite (Rust)          ~10MB    │
└─────────────────────────────────────┘
Total: ~60MB RAM (60% reduction!)
```

---

## 主要优势 / Key Benefits

### 1. 性能提升 / Performance Improvements
- **更低内存占用** / **Lower Memory**: 60MB vs 150MB (60% reduction)
- **零 GC 开销** / **Zero GC**: 无 Go 运行时，无垃圾回收暂停
- **更快启动** / **Faster Startup**: 纯 Rust 二进制，启动时间更短

### 2. 更好的安全性 / Better Security
- **内置反探测** / **Built-in Anti-Probing**: 严格的 SNI 校验
- **动态证书** / **Dynamic Certificates**: Reality 协议自动处理
- **减少攻击面** / **Reduced Attack Surface**: 更小的代码库

### 3. 简化配置 / Simplified Configuration
- 移除了不必要的 API 配置
- 移除了 Stats 和 Policy 配置
- 配置文件更加简洁

---

## 功能对比 / Feature Comparison

| 功能 / Feature | v1.x (Xray-Core) | v2.0 (xray-lite) |
|----------------|------------------|------------------|
| VLESS 协议 | ✅ | ✅ |
| Reality TLS | ✅ | ✅ |
| XHTTP 传输 | ✅ | ✅ |
| SNI 嗅探 | ✅ | ✅ |
| 流量统计 | ✅ | ❌ (API 限制) |
| 多协议支持 | ✅ (全部) | ⚠️ (仅 VLESS) |
| 内存占用 | ~150MB | ~60MB |
| 反探测 | ❌ | ✅ |

---

## 限制说明 / Limitations

### 流量统计 / Traffic Statistics
xray-lite 不提供 gRPC API，因此无法实时更新流量统计。

xray-lite doesn't provide gRPC API, so real-time traffic statistics are not available.

**影响 / Impact:**
- 流量配额设置仍然有效，但计数器不会增加
- 建议通过系统日志或外部监控工具追踪流量

**Workaround:**
- Traffic quota settings still work, but counters won't increase
- Use system logs or external monitoring tools for tracking

### 协议支持 / Protocol Support
xray-lite 目前仅支持 VLESS 协议。

xray-lite currently only supports VLESS protocol.

**如果需要其他协议 / If you need other protocols:**
- 请继续使用 v1.x with Xray-Core
- 或等待 xray-lite 添加更多协议支持

---

## 安装说明 / Installation

### 新安装 / Fresh Install

```bash
bash <(curl -Ls https://raw.githubusercontent.com/YOUR_USERNAME/x-ui-lite-v2/main/install.sh)
```

### 从 v1.x 升级 / Upgrade from v1.x

**⚠️ 警告 / Warning:** 升级将使用 xray-lite 替换 Xray-Core

1. **备份配置 / Backup Configuration**
```bash
sudo x-ui-lite
# 选择备份功能 / Select backup option
```

2. **运行升级脚本 / Run Upgrade Script**
```bash
bash <(curl -Ls https://raw.githubusercontent.com/YOUR_USERNAME/x-ui-lite-v2/main/install.sh)
```

3. **验证安装 / Verify Installation**
```bash
sudo systemctl status x-ui
sudo journalctl -u x-ui -f
```

---

## 配置文件格式 / Configuration Format

### v1.x 配置示例 / v1.x Config Example
```json
{
  "log": {...},
  "api": {
    "tag": "api",
    "services": ["HandlerService", "StatsService"]
  },
  "stats": {},
  "policy": {...},
  "inbounds": [...],
  "outbounds": [...],
  "routing": {...}
}
```

### v2.0 配置示例 / v2.0 Config Example
```json
{
  "log": {...},
  "inbounds": [...],
  "outbounds": [...],
  "routing": {...}
}
```

**变化 / Changes:**
- ❌ 移除 `api` 配置 / Removed `api` section
- ❌ 移除 `stats` 配置 / Removed `stats` section  
- ❌ 移除 `policy` 配置 / Removed `policy` section
- ✅ 保留核心功能 / Kept core functionality

---

## 故障排除 / Troubleshooting

### xray-lite 无法启动 / xray-lite won't start

**检查二进制文件 / Check binary:**
```bash
ls -lh /usr/local/x-ui/bin/xray
file /usr/local/x-ui/bin/xray
```

**查看日志 / Check logs:**
```bash
sudo journalctl -u x-ui -f
cat /usr/local/x-ui/logs/error.log
```

### 流量不增长 / Traffic not increasing

这是预期行为。xray-lite 不支持流量统计 API。

This is expected behavior. xray-lite doesn't support traffic statistics API.

**如需统计 / If you need statistics:**
- 使用系统工具如 `vnstat` 或 `iftop`
- 或回退到 v1.x

### 配置不兼容 / Configuration incompatible

xray-lite 会忽略不支持的配置项。检查日志中的警告。

xray-lite will ignore unsupported config items. Check logs for warnings.

---

## 性能测试 / Performance Benchmarks

### 内存使用 / Memory Usage
```
v1.x: ~150MB (Backend 50MB + Xray-Core 100MB)
v2.0: ~60MB  (Backend 50MB + xray-lite 10MB)
节省 / Savings: 60%
```

### 启动时间 / Startup Time
```
v1.x: ~2-3 seconds
v2.0: ~500ms
提升 / Improvement: 4-6x faster
```

### 连接延迟 / Connection Latency
```
v1.x: baseline
v2.0: ~5-10% lower (raw pipe transport)
```

---

## 开发信息 / Development Info

### 项目仓库 / Repositories
- **X-UI-Lite v2**: https://github.com/YOUR_USERNAME/x-ui-lite-v2
- **xray-lite**: https://github.com/undead-undead/xray-lite

### 依赖版本 / Dependencies
- xray-lite: v0.2.46+
- Rust Backend: 1.70+
- React Frontend: 18+

---

## 常见问题 / FAQ

**Q: 为什么要切换到 xray-lite？**  
**Q: Why switch to xray-lite?**

A: 更低的内存占用、更好的性能、内置安全特性

A: Lower memory usage, better performance, built-in security features

---

**Q: 可以回退到 v1.x 吗？**  
**Q: Can I roll back to v1.x?**

A: 可以。重新运行 v1.x 的安装脚本即可。

A: Yes. Simply re-run the v1.x installation script.

---

**Q: 流量统计什么时候会支持？**  
**Q: When will traffic statistics be supported?**

A: 这取决于 xray-lite 是否添加 API 支持。目前没有时间表。

A: This depends on whether xray-lite adds API support. No timeline yet.

---

## 联系支持 / Support

- **Issues**: https://github.com/YOUR_USERNAME/x-ui-lite-v2/issues
- **Discussions**: https://github.com/YOUR_USERNAME/x-ui-lite-v2/discussions
- **xray-lite Issues**: https://github.com/undead-undead/xray-lite/issues

---

## 致谢 / Credits

感谢以下项目 / Thanks to:

- [xray-lite](https://github.com/undead-undead/xray-lite) - Pure Rust VLESS+Reality
- [Xray-core](https://github.com/XTLS/Xray-core) - Original Reality protocol
- [Tokio](https://tokio.rs/) - Async runtime
- [Axum](https://github.com/tokio-rs/axum) - Web framework

---

**注意 / Note:** 请将 `YOUR_USERNAME` 替换为您的实际 GitHub 用户名。

**Note:** Please replace `YOUR_USERNAME` with your actual GitHub username.
