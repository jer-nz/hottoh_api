# Changelog

## 2.2.0

### New: web interface

A single page embedded in the binary (no file to install, no external resource), served at `/` next
to the API and Swagger UI. English or French, light or dark theme, usable on a phone.

- Stove: thermostat dial, on/off, eco and chrono modes, power level, fans, temperatures, current
  alarm with what to do.
- Schedule: temperatures of the Eco, Normal and Comfort programs, week overview, day editor with
  periods (from, to, program) or drawn on a timeline, copy of a day to others, save of the changed
  days only.
- History: data logger charts over 6 h to 7 days (records kept in the browser), heating time,
  alarm history, table view.
- Module: firmware update check, clocks and time zone, data logger, cloud relay and PIN, Wi-Fi scan,
  module restart and the `[features]` settings.
- Diagnostics: link with the stove, counters, resources, recent requests, raw pages, JSON snapshot.
- Console: any endpoint of the OpenAPI description with example bodies, and the outcome of writes.

New `[web_ui]` section in `config.ini`: `enabled` (default `true`), `port` and `ip` (default: those of
`[http_api]`). On its own port the interface server also answers the API, on the same origin.

### New: runs without configuration

For someone who just runs the program on Windows or macOS: without `config.ini` (argument, working
directory or next to the program), hottoh_api searches the stove on the local network, serves the
interface on `127.0.0.1:3000` (next ports if taken; opens the running instance if there is one) and
opens it in the default browser.

- `[stove] ip` empty or `auto`: the module is searched on the /24 network of the computer (INF
  request on port 5001), again every minute while not found and after 30 s without connection.
  `GET /api/status` reports the search in `discovery`.
- Every section of `config.ini` is optional. Defaults: `[http_api] ip = 127.0.0.1`, `port = 3000`;
  logs in the application data folder of the system.
- `[web_ui] open_browser`.

### New: alarm history

- `GET /api/alarms`: current alarm and the last 100 alarms (pellets low or out, power cut, ignition
  failure, no pellets, door open and the other states 50 to 99), kept in `alarms.json` in the log
  directory. Alarm start and end are logged.

### Changes

- `GET /api/status` adds `connected_since`, the start of the current connection.
- `GET /api/requests`: the last 100 queued requests, newest first.
- `GET /api/timezone` adds `available`, the time zone names accepted by `POST /api/timezone`.

## 2.1.0

### New: Wi-Fi module features

Commands of the module found in its firmware 10.5.0 and in the AppFire application, each one
enabled or disabled in the new `[features]` section of `config.ini` (HTTP 403 when disabled).

- Weekly chrono schedule: `GET` and `POST /api/chrono/schedule`, as half-hour slots or time
  ranges per day; days left out of a `POST` are kept.
- Clocks: `GET /api/clock` (module, stove, offset from the bridge); `POST /api/clock` sets the
  module and stove clocks (disabled by default).
- Time zone: `GET /api/timezone`; `POST /api/timezone` (disabled by default).
- Data logger: `GET /api/datalog/info`, `GET /api/datalog?from=&count=` (history recorded by the
  module every 15 minutes); `POST /api/datalog/clear` (disabled by default).
- Cloud relay PIN: `GET /api/pin`; `POST /api/pin` (disabled by default).
- `POST /api/module/restart` (disabled by default).
- Wi-Fi scan: `GET /api/wifi/scan` (disabled by default: suspends the stove link, and the firmware
  mishandles WPA3 networks).
- Cloud settings: `GET /api/cloud` (HottoH relay balancer, 4-noks cloud server, last upload).
- Firmware update check: `GET /api/firmware` asks the HottoH update server whether a newer module
  firmware is published.
- `GET /api/features`.

### Changes

- Reads made on demand go through the same queue as the writes: `/api/status` counts them in
  `stats.reads_ok`, `reads_refused` and `reads_failed`, and the `Stats` log line shows them.
- The PIN never appears in the log (frames are redacted, even at `debug` level) nor in
  `/api/request/{id}`.
