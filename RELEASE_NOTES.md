# X-UI-Lite v2.5.2 - Global Traffic Reset & Accuracy

This release focuses on functional correctness for traffic management and global statistics.

### 🌟 Key Improvements

#### 🔄 Global Traffic Reset
The "Reset" button in the **Inbound List Header** now performs a **Real Database Reset** for all nodes. This allows you to clear all traffic metrics across your entire panel with a single click.

#### 📊 Absolute Sum Statistics
The total upload and download figures in the header now show the **exact sum** of all nodes from the database. We have removed the session-based "baseline" logic to provide a more transparent and intuitive view of your total data usage.

#### 🛡️ Resilient Collector
Found and fixed a subtle bug in the traffic parsing logic that could sometimes fail to extract node tags correctly. This ensures more reliable "Upload/Download" metrics in the node list.

### 🛠 Installation & Upgrade

```bash
bash <(curl -Ls https://raw.githubusercontent.com/undead-undead/x-ui-lite/main/install.sh)
```

### ⚠️ Note for Users
The new traffic tracking requires `iptables` support in the kernel. Most Linux distributions (Ubuntu, Debian, CentOS) support this out of the box.

---

**Full Changelog**: [CHANGELOG.md](https://github.com/undead-undead/x-ui-lite/blob/v2.5.2/CHANGELOG.md)
