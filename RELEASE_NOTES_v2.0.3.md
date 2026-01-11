# v2.0.3 - Critical Reality Compatibility Fixes

## 🐛 Bug Fixes

### Fixed Reality Connection Timeout

**Issue**: Passwall and other clients could not connect to Reality nodes created in the panel (Timeout).

**Root Cause**: 
`xray-lite` has stricter configuration requirements than `xray-core`. The generated config had multiple incompatibilities:
1. `listen` field was missing (required by xray-lite).
2. `sniffing` was in the wrong location.
3. `serverName` (singular) vs `serverNames` (plural) mismatch in Reality settings.
4. `shortIds` sometimes missing or singular `shortId`.

**Solution**: 
- **Enforce Listen**: Always set `listen: "0.0.0.0"` if missing.
- **Correct Sniffing**: Move `sniffing` config into `settings`.
- **Auto-Fix Reality**: 
  - Convert `serverName` -> `serverNames` array.
  - Convert `shortId` -> `shortIds` array.
  - Ensure `shortIds` is never null.

## 📦 Changes

### Backend (v2.0.2 → v2.0.3)

**Modified Files**:
- `backend/src/services/xray_service.rs` - Comprehensive config patch logic
- `backend/Cargo.toml` - Version 2.0.3

## 🎯 Testing

After upgrading:

1. **Panel Restart**: Will auto-regenerate config.
2. **Verify Config**:
   ```bash
   cat /usr/local/x-ui/data/xray.json
   ```
   Look for `serverNames` (plural) inside `realitySettings`.
3. **Connect**: Test with Passwall/V2RayN.

## 🔄 Upgrade

```bash
bash <(curl -Ls https://raw.githubusercontent.com/undead-undead/x-ui-lite-v2/main/install.sh)
```
