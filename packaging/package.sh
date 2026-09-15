#!/bin/sh
# Builds the release files for one architecture from a static Linux binary:
#   <out>/hottoh_api-<version>-linux-<arch>.tar.gz   binary, sample config, service files
#   <out>/hottoh-api_<version>_<arch>.apk             Alpine package (OpenRC)
#   <out>/hottoh-api_<version>_<arch>.deb             Debian package (systemd)
# Usage: packaging/package.sh <version> <amd64|arm64> <binary> <output directory>
# Requires nfpm. Run from the repository root.
set -eu

if [ $# -ne 4 ]; then
    echo "usage: $0 <version> <amd64|arm64> <binary> <output directory>" >&2
    exit 2
fi
version=$1 arch=$2 binary=$3 out=$4

case "$arch" in
    amd64|arm64) ;;
    *) echo "unsupported architecture: $arch" >&2; exit 2 ;;
esac
[ -x "$binary" ] || { echo "binary not found or not executable: $binary" >&2; exit 1; }
mkdir -p "$out"

name="hottoh_api-$version-linux-$arch"
staging=$(mktemp -d)
trap 'rm -rf "$staging"' EXIT
mkdir -p "$staging/$name"
cp "$binary" "$staging/$name/hottoh_api"
chmod 0755 "$staging/$name/hottoh_api"
cp README.md CHANGELOG.md LICENSE docs/CONFIGURATION.md docs/API.md packaging/config.ini "$staging/$name/"
cp -r packaging/openrc packaging/systemd "$staging/$name/"
tar -czf "$out/$name.tar.gz" -C "$staging" "$name"

# nFPM expands variables in version and arch, not in file paths
binary_path=$(cd "$(dirname "$binary")" && pwd)/$(basename "$binary")
sed "s#\${BINARY}#$binary_path#" packaging/nfpm.yaml > "$staging/nfpm.yaml"
for packager in apk deb; do
    VERSION="$version" ARCH="$arch" \
        nfpm package --config "$staging/nfpm.yaml" --packager "$packager" \
        --target "$out/hottoh-api_${version}_${arch}.$packager"
done
ls -l "$out"
