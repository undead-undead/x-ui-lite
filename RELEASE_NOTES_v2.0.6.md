# v2.0.6 - Debugging & Diagnostics Update

## 🛠️ Changes

### 1. Enable Info Logs
**Change**: Default log level for xray-lite changed from `error` to `info`.
**Reason**: To allow debugging of connection issues (handshake failures, SNI mismatches) which are currently silent.
**Action**: After upgrading, check logs via `journalctl -u x-ui -f` or in `/usr/local/x-ui/logs/`.

## 🔍 Diagnostics Guide

If you still experience timeouts:

1. **Check Process**:
   ```bash
   ps aux | grep vless-server
   ```
   If not running, run manually to see error: `/usr/local/x-ui/bin/xray -c /usr/local/x-ui/data/xray.json`

2. **Check Logs**:
   With v2.0.6, you should see connection attempts.
   - **No logs?** -> Network/Firewall issue. Port is blocked.
   - **Logs appear but fail?** -> Protocol mismatch or invalid UUID.

3. **Validate UUID**:
   Ensure the Client ID in the panel is a valid 36-char UUID (e.g. `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`).
   **xray-lite enforces strict UUID format.**

## 🔄 Upgrade

```bash
bash <(curl -Ls https://raw.githubusercontent.com/undead-undead/x-ui-lite-v2/main/install.sh)
```
