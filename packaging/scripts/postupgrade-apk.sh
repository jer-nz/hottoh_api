#!/bin/sh
# Restarts the service after an upgrade, if it was running
if rc-service -q hottoh_api status 2>/dev/null; then
    rc-service -q hottoh_api restart || true
fi
exit 0
