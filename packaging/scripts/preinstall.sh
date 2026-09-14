#!/bin/sh
# Creates the system user running the service (Alpine busybox or Debian shadow tools)
set -e
if ! grep -q '^hottoh:' /etc/group; then
    if command -v groupadd >/dev/null 2>&1; then
        groupadd --system hottoh
    else
        addgroup -S hottoh
    fi
fi
if ! grep -q '^hottoh:' /etc/passwd; then
    if command -v useradd >/dev/null 2>&1; then
        useradd --system --gid hottoh --home-dir /var/lib/hottoh_api --no-create-home \
            --shell /usr/sbin/nologin hottoh
    else
        adduser -S -D -H -h /var/lib/hottoh_api -s /sbin/nologin -G hottoh hottoh
    fi
fi
exit 0
