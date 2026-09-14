#!/bin/sh
# Stops the service before removal (Debian also calls this script on upgrade: "upgrade")
case "$1" in
    upgrade|failed-upgrade) exit 0 ;;
esac
if [ -d /run/systemd/system ]; then
    systemctl disable --now hottoh_api.service >/dev/null 2>&1 || true
elif command -v rc-service >/dev/null 2>&1; then
    rc-service -q hottoh_api stop 2>/dev/null || true
    rc-update -q del hottoh_api 2>/dev/null || true
fi
exit 0
