#!/bin/bash

red="\033[0;31m"
green="\033[0;32m"
yellow="\033[0;33m"
plain="\033[0m"

# Check Root
[[ $EUID -ne 0 ]] && echo -e "\033[0;31mError: Must be root to run this script!\033[0m" && exit 1

# Language Selection
echo -e "1. 简体中文"
echo -e "2. English"
read -p "请选择语言 / Please select language [1-2] (default 1): " lang_choice < /dev/tty
[[ -z $lang_choice ]] && lang_choice="1"

if [[ $lang_choice == "2" ]]; then
    LANG="en"
else
    LANG="zh"
fi

# Localization
declare -A msg
if [[ $LANG == "zh" ]]; then
    msg[err_root]="${red}错误: 必须使用 root 用户运行此脚本！${plain}"
    msg[detect_arch]="${green}正在检测系统架构...${plain}"
    msg[sys_arch]="${green}系统架构: %s${plain}"
    msg[err_arch]="${red}未检测到支持的系统架构: %s${plain}"
    msg[install_deps]="${yellow}正在安装依赖...${plain}"
    msg[open_port]="${yellow}正在尝试放行端口 %s...${plain}"
    msg[port_opened]="${green}%s 端口 %s 已放行${plain}"
    msg[bbr_check]="${yellow}正在检测 BBR 状态...${plain}"
    msg[bbr_on]="${green}BBR 已开启！${plain}"
    msg[bbr_enabling]="${yellow}正在尝试开启 BBR...${plain}"
    msg[bbr_fail]="${red}BBR 开启失败，建议升级内核。${plain}"
    msg[xray_exists]="${green}检测到 Xray-lite 已存在，跳过安装${plain}"
    msg[xray_installing]="${green}正在安装 Xray-lite...${plain}"
    msg[xray_fail]="${red}Xray-lite 下载失败，请检查网络！${plain}"
    msg[xui_installing]="${green}开始安装 X-UI-Lite...${plain}"
    msg[xui_downloading]="${yellow}正在下载发布包: %s${plain}"
    msg[xui_fail]="${red}下载失败，请检查网络！${plain}"
    msg[input_port]="请输入面板端口 (默认: 8080): "
    msg[input_root]="请输入面板根路径 (默认: /): "
    msg[setting_admin]="${green}正在设置管理员账户...${plain}"
    msg[input_user]="请输入管理员用户名 (默认: admin): "
    msg[input_pass]="请输入管理员密码 (默认: admin): "
    msg[install_success]="${green}X-UI-Lite 安装成功！${plain}"
    msg[visit_url]="访问地址: %s"
    msg[manage_user]="管理用户: %s"
    msg[manage_pass]="管理密码: %s"
    msg[manage_menu]="管理菜单: %s"
    msg[firewall_warn]="${yellow}如果是云服务器，请务必放行 %s 端口${plain}"
else
    msg[err_root]="${red}Error: Must be root to run this script!${plain}"
    msg[detect_arch]="${green}Detecting system architecture...${plain}"
    msg[sys_arch]="${green}System Arch: %s${plain}"
    msg[err_arch]="${red}Unsupported architecture: %s${plain}"
    msg[install_deps]="${yellow}Installing dependencies...${plain}"
    msg[open_port]="${yellow}Attempting to open port %s...${plain}"
    msg[port_opened]="${green}%s port %s opened${plain}"
    msg[bbr_check]="${yellow}Checking BBR status...${plain}"
    msg[bbr_on]="${green}BBR is already enabled!${plain}"
    msg[bbr_enabling]="${yellow}Attempting to enable BBR...${plain}"
    msg[bbr_fail]="${red}BBR failed to enable, please upgrade kernel.${plain}"
    msg[xray_exists]="${green}Xray-lite detected, skipping installation${plain}"
    msg[xray_installing]="${green}Installing Xray-lite...${plain}"
    msg[xray_fail]="${red}Xray-lite download failed, check network!${plain}"
    msg[xui_installing]="${green}Starting X-UI-Lite installation...${plain}"
    msg[xui_downloading]="${yellow}Downloading package: %s${plain}"
    msg[xui_fail]="${red}Download failed, check network!${plain}"
    msg[input_port]="Please input panel port (default: 8080): "
    msg[input_root]="Please input panel web root (default: /): "
    msg[setting_admin]="${green}Setting admin account...${plain}"
    msg[input_user]="Please input admin username (default: admin): "
    msg[input_pass]="Please input admin password (default: admin): "
    msg[install_success]="${green}X-UI-Lite installed successfully!${plain}"
    msg[visit_url]="URL: %s"
    msg[manage_user]="Username: %s"
    msg[manage_pass]="Password: %s"
    msg[manage_menu]="Menu: %s"
    msg[firewall_warn]="${yellow}Make sure to open port %s in firewall${plain}"
