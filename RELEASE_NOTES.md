# X-UI-Lite v2.5.0 - Accurate Traffic Statistics Release

This release introduces a major breakthrough in traffic tracking for the X-UI-Lite panel, alongside several UI refinements and codebase optimizations.

### 🌟 Key Highlights

#### 📊 100% Accurate Traffic Statistics
We have completely replaced the unreliable log-parsing method with a robust **Linux Kernel Iptables Counter** system. 
- **Bit-Perfect Accuracy**: Tracks traffic at the network layer for 100% precision.
- **Kernel-Level Tracking**: Works independently of Xray logs or APIs.
- **Atomic Updates**: Uses SQL atomic operations to ensure no traffic data is ever lost.
- **Automatic Sync**: `iptables` rules are automatically created and synchronized with your inbounds.

#### 🎨 UI & UX Refinements
- Simplified XHTTP mode descriptions for a cleaner, more professional interface.
- Consistently uses internationalization keys for all UI elements.

#### 🔧 System & Codebase
- **Removed Legacy Scripts**: Cleaned up the repository by removing outdated installation and test scripts.
- **Optimized `install.sh`**: Updated the installation script to support the new `v2.5.0` version and its dependencies.
- **Full Compatibility**: Seamlessly integrated with the latest `xray-lite` core.

### 🛠 Installation & Upgrade

To install or upgrade to v2.5.0, run the following command:

```bash
bash <(curl -Ls https://raw.githubusercontent.com/undead-undead/x-ui-lite/main/install.sh)
```

### ⚠️ Note for Users
The new traffic tracking requires `iptables` support in the kernel. Most Linux distributions (Ubuntu, Debian, CentOS) support this out of the box.

---

**Full Changelog**: [CHANGELOG.md](https://github.com/undead-undead/x-ui-lite/blob/v2.5.0/CHANGELOG.md)
