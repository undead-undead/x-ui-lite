# v2.0.7 - Reality Config Cleanup

## 🐛 Bug Fix

**Issue**: `Fallback total` errors when connecting via Reality.
**Root Cause**: Empty strings (`""`) in `serverNames` or `shortIds` arrays (caused by empty fields in the panel) were causing `xray-lite`'s Reality validation logic to reject connections.
**Fix**: Backend now actively filters out empty strings from `serverNames` and `shortIds` arrays.

## 🎯 Verification

1. **Upgrade**:
   ```bash
   bash <(curl -Ls https://raw.githubusercontent.com/undead-undead/x-ui-lite-v2/main/install.sh)
   ```

2. **Verify ShortId**:
   If ShortId is empty in the panel, generated config will now have `shortIds: []` instead of `shortIds: [""]`.

## 🛠️ Diagnostics

If connection still fails with `Fallback total`, please ensure:
1. Client `Server Name` matches panel exactly (no spaces).
2. Client `Public Key` is correct.
3. Server time is synchronized.