fi

# Helper function
i18n() {
    local key=$1
    shift
    local text="${msg[$key]}"
    echo -e "$(printf "$text" "$@")"
}

# Detect Arch
arch=$(arch)
if [[ $arch == "x86_64" || $arch == "x64" || $arch == "amd64" ]]; then
    arch="amd64"
elif [[ $arch == "aarch64" || $arch == "arm64" ]]; then
    arch="arm64"
else
    i18n "err_arch" "$arch"
    exit 1
fi

i18n "sys_arch" "$arch"

# Global Config
INSTALL_PATH="/usr/local/x-ui"
BIN_PATH="$INSTALL_PATH/bin/x-ui-backend"
XRAY_BIN_PATH="$INSTALL_PATH/bin/xray"
ENV_FILE="$INSTALL_PATH/.env"
SERVICE_FILE="/etc/systemd/system/x-ui.service"

RELEASE_URL="https://github.com/undead-undead/x-ui-lite/releases/download/v2.5.18/x-ui-linux-${arch}.tar.gz"

install_dependencies() {
    i18n "install_deps"
    if [[ -f /usr/bin/apt ]]; then
        apt-get update -y -qq >/dev/null 2>&1
        apt-get install -y -qq curl wget tar unzip >/dev/null 2>&1
    elif [[ -f /usr/bin/yum ]]; then
        yum install -y -q curl wget tar unzip >/dev/null 2>&1
    fi
}

open_port() {
    local port=$1
    i18n "open_port" "$port"
    
    # Check UFW
    if command -v ufw >/dev/null 2>&1; then
        if ufw status | grep -q "Status: active"; then
            ufw allow ${port}/tcp >/dev/null 2>&1
            ufw allow ${port}/udp >/dev/null 2>&1
            i18n "port_opened" "UFW" "$port"
        fi
    fi
    
    # Check firewalld
    if command -v firewall-cmd >/dev/null 2>&1; then
        if systemctl is-active --quiet firewalld; then
            firewall-cmd --permanent --add-port=${port}/tcp >/dev/null 2>&1
            firewall-cmd --permanent --add-port=${port}/udp >/dev/null 2>&1
            firewall-cmd --reload >/dev/null 2>&1
            i18n "port_opened" "Firewalld" "$port"
        fi
    fi
    
    # Check iptables
    if command -v iptables >/dev/null 2>&1; then
        iptables -I INPUT -p tcp --dport ${port} -j ACCEPT >/dev/null 2>&1
        iptables -I INPUT -p udp --dport ${port} -j ACCEPT >/dev/null 2>&1
        i18n "port_opened" "Iptables" "$port"
    fi
}

