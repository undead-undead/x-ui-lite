#!/bin/bash
# Quick fix: Direct install v2.0.0 with fresh script

echo "Downloading latest install script..."
curl -Ls https://raw.githubusercontent.com/undead-undead/x-ui-lite-v2/main/install.sh -o /tmp/x-ui-install-v2.sh

echo "Running installation..."
bash /tmp/x-ui-install-v2.sh

rm -f /tmp/x-ui-install-v2.sh
