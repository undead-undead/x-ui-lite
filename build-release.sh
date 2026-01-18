#!/bin/bash
# Build and package script for x-ui-lite v2.0.0

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

VERSION="v2.9.5"
RELEASE_DIR="release"

echo -e "${GREEN}Building X-UI-Lite ${VERSION}...${NC}"

# Clean previous builds
rm -rf ${RELEASE_DIR}
mkdir -p ${RELEASE_DIR}

# Check if backend binary exists

if [[ ! -f backend/target/x86_64-unknown-linux-musl/release/x-ui-backend ]]; then
    echo -e "${RED}Error: Backend binary (musl) not found. Please run 'cd backend && cargo build --target x86_64-unknown-linux-musl --release' first${NC}"
    exit 1
fi

# Check if web dist exists
if [[ ! -d web/dist ]]; then
    echo -e "${YELLOW}Warning: web/dist not found. Building frontend...${NC}"
    cd web
    npm install
    npm run build
    cd ..
fi

# Package for amd64
echo -e "${GREEN}Packaging for amd64...${NC}"
mkdir -p ${RELEASE_DIR}/temp/bin
cp backend/target/x86_64-unknown-linux-musl/release/x-ui-backend ${RELEASE_DIR}/temp/bin/
cp -r web/dist ${RELEASE_DIR}/temp/bin/

cd ${RELEASE_DIR}/temp
tar -czf ../x-ui-linux-amd64.tar.gz bin/
cd ../..
rm -rf ${RELEASE_DIR}/temp

# Calculate checksums
echo -e "${GREEN}Calculating checksums...${NC}"
cd ${RELEASE_DIR}
sha256sum x-ui-linux-*.tar.gz > checksums.txt
cd ..

echo -e "${GREEN}Release packages created:${NC}"
ls -lh ${RELEASE_DIR}/

echo -e "${GREEN}Done!${NC}"
echo -e "${YELLOW}Next steps:${NC}"
echo "1. Review the packages in ${RELEASE_DIR}/"
echo "2. Run: gh release create ${VERSION} ${RELEASE_DIR}/* --title \"${VERSION} - Powered by xray-lite\" --notes-file RELEASE_NOTES.md"
