# v2.0.8 - Frontend Validation & UUID Fix

## 🛡️ Improvements

**Frontend Validation**:
- Added strict **36-character UUID length check** in the "Add/Edit Inbound" modal.
- Prevents saving configurations with invalid/short UUIDs that would cause `xray-lite` to crash or refuse connections.

**Why this matters**:
`xray-lite` is strictly compliant with VLESS standards and requires valid UUIDs. This UI update prevents the "Fallback total" or "Crash loop" issues caused by malformed IDs.

## 🔄 Upgrade

```bash
bash <(curl -Ls https://raw.githubusercontent.com/undead-undead/x-ui-lite-v2/main/install.sh)
```
