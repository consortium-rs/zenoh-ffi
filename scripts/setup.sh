#!/usr/bin/env bash
# Install dependencies locally and build the examples (macOS).
set -euo pipefail
cd "$(dirname "$0")/.."
version=1.10.1
case "$(uname -s)/$(uname -m)" in
  Darwin/arm64) target=aarch64-apple-darwin ;;
  Darwin/x86_64) target=x86_64-apple-darwin ;;
  *) echo 'This setup script currently supports macOS only.' >&2; exit 1 ;;
esac
for tool in curl unzip tar cargo uv vp node cmake protoc install_name_tool codesign; do
  command -v "$tool" >/dev/null || { echo "Missing prerequisite: $tool" >&2; exit 1; }
done
mkdir -p .zenoh
fetch() {
  local url="$1" output="$2"
  if [ ! -f "$output" ]; then
    curl -fL --retry 3 "$url" -o "$output.part"
    mv "$output.part" "$output"
  fi
}
fetch "https://github.com/eclipse-zenoh/zenoh-ts/releases/download/$version/zenoh-ts-$version-$target-standalone.zip" .zenoh/bridge.zip
fetch "https://github.com/eclipse-zenoh/zenoh-c/releases/download/$version/zenoh-c-$version-$target-standalone.zip" .zenoh/zenoh-c.zip
fetch "https://codeload.github.com/eclipse-zenoh/zenoh-cpp/tar.gz/refs/tags/$version" .zenoh/zenoh-cpp.tar.gz
unzip -qo .zenoh/bridge.zip -d .zenoh/bridge
unzip -qo .zenoh/zenoh-c.zip -d .zenoh/zenoh-c
tar -xzf .zenoh/zenoh-cpp.tar.gz -C .zenoh
# The release dylib contains its CI runner's absolute install name.
install_name_tool -id @rpath/libzenohc.dylib .zenoh/zenoh-c/lib/libzenohc.dylib
codesign --force --sign - .zenoh/zenoh-c/lib/libzenohc.dylib
# Build only the two OpenCV modules used by the example (no GUI/camera dependencies).
fetch "https://github.com/opencv/opencv/archive/refs/tags/4.13.0.tar.gz" .zenoh/opencv.tar.gz
if [ ! -d .zenoh/opencv-4.13.0 ]; then
  tar -xzf .zenoh/opencv.tar.gz -C .zenoh
fi
cmake -S .zenoh/opencv-4.13.0 -B .zenoh/opencv-build \
  -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX="$PWD/.zenoh/opencv" \
  -DBUILD_LIST=core,imgproc -DBUILD_TESTS=OFF -DBUILD_PERF_TESTS=OFF \
  -DBUILD_EXAMPLES=OFF -DBUILD_opencv_apps=OFF -DBUILD_JAVA=OFF \
  -DBUILD_opencv_python3=OFF -DWITH_IPP=OFF -DWITH_ITT=OFF \
  -DWITH_OPENCL=OFF -DWITH_LAPACK=OFF -DWITH_EIGEN=OFF -DENABLE_CCACHE=OFF \
  -DWITH_JPEG=OFF -DWITH_PNG=OFF -DWITH_TIFF=OFF -DWITH_WEBP=OFF \
  -DWITH_OPENJPEG=OFF -DWITH_OPENEXR=OFF -DWITH_AVIF=OFF \
  -DWITH_PROTOBUF=OFF -DWITH_FLATBUFFERS=OFF
cmake --build .zenoh/opencv-build --parallel 4
cmake --install .zenoh/opencv-build
protoc -I proto --python_out=python/src/python proto/demo.proto
(cd python && uv sync --locked)
(cd ts && vp install --frozen-lockfile)
cargo build --locked --manifest-path rust/Cargo.toml
cmake -S cpp -B cpp/build
cmake --build cpp/build