enable_bbr() {
    i18n "bbr_check"
    if sysctl net.ipv4.tcp_congestion_control | grep -q bbr; then
        i18n "bbr_on"
        return
    fi

    i18n "bbr_enabling"
    echo "net.core.default_qdisc=fq" >> /etc/sysctl.conf
    echo "net.ipv4.tcp_congestion_control=bbr" >> /etc/sysctl.conf
    sysctl -p >/dev/null 2>&1
    if sysctl net.ipv4.tcp_congestion_control | grep -q bbr; then
        i18n "bbr_on"
    else
        i18n "bbr_fail"
    fi
}

install_xray() {
    # Force update: remove existing binary if it exists
    if [[ -f "$XRAY_BIN_PATH" ]]; then
        rm -f "$XRAY_BIN_PATH"
    fi

    i18n "xray_installing"
    
    # Download xray-lite from x-ui-lite release
    local xray_lite_arch=""
    if [[ $arch == "amd64" ]]; then
        xray_lite_arch="x86_64"
    elif [[ $arch == "arm64" ]]; then
        xray_lite_arch="aarch64"
    fi
    
    local xray_lite_file="vless-server-linux-${xray_lite_arch}"
    # Download from x-ui-lite release instead of xray-lite repo
    local xray_lite_url="https://github.com/undead-undead/xray-lite/releases/download/v0.3.6/${xray_lite_file}"
    
    # Try downloading xray-lite
    wget -N --no-check-certificate -q -O /tmp/vless-server $xray_lite_url
    if [[ $? -ne 0 ]]; then
        echo -e "${red}Failed to download xray-lite binary${plain}"
        echo -e "${yellow}xray-lite binary not available for architecture: ${xray_lite_arch}${plain}"
        i18n "xray_fail"
        return 1
    fi
    
    # Install the vless-server binary
    mv /tmp/vless-server $XRAY_BIN_PATH
    chmod +x $XRAY_BIN_PATH
    
    # Keygen binary is no longer needed as key generation is now native in backend
    # but we install it anyway just in case user wants to use cli
    # echo -e "${green}Skipping keygen tool installation (native support active)...${plain}"
}

