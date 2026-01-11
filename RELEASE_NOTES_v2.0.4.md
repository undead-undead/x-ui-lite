# v2.0.4 - Critical Crash Fix

## 🚨 Critical Fix

**Issue**: `xray-lite` failed to start (crashed immediately) because generated config was missing required fields.

**Symptoms**:
- `systemctl status x-ui` shows active, but `ps aux | grep vless-server` shows nothing.
- Clients timeout.
- Log error: `Failed to deserialize config: missing field clients` (if run manually).

**Root Cause**:
`xray-lite` requires `clients` field to exist in `settings`, even if empty. If you created a new Inbound but the settings json was minimal, `xray-lite` would fail to load the config.

**Solution**:
- Forcefully inject `"clients": []` into configuration if missing.
- Forcefully inject `"decryption": "none"` if missing.

## 📦 Changes

### Backend (v2.0.3 → v2.0.4)

**Modified Files**:
- `backend/src/services/xray_service.rs` - Added mandatory field injection.
- `backend/Cargo.toml` - Version 2.0.4

## 🎯 Testing

After upgrading to v2.0.4:

1. **Verify Process**:
   ```bash
   ps aux | grep vless-server
   ```
   **Must show a running process now!**

2. **Verify Config**:
   ```bash
   cat /usr/local/x-ui/data/xray.json
   ```
   Should see `"clients": []` (or with clients) inside `settings`.

## 🔄 Upgrade

```bash
bash <(curl -Ls https://raw.githubusercontent.com/undead-undead/x-ui-lite-v2/main/install.sh)
```
