#!/bin/sh
# The service is neither enabled nor started: set the stove address in
# /etc/hottoh_api/config.ini first.
set -e
for dir in /var/log/hottoh_api /var/lib/hottoh_api; do
    mkdir -p "$dir"
    chown hottoh:hottoh "$dir"
    chmod 0755 "$dir"
done
if [ -d /run/systemd/system ]; then
    systemctl daemon-reload >/dev/null 2>&1 || true
    # Debian upgrade: "configure <old version>"
    if [ "$1" = "configure" ] && [ -n "$2" ]; then
        systemctl try-restart hottoh_api.service >/dev/null 2>&1 || true
    fi
fi
exit 0
