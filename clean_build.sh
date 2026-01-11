#!/bin/bash
set -e

echo "🧹 Cleaning previous builds and caches..."
rm -rf release
rm -rf web/dist
rm -rf web/node_modules/.vite # Clear Vite cache
touch backend/src/main.rs

echo "📦 Building Frontend (Forcing new build)..."
cd web
# npm install # Skip install to save time if node_modules exists
if [ ! -d "node_modules" ]; then
    npm install
fi
npm run build
cd ..

echo "🦀 Building Backend (Embedding new frontend)..."
cd backend
cargo build --release
cd ..

echo "📦 Packaging Release..."
RELEASE_DIR="release"
mkdir -p ${RELEASE_DIR}

mkdir -p ${RELEASE_DIR}/temp/bin
cp backend/target/release/x-ui-backend ${RELEASE_DIR}/temp/bin/
cp -r web/dist ${RELEASE_DIR}/temp/bin/

# Copy xray-lite assets if available
if [ -f "/home/biubiuboy/xray-lite/target/release/vless-server" ]; then
    cp /home/biubiuboy/xray-lite/target/release/vless-server ${RELEASE_DIR}/vless-server-linux-x86_64
fi
if [ -f "/home/biubiuboy/xray-lite/target/release/keygen" ]; then
    cp /home/biubiuboy/xray-lite/target/release/keygen ${RELEASE_DIR}/keygen-linux-x86_64
fi

cd ${RELEASE_DIR}/temp
tar -czf ../x-ui-linux-amd64.tar.gz bin/
cd ../..
rm -rf ${RELEASE_DIR}/temp

cd ${RELEASE_DIR}
sha256sum * > checksums.txt
cd ..

echo "✅ Done! Release ready in release/"