- An answer without parameters (empty data logger) is accepted.

## 2.0.0

Follows 1.0.0 (April 2025, published from 0.1.0 sources). The major version reflects the breaking
API changes below.

### Wi-Fi module firmware 10.5.0

- Supports the current firmware of the HottoH Wi-Fi module ("Wifier 2.0"), **10.5.0**. The whole
  protocol handling was checked against its code: frame format and CRC, field order and integer
  types of the INF and DAT pages, write indexes and their ranges, `ERR` codes, one client and one
  frame at a time.
- Tested on a real stove with firmware 10.5.0: reading of every page, writes, refused values,
  reconnection after the module restarts.
- Updating modules still on firmware 10.1.0 is recommended: that version loses its Wi-Fi link
  about every five minutes and restarts (its Internet check loads a whole web page in memory).
  hottoh_api reconnects by itself, but data is unavailable during each restart.

### Breaking changes

- `GET /api/dat/1`: fields were shifted by one and are now aligned with the firmware:
  `index_chrono_mode`, `index_program_{1,2,3}_temp`, `_min`, `_max`, in °C
  (`index_state` and `index_temperature_*` are removed).
- `GET /api/dat/2`: `index_airex_{1,2,3}` renamed `index_fan_{1,2,3}_speed` (actual fan speeds);
  temperatures are now in °C like page 0.
- `GET /api/dat/0`: `index_timer_on` renamed `index_chrono_mode` (0 manual, 2 chrono);
  new `index_stove_state_raw`; an unknown state is reported as `"Unknown"` instead of making the
  whole page invalid.
- Write values are validated against the ranges reported by the stove (power, temperatures, fans).
- Rust edition 2024, minimum Rust 1.88 (required by actix-web 4.15).

### Fixes

- Ctrl-C panicked in the `ctrlc` handler (`System::current()` called outside the actix system);
  `ctrlc` is removed, actix-web already stops on SIGINT and SIGTERM.
- Panics were only printed on stderr and lost under a service manager: they are now logged with a
  backtrace, and the process exits if the TCP worker thread dies.
- The write queue was unbounded, and writes queued while the stove was unreachable stayed
  `pending` until the next connection.
- Log lines of the last second before a crash could be lost (buffered writer).

- A TCP read ending in the middle of a frame could panic and stop the TCP thread for good, leaving
  the API serving stale data.
- `set_chrono_mode` sent `true`/`false` instead of a number and did nothing; it now sends `2`/`0`.
- Stove errors (`ERR;17;`, `ERR;19;`) were silently dropped.
- Temperatures are rounded to the tenth instead of truncated.
- The logger handle was dropped at startup, which could stop buffered log writes.

### New

- Release files: static Linux binaries (amd64, arm64), Alpine `.apk` (OpenRC) and Debian `.deb`
  (systemd) packages, archives with service files, Windows and macOS binaries, `SHA256SUMS`.

- `GET /api/request/{id}`: outcome of a write (`pending`, `sent`, `ok`, `error`, `timeout`).
- `GET /api/status`: connection state, last answer, last error.
- Writes are sent before the periodic reads and retried up to 3 times.
- Reconnection after 3 unanswered requests; connect timeout; configurable `poll_interval_ms`.
- `/api/status` also returns `uptime_s`, `version`, counters (`stats`) and process resources.
- Log: every exchange at `debug` level, stove state changes at `info`, periodic `Stats` line
  (`stats_interval_s`), size based rotation (`max_file_size_mb`), gzip of rotated files
  (`compress`); repeated connection failures are summarised.
- Writes refused with HTTP 503 when 32 are already waiting.
- Tests: CRC, frames, pages, write answers, reassembly, ranges, random input; TCP worker against
  a fake Wi-Fi module (split and noisy answers, silent stove, closed connection, unreachable
  stove, refused writes); HTTP handlers.
- Dependencies updated; `crc-any` and `strum` removed; `Cargo.lock` versioned; CI moved from the
  unmaintained `actions-rs` actions to `dtolnay/rust-toolchain`.
