!#/usr/bin/sh
which wasm-bindgen python3 || { echo "python3 and wasm-bindgen is required"; exit 1; }
CONFIG=${1:-debug}
TARGET=wasm32-unknown-unknown
if [ "$CONFIG" != "debug" ]; then
  BUILD_CONFIG="--$CONFIG"
fi
WASM_OUTPUT=./target/$TARGET/$CONFIG/rubik.wasm
WEB_OUTPUT=web

echo "Building for $CONFIG"
RUSTFLAGS='--cfg getrandom_backend="wasm_js"' cargo build --target $TARGET $BUILD_CONFIG || { echo "cargo build failed"; exit 1; }
wasm-bindgen --out-dir $WEB_OUTPUT --target web $WASM_OUTPUT --no-typescript || { echo "wasm-bindgen failed"; exit 1; }
python3 -m http.server --directory $WEB_OUTPUT
