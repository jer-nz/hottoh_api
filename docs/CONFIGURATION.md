# Configuration

hottoh_api reads `config.ini`, given as argument, or found in the working directory or next to the
program. The packages install it as `/etc/hottoh_api/config.ini`.

Every section and setting is optional. **Without any file**, hottoh_api runs as a standalone app:
the stove is searched on the network, the interface is served on `127.0.0.1:3000` (the next ports
if taken; the running instance is opened if there is one) and opened in the default browser, and
logs go to the application data folder of the system (`%LOCALAPPDATA%\hottoh_api\logs`,
`~/Library/Logs/hottoh_api`, `~/.local/state/hottoh_api/logs`).

```sh
hottoh_api /etc/hottoh_api/config.ini
```

## Reference

Defaults are shown.

```ini
[stove]
ip = auto               # stove IP address or host name; empty or auto: searched on the network
port = 5001             # TCP port of the Wi-Fi module
poll_interval_ms = 1000 # pause between two polling cycles (INF + DAT 0/1/2)

[http_api]
ip = 127.0.0.1          # 0.0.0.0 to listen on every interface
port = 3000
# edit_features = true  # features can be changed from the interface and saved here (default: only without config.ini)

[web_ui]
enabled = true          # web interface at /
# port = 8080           # own port (default: the [http_api] port)
# ip = 0.0.0.0          # own address (default: the [http_api] address)
# open_browser = false  # open the interface at startup (default: only without config.ini)

[log]
level = info            # trace, debug, info, warn, error (per module: "debug, actix_server = info")
# directory = logs      # log files (default: application data folder of the system)
max_log_files = 7       # rotated log files to keep
compress = false        # gzip rotated files (the previous one stays plain)
max_file_size_mb = 100  # also rotate when the file grows beyond this size
stats_interval_s = 3600 # statistics line in the log (0 = disabled)

[features]
chrono_schedule_read = true
chrono_schedule_write = true
clock_read = true
clock_write = false
timezone_read = true
timezone_write = false
datalog_read = true
datalog_clear = false
pin_read = true
pin_write = false
module_restart = false
wifi_scan = false
cloud_read = true
firmware_update_check = true
```

The commented [`config.ini`](../config.ini) of the repository is a good starting point.

## Stove search

When `[stove] ip` is empty or `auto`, every host of the local /24 network (the one of the default
route) is tried on the module port, and the first one answering like a HottoH module is used. The
search runs again every minute while nothing is found, and after 30 s without connection (the
module may have got another address from DHCP). `GET /api/status` shows its state in `discovery`.

For a service, a fixed address (a DHCP reservation for the module on your router) starts faster
and never picks the wrong device.

## Web interface

`[web_ui]` serves the interface at `/` on the API port. With its own `port` (or `ip`), a second
server answers there with the interface **and** the API, so that the page keeps talking to the API
on its own origin; the API port then serves the API only. `enabled = false` removes the interface;
the API and Swagger UI are not affected.

The interface starts in English with the dark theme; the language and theme buttons of the top bar
are remembered by the browser.

## Features

`[features]` enables or disables the [module endpoints](API.md#module-features) and the matching
controls of the interface. Reads and the weekly schedule are enabled by default. Features that
change the setup of the module (clock, time zone, PIN), delete data or restart the module are
disabled and must be enabled explicitly. A disabled feature answers HTTP 403, and the interface
shows why instead of the control. `GET /api/features` lists the current settings, and the startup
log lists the enabled ones.

With `[http_api] edit_features = true`, the Module tab of the interface (or `POST /api/features`)
switches them on and off while running, and saves them to this section of the file: its other
lines, comments and sections are left as they are. The file must then be writable by the user
running hottoh_api (`chown hottoh /etc/hottoh_api/config.ini` for the packages). Anyone who can
reach the API can then enable any feature, so leave it off on a network you do not trust. Without
configuration file it is on, and changes last until hottoh_api stops.

`wifi_scan` stays disabled unless you need it: during the scan the module suspends its link with
the stove board, and its firmware mishandles WPA3 networks (see [Wi-Fi scan](API.md#wi-fi-scan)).

## Logs

- Stove state changes (state, on/off, modes, set points, Modbus link) and alarms are logged at
  `info`.
- A `Stats` line every `stats_interval_s`: requests, answers, timeouts, invalid frames, write
  outcomes and average latency over the period; reconnections and maximum latency since start;
  memory, threads and open file descriptors.
- At `debug`, every exchange with the stove is logged (sent frame, answer, delay): about 1 GiB per
  month before compression. Use it to investigate a problem, with `compress = true`. The cloud PIN
  is always redacted.
- Panics are logged with a backtrace. If the TCP worker thread dies, the process exits (code 70) so
  that the service manager restarts it instead of serving frozen data.
- The alarm history is kept in `alarms.json`, in the log directory.
- The server stops cleanly on SIGINT and SIGTERM.
