# Hottoh API

[![Rust CI](https://github.com/jer-nz/hottoh_api/actions/workflows/ci.yml/badge.svg)](https://github.com/jer-nz/hottoh_api/actions/workflows/ci.yml)
[![Release](https://github.com/jer-nz/hottoh_api/actions/workflows/release.yml/badge.svg)](https://github.com/jer-nz/hottoh_api/actions/workflows/release.yml)

A Rust application for controlling HottoH stoves (CMG, Edilkamin and more) through an HTTP API.

## Project Overview

Hottoh API is a bridge between the local TCP protocol of the HottoH Wi-Fi module
("Wifier 2.0", port 5001) and a RESTful HTTP API, usable from Home Assistant or any HTTP client.

This project is based on the work done by benlbrm on the [hottohpy](https://github.com/benlbrm/hottohpy)
project. Special thanks to him for his work. Since 2.0.0 the protocol handling has been checked
against the module firmware 10.5.0, the current version (field order, integer types, write ranges
and error codes).

### Key Features

- Robust TCP link: frame reassembly, CRC and length checks, automatic reconnection
- Built for long-running use: bounded queues, panics logged and supervised, periodic statistics
  (counters, latency, memory, threads, file descriptors)
- RESTful HTTP API with Swagger documentation
- Asynchronous writes with status tracking (`/api/request/{id}`)
- Weekly chrono schedule, module and stove clocks, time zone, history recorded by the module
  (data logger), cloud relay PIN and module restart, each one enabled or disabled in `[features]`
- Connection monitoring (`/api/status`)
- Very low CPU and memory usage (about 10 MiB)

## Important: one client at a time

hottoh_api talks to the stove through its HottoH Wi-Fi module (WiFire), and this module accepts
**a single local TCP connection**: as soon as a client is connected, it stops listening on port
5001 until that client disconnects. While hottoh_api runs, any other local client is refused: a
second hottoh_api instance, a test script, or the HottoH
[AppFire](https://play.google.com/store/apps/details?id=com.hottoh.appfire) mobile app when it
connects locally.

AppFire can still be used alongside hottoh_api if the **cloud mode** of the Wi-Fi module is
enabled: the app then goes through the HottoH cloud relay instead of the local connection. With
cloud mode disabled, AppFire and hottoh_api compete for the same connection and cannot be used at
the same time.

## Getting Started

### Prerequisites

- Rust 1.88 or later (edition 2024)
- A stove with a HottoH Wi-Fi module reachable on the LAN (TCP port 5001)
- `curl` at build time (utoipa-swagger-ui downloads Swagger UI)

### Installation

Each [release](https://github.com/jer-nz/hottoh_api/releases) provides statically linked Linux
binaries (amd64 and arm64, no runtime dependency) and packages that install the service:

| File | Content |
|---|---|
| `hottoh-api_<version>_<arch>.apk` | Alpine: `/usr/bin/hottoh_api`, OpenRC service, `/etc/hottoh_api/config.ini` |
| `hottoh-api_<version>_<arch>.deb` | Debian/Ubuntu: same, with a systemd unit |
| `hottoh_api-<version>-linux-<arch>.tar.gz` | Binary, sample configuration, OpenRC and systemd files |
| `hottoh_api-windows-amd64.exe`, `hottoh_api-macos-arm64` | Binary only |
| `SHA256SUMS` | Checksums |

```sh
# Alpine (the package is not signed)
apk add --allow-untrusted ./hottoh-api_2.0.0_amd64.apk
vi /etc/hottoh_api/config.ini   # stove IP address
rc-update add hottoh_api default && rc-service hottoh_api start

# Debian / Ubuntu
dpkg -i ./hottoh-api_2.0.0_amd64.deb
editor /etc/hottoh_api/config.ini
systemctl enable --now hottoh_api
```

The packages create a `hottoh` system user, logs go to `/var/log/hottoh_api/`. The service is not
started at installation; it is restarted on upgrade if it was running, and a modified
`config.ini` is kept.

From source:

```sh
git clone https://github.com/jer-nz/hottoh_api.git
cd hottoh_api
cargo build --release
```

### Configuration

`config.ini`:

```ini
[stove]
ip = 192.168.1.100      # stove IP address
port = 5001             # stove TCP port
poll_interval_ms = 1000 # optional, pause between two polling cycles (INF + DAT 0/1/2)

[http_api]
ip = 0.0.0.0            # listen on all interfaces
port = 3000             # HTTP API port

[log]
level = info            # trace, debug, info, warn, error (per module: "debug, actix_server = info")
directory = logs        # directory for log files
max_log_files = 7       # rotated log files to keep
compress = false        # optional, gzip rotated files (the previous one stays plain)
max_file_size_mb = 100  # optional, also rotate when the file grows beyond this size
stats_interval_s = 3600 # optional, statistics line in the log (0 = disabled)

[features]              # optional section, defaults shown
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

`[features]` gates the [module endpoints](#module-features). Reads and the weekly schedule are
enabled by default. Features that change the setup of the module (clock, time zone, PIN), delete
data or restart the module are disabled and must be enabled explicitly; a disabled feature answers
HTTP 403. `GET /api/features` shows the current settings, the startup log lists the enabled ones.

At `debug` level every exchange with the stove is logged (sent frame, answer, delay), about
1 GiB per month before compression: use it to investigate a problem or test stability, with
`compress = true`.

### Run

```sh
./target/release/hottoh_api config.ini
```

or `./target/release/hottoh_api` with `config.ini` in the current directory.

## API Documentation

Swagger UI: `http://localhost:3000/swagger-ui/`

### GET endpoints

| Endpoint | Content |
|---|---|
| `/api/inf` | Module information: hostname, firmware version, Wi-Fi signal |
| `/api/dat/0` | Main data: state, on/off, eco, chrono mode, temperatures, power, fan set points |
| `/api/dat/1` | Chrono programs: mode and temperature of programs 1 to 3 |
| `/api/dat/2` | Flow switch, pump, actual fan speeds, puffer/boiler/DHW/room 3 temperatures |
| `/api/status` | Link with the stove (`connected`, `last_response_at`, `last_error`), `pending_writes`, `uptime_s`, counters (`stats`) and process resources (`process`) |
| `/api/request/{id}` | Outcome of a write: `pending`, `sent`, `ok`, `error` (with `error_code`) or `timeout` |

The [module features](#module-features) add the weekly schedule, clocks, time zone, data logger
and PIN.

Temperatures are in °C. Every data page has a `last_updated` timestamp; check `/api/status` to
detect stale data when the stove is unreachable.

### POST endpoints

| Endpoint | Body | Accepted values |
|---|---|---|
| `/api/dat/set_on_off` | `{"value": true}` | `true` / `false` |
| `/api/dat/set_eco_mode` | `{"value": true}` | `true` / `false` |
| `/api/dat/set_power_level` | `{"value": 3}` | `index_power_min` to `index_power_max` |
| `/api/dat/set_ambiance_temp` | `{"ambiance": 1, "value": 21.5}` | set point min/max of the ambiance |
| `/api/dat/set_chrono_mode` | `{"value": true}` | `true` / `false` |
| `/api/dat/set_chrono_temp` | `{"chrono": 1, "value": 20.0}` | program min/max from `/api/dat/1` |
| `/api/dat/set_fan_speed` | `{"fan": 1, "value": 3}` | 0 to `index_fan_N_set_max` |

Ranges come from the values reported by the stove once they have been read, and fall back to the
firmware limits otherwise. An invalid value is refused with HTTP 400. When 32 writes are already
waiting for the stove (stove unreachable), new writes are refused with HTTP 503; a write that
could not be sent within 60 s ends as `timeout`.

Writes are asynchronous: the answer comes immediately with a `request_id`.

```json
{"success": true, "request_id": 71, "status_url": "/api/request/71", "message": "..."}
```

`GET /api/request/71` then gives the outcome returned by the stove:

```json
{"request_id": 71, "command": "AmbianceTemperature1", "value": "240", "status": "ok", "attempts": 1, ...}
```

Stove error codes: `17` value out of range, `19` the stove board refused the value (Modbus write
failed). The last 100 requests are kept.

## Module features

These endpoints use commands of the Wi-Fi module found in its firmware (10.5.0) and in the AppFire
application. Reads are sent to the stove when the endpoint is called and the answer comes back in
the HTTP response (HTTP 504 if the stove does not answer within 30 s, 502 with `error_code` if it
refuses). Writes are asynchronous like the settings above.

| Endpoint | Feature | Content |
|---|---|---|
| `GET /api/features` | - | Enabled and disabled features |
| `GET /api/chrono/schedule` | `chrono_schedule_read` | Weekly schedule (takes about 2 s, see below) |
| `POST /api/chrono/schedule` | `chrono_schedule_write` | Replaces the schedule of some days |
| `GET /api/clock` | `clock_read` | Module clock (UTC), stove clock (local), offset from the bridge |
| `POST /api/clock` | `clock_write` | `{}` (clock of the bridge) or `{"utc": 1757930400}`: sets the module and stove clocks |
| `GET /api/timezone` | `timezone_read` | `{"zone": "Europe/Paris", "known": true}` |
| `POST /api/timezone` | `timezone_write` | `{"zone": "Europe/Paris"}`, a name offered by AppFire; the module then sets the stove clock |
| `GET /api/datalog/info` | `datalog_read` | Time of the oldest and newest records |
| `GET /api/datalog?from=<utc>&count=<n>` | `datalog_read` | Records from the first one at or after `from` (default: one hour ago), `count` 1 to 100 (default 60) |
| `POST /api/datalog/clear` | `datalog_clear` | Deletes the whole history |
| `GET /api/pin` | `pin_read` | `{"pin": "..."}`: security PIN of the cloud relay, as set in AppFire |
| `POST /api/pin` | `pin_write` | `{"pin": "a1b2c3"}` (5 to 10 letters or digits); AppFire in cloud mode must be paired again |
| `POST /api/module/restart` | `module_restart` | Restarts the Wi-Fi module (no answer, data unavailable for about 15 s) |
| `GET /api/wifi/scan` | `wifi_scan` | Networks seen by the module (BSSID, SSID, RSSI, security), strongest first |
| `GET /api/cloud` | `cloud_read` | HottoH relay balancer (AppFire cloud mode), 4-noks cloud server, last upload |
| `GET /api/firmware[?refresh=true]` | `firmware_update_check` | Installed firmware, newest one on the HottoH update server, `update_available` |

### Weekly schedule

When chrono mode is on (`/api/dat/set_chrono_mode`), the stove follows a weekly schedule of
half-hour slots, each one running chrono program 1, 2 or 3 (temperatures in `/api/dat/1`) or
none (0). Days go from `sunday` to `saturday`, as in the firmware.

```json
{"slot_minutes": 30, "days": [
  {"day": "monday", "slots": [0, 0, ..., 1, 1, 1, 0, ...], "ranges": [{"start": "06:30", "end": "08:00", "program": 1}]},
  ...
]}
```

The module keeps a copy of the schedule and refreshes it from the stove when asked for it, so
the bridge reads it twice, 2 s apart, to return the current one.

`POST /api/chrono/schedule` takes the days to change, each with `ranges` (the rest of the day gets
no program) or its 48 `slots`. Days left out keep their schedule: the current one is read first.

```json
{"days": [
  {"day": "monday", "ranges": [{"start": "06:30", "end": "08:00", "program": 1}, {"start": "18:00", "end": "22:30", "program": 2}]},
  {"day": "sunday", "ranges": []}
]}
```

### Data logger

The module records the stove every 15 minutes: `utc`, `time`, `power_level`, `room_temp`,
`water_temp`, `smoke_temp` (°C), `state` and `state_raw` (AppFire shows states 50 to 99 as
alarms). To read a long period, ask again from the `utc` of the last record plus one; the list is
empty after the newest record.

### Wi-Fi scan

`wifi_scan` is disabled by default: during the scan the module suspends its link with the stove
board for a few seconds, and its table of security names stops before WPA3: a WPA3 or WPA2/WPA3
network is reported with whatever the module reads past the table (`security: "INVALID"`, the
text in `security_raw`), and in the worst case the module could crash and restart. Tested on a
WPA2/WPA3 access point: `security_raw` was `"4"`, no crash, stove link kept.
The module does not tell which network it uses, nor its Wi-Fi password: no command reads the Wi-Fi
settings back.

### Firmware update check

The module cannot tell whether a newer firmware exists. The bridge asks the HottoH update server
(`http://update.hottoh.it/update/upgrade_<major>_<minor>_<patch>.bin`, the files AppFire has the
module download) whether the next patch, minor and major versions exist, with `HEAD` requests
over plain HTTP (nothing is downloaded). The result is kept 6 hours. The update itself is not
started by the bridge: use AppFire.

### Not supported on purpose

The firmware also has commands to change the Wi-Fi settings, update the firmware, register with
the HottoH cloud and change its servers, and to fill the history with test data. They could
disconnect or break the module and are not exposed.

## Protocol notes (firmware 10.5.0)

- Frame: `#<id:5><desc:4><params length:4 hex><CMD:3><R|W|E><params;...;><CRC16:4 hex>\n`,
  CRC-16/CCITT-FALSE over everything between `#` and the CRC.
- The module handles one frame per TCP read and answers nothing to an invalid frame, an unknown
  command or an unknown DAT page: hottoh_api keeps a single request in flight with a 5 s timeout.
- Writes: `DAT W <index>;<value>;` answered by `OK;` or `ERR;<code>;`. Chrono mode is `0`
  (manual) or `2` (chrono). Indexes 12 and 13 are acknowledged but ignored by this firmware.
- In DAT page 0, `index_fan_N` repeats the fan set point; actual fan speeds are in page 2
  (`index_fan_N_speed`).
- Module commands: `SCH` (schedule: `0;` then 7 × 48 programs, Sunday first), `CLK` (`R`:
  `<module utc>;<stove local time>;`, `W`: `<utc>;`), `TMZ` and `PIN` (strings quoted as `\"...\"`),
  `ME0` (`<first utc>;<last utc>;`), `MET` (`<from utc>;<count>;`, 6 fields per record, `ERR;6;`
  when no record), `MEC` (clear), `RST` (restart, no answer). Error codes: `16` data unavailable,
  `17` missing or out of range, `18` value refused by the module.

## Logs and stability

- Stove state changes (state, on/off, modes, set points, Modbus link) are logged at `info`.
- A `Stats` line every `stats_interval_s`: requests, answers, timeouts, invalid frames, write
  outcomes and average latency over the period; reconnections and maximum latency since start;
  RSS, threads and open file descriptors (watch these for leaks).
- Panics are written to the log with a backtrace. If the TCP worker thread dies, the process exits
  (code 70) so that the service manager restarts it instead of serving frozen data.
- The server stops cleanly on SIGINT and SIGTERM.

## Project Structure

- `packaging/` - nFPM configuration, OpenRC and systemd services, package scripts
- `src/main.rs` - Application entry point
- `src/hottoh/` - Main module directory
  - `config.rs` - Configuration handling
  - `http_api.rs` - HTTP API implementation (data pages and settings)
  - `http_module.rs` - HTTP endpoints of the module features
  - `module_data.rs` - Schedule, clock, time zone, data logger, PIN, Wi-Fi scan and cloud answers
  - `firmware_update.rs` - Check of the HottoH update server
  - `logger.rs` - Logging system
  - `tcp_client.rs` - TCP worker (connection, polling, writes, reconnection)
  - `tcp_client_structs.rs` - Frame encoding/decoding and reassembly
  - `hottoh_const.rs` - Commands, states and write indexes
  - `hottoh_structs.rs` - Data pages and CRC
  - `shared_struct.rs` - State shared between the TCP worker and the HTTP API (`Bridge`)
  - `stats.rs` - Periodic statistics line and process metrics

## Publishing a release

1. Set the version in `Cargo.toml` and describe it in a `## <version>` section of `CHANGELOG.md`.
2. Open a pull request to `main`. Besides the tests, the CI builds the static binary and installs
   the `.apk` and `.deb` packages in clean Alpine and Debian containers. Merge once it is green.
3. Optionally, run the *Release* workflow by hand (Actions → Release → Run workflow): it builds every
   file and attaches them to the run, without publishing anything.
4. Tag `main` with the version and push the tag:

   ```sh
   git tag v2.0.0
   git push origin v2.0.0
   ```

The *Release* workflow then builds the binaries (static Linux amd64 and arm64, Windows, macOS),
the packages ([nFPM](https://nfpm.goreleaser.com/), `packaging/package.sh`), tests them and
publishes the GitHub release with the changelog section as notes. It stops if the tag does not
match the version in `Cargo.toml`.

## Contributing

Contributions are welcome!

## License

[MIT License](LICENSE)
