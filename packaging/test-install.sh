#!/bin/sh
# Installs a package in a clean container and checks the result.
# Usage: packaging/test-install.sh <package.apk|package.deb>   (needs docker)
set -eu
package=$1
dir=$(cd "$(dirname "$package")" && pwd)
file=$(basename "$package")

case "$file" in
    *.apk) image=alpine:3 install="apk add --no-cache --allow-untrusted /pkg/$file" service=/etc/init.d/hottoh_api ;;
    *.deb) image=debian:stable-slim install="dpkg -i /pkg/$file" service=/usr/lib/systemd/system/hottoh_api.service ;;
    *) echo "unknown package type: $file" >&2; exit 2 ;;
esac

docker run --rm -v "$dir:/pkg:ro" "$image" sh -euxc "
    $install
    test -x /usr/bin/hottoh_api
    test -f /etc/hottoh_api/config.ini
    test -f $service
    grep -q "^hottoh:" /etc/passwd
    test \"\$(stat -c %U /var/log/hottoh_api)\" = hottoh
    # static binary: runs without any library, and reports a missing configuration
    ! /usr/bin/hottoh_api /nonexistent.ini 2>/tmp/err
    grep -q 'Failed to load configuration' /tmp/err
    # the packaged configuration is valid: the service starts and answers on port 3000
    sed -i 's#^directory = .*#directory = /tmp#' /etc/hottoh_api/config.ini
    /usr/bin/hottoh_api /etc/hottoh_api/config.ini & pid=\$!
    sleep 3
    kill -0 \$pid
    kill \$pid
    wait \$pid || true
    grep -q 'HTTP server on 0.0.0.0:3000' /tmp/hottoh_api_r*.log
"
echo "OK: $file"
