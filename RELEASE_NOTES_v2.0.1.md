# v2.0.1 - Reality Key Generation Fix

## 🐛 Bug Fixes

### Fixed Reality Key Generation

**Issue**: Frontend "Generate Keys" button failed with error "Failed to generate keys, please try again"

**Root Cause**: Backend was calling `xray x25519` command, but xray-lite binary doesn't have this subcommand.

**Solution**: 
- Modified backend to use xray-lite's dedicated `keygen` binary
- Added keygen tool to installation script
- Maintained backward compatibility with xray-core's `x25519` command

## 📦 Changes

### Backend (v2.0.0 → v2.0.1)

**Modified Files**:
- `backend/src/handlers/xray.rs` - Use keygen binary instead of `xray x25519`
- `backend/Cargo.toml` - Updated version to 2.0.0
- `install.sh` - Download and install keygen tool

**Logic Flow**:
```rust
1. Check if keygen binary exists (/usr/local/x-ui/bin/keygen)
2. If yes: Use keygen (xray-lite style)
3. If no: Fallback to `xray x25519` (xray-core compatibility)
```

### Installation Script

**New Step**:
```bash
# Download keygen tool
wget https://github.com/undead-undead/x-ui-lite-v2/releases/download/v2.0.1/keygen-linux-x86_64
```

## 🎯 Testing

### Verify the Fix

1. **Install/Upgrade**:
   ```bash
   bash <(curl -Ls https://raw.githubusercontent.com/undead-undead/x-ui-lite-v2/main/install.sh)
   ```

2. **Check keygen**:
   ```bash
   ls -lh /usr/local/x-ui/bin/keygen
   /usr/local/x-ui/bin/keygen  # Should output key pair
   ```

3. **Test in Panel**:
   - Login to panel
   - Go to Inbound page
   - Click "Add Inbound"
   - Click "Generate Keys" button
   - Should see private_key and public_key populated ✅

## 📊 Release Files

Same as v2.0.0, with updated backend:

- `x-ui-linux-amd64.tar.gz` - Backend v2.0.0 (with keygen fix)
- `vless-server-linux-x86_64` - xray-lite core
- `keygen-linux-x86_64` - Reality key generation tool ✨
- `checksums.txt` - SHA256 checksums

## 🔄 Upgrade from v2.0.0

If you installed v2.0.0:

```bash
# Re-run installation script
bash <(curl -Ls https://raw.githubusercontent.com/undead-undead/x-ui-lite-v2/main/install.sh)
```

The script will:
1. Download updated backend with keygen support
2. Download keygen tool
3. Restart service

## ✅ Verification

After upgrade, check:

```bash
# Backend should recognize keygen
sudo journalctl -u x-ui -n 20

# Test key generation in panel
# Click "Generate Keys" - should work now!
```

---

**Full Changelog**: https://github.com/undead-undead/x-ui-lite-v2/compare/v2.0.0...v2.0.1