install_x_ui() {
    i18n "xui_installing"
    install_dependencies

    # Stop service if running
    systemctl stop x-ui 2>/dev/null

    # Create dirs
    mkdir -p $INSTALL_PATH/bin
    mkdir -p $INSTALL_PATH/data
    mkdir -p $INSTALL_PATH/logs

    # Download X-UI
    i18n "xui_downloading" "$RELEASE_URL"
    
    # Work in /tmp directory
    cd /tmp || { echo -e "${red}Failed to cd to /tmp${plain}"; return 1; }
    rm -f x-ui-linux-${arch}.tar.gz
    
    # Use -L to follow redirects from GitHub
    if command -v wget &> /dev/null; then
        wget -L -q --no-check-certificate -O x-ui-linux-${arch}.tar.gz "$RELEASE_URL"
    elif command -v curl &> /dev/null; then
        curl -fsSL -o x-ui-linux-${arch}.tar.gz "$RELEASE_URL"
    else
        echo -e "${red}Error: Neither wget nor curl is available${plain}"
        return 1
    fi
    
    # Verify download
    if [[ ! -f x-ui-linux-${arch}.tar.gz || ! -s x-ui-linux-${arch}.tar.gz ]]; then
        echo -e "${red}Failed to download X-UI v2.0.0${plain}"
        echo -e "${red}Please check if the v2.0.0 release exists at:${plain}"
        echo -e "${red}https://github.com/undead-undead/x-ui-lite/releases/tag/v2.0.0${plain}"
        i18n "xui_fail"
        return 1
    fi

    # Clean old dist folders to ensure new version takes effect
    rm -rf $INSTALL_PATH/dist
    rm -rf $INSTALL_PATH/bin/dist
    
    # Extract
    tar -zxf x-ui-linux-${arch}.tar.gz -C $INSTALL_PATH
    chmod +x $BIN_PATH
    
    # Install Xray
    install_xray

    # Init .env with default user inputs/updates
    
    local default_port="8080"
    local default_web_root="/"
    
    # If .env exists, load current values as defaults
    if [[ -f $ENV_FILE ]]; then
        local current_port=$(grep "SERVER_PORT" $ENV_FILE | cut -d '=' -f2)
        local current_root=$(grep "WEB_ROOT" $ENV_FILE | cut -d '=' -f2)
        [[ ! -z $current_port ]] && default_port=$current_port
        [[ ! -z $current_root ]] && default_web_root=$current_root
    fi

    # Input Port with validation
    while true; do
        printf "$(i18n "input_port")"
        read -p "($default_port): " port < /dev/tty
        [[ -z $port ]] && port=$default_port
        
        # Validate port range
        if [[ $port =~ ^[0-9]+$ ]] && [ $port -ge 1024 ] && [ $port -le 65535 ]; then
            break
        else
            echo -e "${red}端口必须在 1024-65535 之间 / Port must be between 1024-65535${plain}"
        fi
    done
    open_port $port
    
    # Input Web Root
    printf "$(i18n "input_root")"
    read -p "($default_web_root): " web_root < /dev/tty
    [[ -z $web_root ]] && web_root=$default_web_root
    
    # Normalize Web Root
    [[ ! $web_root =~ ^/ ]] && web_root="/${web_root}"
    [[ ! $web_root =~ /$ ]] && web_root="${web_root}/"
    web_root=$(echo "$web_root" | sed 's|//*|/|g')

    echo -e "${green}Web Root configured as: $web_root${plain}"
    
    # Generate/Update .env
    if [[ ! -f $ENV_FILE ]]; then
        # New file
        local jwt_secret=$(cat /proc/sys/kernel/random/uuid)
        cat > $ENV_FILE <<EOF
DATABASE_URL=sqlite://$INSTALL_PATH/data/x-ui.db
JWT_SECRET=$jwt_secret
JWT_EXPIRATION_HOURS=24
SERVER_HOST=0.0.0.0
SERVER_PORT=$port
XRAY_BIN_PATH=$XRAY_BIN_PATH
XRAY_CONFIG_PATH=$INSTALL_PATH/data/xray.json
WEB_ROOT=$web_root
WEB_DIST_PATH=$INSTALL_PATH/bin/dist
RUST_LOG=info
EOF
    else
        # Update existing
        update_env "SERVER_PORT" "$port"
        update_env "WEB_ROOT" "$web_root"
        update_env "XRAY_BIN_PATH" "$XRAY_BIN_PATH"
        update_env "WEB_DIST_PATH" "$INSTALL_PATH/bin/dist"
    fi

    # Create Service
    cat > $SERVICE_FILE <<EOF
[Unit]
Description=X-UI-Lite Service
After=network.target

[Service]
Type=simple
WorkingDirectory=$INSTALL_PATH
ExecStart=$BIN_PATH
Restart=always
User=root
EnvironmentFile=$INSTALL_PATH/.env

[Install]
WantedBy=multi-user.target
EOF

    systemctl daemon-reload
    
    # Auto Enable BBR
    enable_bbr

    # 确保数据目录和数据库文件存在
    mkdir -p $INSTALL_PATH/data
    touch $INSTALL_PATH/data/x-ui.db
    chmod 755 $INSTALL_PATH/data
    chmod 644 $INSTALL_PATH/data/x-ui.db

    systemctl enable x-ui
    systemctl start x-ui
    
    # 等待服务启动并初始化数据库
    echo -e "${yellow}Waiting for service to initialize...${plain}"
    sleep 2
    
    # 设置初始账户密码
    i18n "setting_admin"
    read -p "$(i18n "input_user")" admin_user < /dev/tty
    [[ -z $admin_user ]] && admin_user="admin"
    
    # Password input with validation
    while true; do
        read -s -p "$(i18n "input_pass")" admin_pass < /dev/tty
        echo
        [[ -z $admin_pass ]] && admin_pass="admin"
        
        # Validate password length
        if [ ${#admin_pass} -ge 4 ]; then
            break
        else
            echo -e "${red}密码至少需要 4 位 / Password must be at least 4 characters${plain}"
        fi
    done
    
    # 设置初始账户密码前先停止服务，避免数据库锁竞争
    systemctl stop x-ui 
    
    # 使用后端命令行工具设置账户
    cd $INSTALL_PATH
    echo -e "${yellow}Setting up admin account...${plain}"
    $BIN_PATH -u "$admin_user" -p "$admin_pass"
    if [[ $? -ne 0 ]]; then
        echo -e "${red}Failed to set account! Using default: admin/admin${plain}"
        admin_user="admin"
        admin_pass="admin"
    fi
    systemctl start x-ui
    
    # 获取公网IP
    public_ip=$(curl -s https://api.ipify.org || curl -s https://ifconfig.me/ip || echo "YOUR_IP")
    
    # 从 .env 读取端口和根路径
    current_port=$(grep "SERVER_PORT" $ENV_FILE | cut -d '=' -f2)
    [[ -z $current_port ]] && current_port="8080"
    
    current_web_root=$(grep "WEB_ROOT" $ENV_FILE | cut -d '=' -f2)
    [[ -z $current_web_root ]] && current_web_root="/"

    echo -e ""
    i18n "install_success"
    echo -e "${green}----------------------------------------------${plain}"
    i18n "visit_url" "${yellow}http://${public_ip}:${current_port}${current_web_root}${plain}"
    i18n "manage_user" "${yellow}${admin_user}${plain}"
    i18n "manage_pass" "${yellow}${admin_pass}${plain}"
    i18n "manage_menu" "${yellow}x-ui-lite${plain}"
    echo -e "${green}----------------------------------------------${plain}"
    i18n "firewall_warn" "${current_port}"

    cat > /usr/bin/x-ui-lite <<EOF
#!/bin/bash
red="\033[0;31m"
green="\033[0;32m"
yellow="\033[0;33m"
plain="\033[0m"

INSTALL_PATH="/usr/local/x-ui"
ENV_FILE="\$INSTALL_PATH/.env"
BIN_PATH="\$INSTALL_PATH/bin/x-ui-backend"

# Language settings
LANG_FILE="\$INSTALL_PATH/.lang"
if [[ ! -f "\$LANG_FILE" ]]; then
    echo "$LANG" > "\$LANG_FILE"
fi
CUR_LANG=\$(cat "\$LANG_FILE")

# Localization
declare -A msg
if [[ "\$CUR_LANG" == "zh" ]]; then
    msg[menu_title]="X-UI-Lite 管理脚本"
    msg[menu_0]="退出脚本"
    msg[menu_1]="启动 X-UI-Lite"
    msg[menu_2]="停止 X-UI-Lite"
    msg[menu_3]="重启 X-UI-Lite"
    msg[menu_4]="查看状态"
    msg[menu_5]="修改面板端口"
    msg[menu_6]="修改面板根路径"
    msg[menu_7]="修改账户密码"
    msg[menu_8]="开启 BBR 加速"
    msg[menu_9]="查看运行日志"
    msg[menu_10]="卸载 X-UI-Lite"
    msg[menu_11]="切换语言 / Switch Language"
    msg[input_choice]="请输入选择 [0-11]: "
    msg[err_choice]="请输入正确的数字 [0-11]"
    msg[lang_switched]="语言已切换为: 简体中文"
    msg[started]="X-UI-Lite 已启动"
    msg[stopped]="X-UI-Lite 已停止"
    msg[restarted]="X-UI-Lite 已重启"
    msg[input_port]="请输入新端口: "
    msg[err_port_empty]="端口不能为空"
    msg[port_changed]="端口已修改为: %s"
    msg[input_root]="请输入面板根路径 (例如 /panel/，默认为 /): "
    msg[root_changed]="根路径已修改为: %s"
    msg[input_user]="请输入新用户名: "
    msg[input_pass]="请输入新密码: "
    msg[err_user_pass_empty]="用户名或密码不能为空"
    msg[account_changed]="账户信息已更新"
    msg[confirm_uninstall]="确定要卸载 X-UI-Lite 吗？[y/N]: "
    msg[uninstalled]="X-UI-Lite 已卸载"
    msg[cancel_uninstall]="已取消卸载"
    msg[bbr_on]="BBR 已开启"
else
    msg[menu_title]="X-UI-Lite Management Script"
    msg[menu_0]="Exit"
    msg[menu_1]="Start X-UI-Lite"
    msg[menu_2]="Stop X-UI-Lite"
    msg[menu_3]="Restart X-UI-Lite"
    msg[menu_4]="Check Status"
    msg[menu_5]="Change Panel Port"
    msg[menu_6]="Change Web Root"
    msg[menu_7]="Change Credentials"
    msg[menu_8]="Enable BBR"
    msg[menu_9]="Check Logs"
    msg[menu_10]="Uninstall X-UI-Lite"
    msg[menu_11]="Switch Language / 切换语言"
    msg[input_choice]="Please enter selection [0-11]: "
    msg[err_choice]="Please enter a valid number [0-11]"
    msg[lang_switched]="Language switched to: English"
    msg[started]="X-UI-Lite started"
    msg[stopped]="X-UI-Lite stopped"
    msg[restarted]="X-UI-Lite restarted"
    msg[input_port]="Enter new port: "
    msg[err_port_empty]="Port cannot be empty"
    msg[port_changed]="Port changed to: %s"
    msg[input_root]="Enter web root (e.g. /panel/, default /): "
    msg[root_changed]="Web root changed to: %s"
    msg[input_user]="Enter new username: "
    msg[input_pass]="Enter new password: "
    msg[err_user_pass_empty]="Username or password cannot be empty"
    msg[account_changed]="Credentials updated"
    msg[confirm_uninstall]="Are you sure to uninstall X-UI-Lite? [y/N]: "
    msg[uninstalled]="X-UI-Lite uninstalled"
    msg[cancel_uninstall]="Uninstall cancelled"
    msg[bbr_on]="BBR enabled"
fi

i18n() {
    local key=\$1
    shift
    local text="\${msg[\$key]}"
    echo -e "\$(printf "\$text" "\$@")"
}

check_root() {
    [[ \$EUID -ne 0 ]] && echo -e "\${red}Error: Must be root!\${plain}" && exit 1
}

open_port() {
    local port=\$1
    # Check UFW
    if command -v ufw >/dev/null 2>&1; then
        if ufw status | grep -q "Status: active"; then
            ufw allow \${port}/tcp >/dev/null 2>&1
            ufw allow \${port}/udp >/dev/null 2>&1
        fi
    fi
    # Check firewalld
    if command -v firewall-cmd >/dev/null 2>&1; then
        if systemctl is-active --quiet firewalld; then
            firewall-cmd --permanent --add-port=\${port}/tcp >/dev/null 2>&1
            firewall-cmd --permanent --add-port=\${port}/udp >/dev/null 2>&1
            firewall-cmd --reload >/dev/null 2>&1
        fi
    fi
}

update_env() {
    local key=\$1
    local val=\$2
    if grep -q "^\${key}=" \$ENV_FILE; then
        sed -i "s|^\${key}=.*|\${key}=\${val}|" \$ENV_FILE
    else
        echo "\${key}=\${val}" >> \$ENV_FILE
    fi
}

start() { systemctl start x-ui && i18n "started"; }
stop() { systemctl stop x-ui && i18n "stopped"; }
restart() { systemctl restart x-ui && i18n "restarted"; }
status() { systemctl status x-ui; }

set_port() {
    read -p "\$(i18n "input_port")" port
    [[ -z \$port ]] && i18n "err_port_empty" && return
    open_port \$port
    update_env "SERVER_PORT" "\$port"
    restart
    i18n "port_changed" "\$port"
}

set_web_root() {
    read -p "\$(i18n "input_root")" path
    [[ -z \$path ]] && path="/"
    [[ ! \$path =~ ^/ ]] && path="/\${path}"
    [[ ! \$path =~ /\$ ]] && path="\${path}/"
    path=\$(echo "\$path" | sed 's|//*|/|g')
    update_env "WEB_ROOT" "\$path"
    restart
    i18n "root_changed" "\$path"
}

set_account() {
    read -p "\$(i18n "input_user")" username
    read -p "\$(i18n "input_pass")" password
    [[ -z \$username || -z \$password ]] && i18n "err_user_pass_empty" && return
    cd \$INSTALL_PATH
    \$BIN_PATH -u "\$username" -p "\$password"
    i18n "account_changed"
}

uninstall() {
    read -p "\$(i18n "confirm_uninstall")" confirm
    if [[ \$confirm == "y" || \$confirm == "Y" ]]; then
        systemctl stop x-ui
        systemctl disable x-ui
        rm -f /etc/systemd/system/x-ui.service
        rm -f /usr/bin/x-ui-lite
        rm -rf \$INSTALL_PATH
        systemctl daemon-reload
        i18n "uninstalled"
    else
        i18n "cancel_uninstall"
    fi
}

enable_bbr() {
    if sysctl net.ipv4.tcp_congestion_control | grep -q bbr; then
        i18n "bbr_on"
    else
        echo "net.core.default_qdisc=fq" >> /etc/sysctl.conf
        echo "net.ipv4.tcp_congestion_control=bbr" >> /etc/sysctl.conf
        sysctl -p
        i18n "bbr_on"
    fi
}

switch_lang() {
    if [[ "\$CUR_LANG" == "zh" ]]; then
        echo "en" > "\$LANG_FILE"
    else
        echo "zh" > "\$LANG_FILE"
    fi
    exec "\$0"
}

show_menu() {
    echo -e "
  \${green}\$(i18n "menu_title")\${plain}
  \${green}0.\${plain} \$(i18n "menu_0")
  \${green}1.\${plain} \$(i18n "menu_1")
  \${green}2.\${plain} \$(i18n "menu_2")
  \${green}3.\${plain} \$(i18n "menu_3")
  \${green}4.\${plain} \$(i18n "menu_4")
  \${green}5.\${plain} \$(i18n "menu_5")
  \${green}6.\${plain} \$(i18n "menu_6")
  \${green}7.\${plain} \$(i18n "menu_7")
  \${green}8.\${plain} \$(i18n "menu_8")
  \${green}9.\${plain} \$(i18n "menu_9")
  \${green}10.\${plain} \$(i18n "menu_10")
  \${green}11.\${plain} \$(i18n "menu_11")
 "
    read -p "\$(i18n "input_choice")" num
    case \$num in
        0) exit 0 ;;
        1) start ;;
        2) stop ;;
        3) restart ;;
        4) status ;;
        5) set_port ;;
        6) set_web_root ;;
        7) set_account ;;
        8) enable_bbr ;;
        9) journalctl -u x-ui -f ;;
        10) uninstall ;;
        11) switch_lang ;;
        *) i18n "err_choice" ;;
    esac
}

check_root
if [[ \$# > 0 ]]; then
    case \$1 in
        start) start ;;
        stop) stop ;;
        restart) restart ;;
        status) status ;;
        install_bbr) enable_bbr ;;
        uninstall) uninstall ;;
        *) show_menu ;;
    esac
else
    show_menu
fi
EOF
    
    chmod +x /usr/bin/x-ui-lite
    ln -sf /usr/bin/x-ui-lite /usr/bin/x-ui
}

# Install Entry
install_x_ui
