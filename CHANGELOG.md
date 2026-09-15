# Changelog

## Unreleased

### Features changed from the interface

- `POST /api/features` enables or disables features while running and saves them to the
  `[features]` section of `config.ini` (only those lines are rewritten, comments kept). Allowed by the
  new `[http_api] edit_features` (default: only without configuration file). `GET /api/config` gives
  the file in use and whether features can be changed.
- Module tab: a switch for each feature, grouped by topic; the risky ones turn orange.

### Module tab redesigned

- Wi-Fi module card: host name, firmware, signal, address, **stove clock** (on time, or its offset
  in minutes) and time zone, manufacturer, stove setup.
- Clock: the stove clock is shown and judged, not the module one, which is only the stove minute
  plus a counter reset every few seconds; the stove minute changes about 20 s after the real one, so
  it is not reported late in the first half of a minute. Setting the clock says the module accepted
  it and warns when the stove is already on time (nothing visible changes then).
- Nothing is asked to the HottoH servers or the module without a click: the firmware check, the
  cloud servers and the data logger are read on demand (Details card). Only the clock and time zone
  are read when the tab opens.
- Details card: clocks (stove to the minute, computer, offset, module), data logger, cloud and PIN.
- Tools card: set the clock, time zone, Wi-Fi scan, relay PIN, clear the history, restart the
  module; a disabled one says which feature to enable.

### Interface fixes

- Phone: the day timeline of the schedule no longer scrolls sideways to reach the afternoon; it is
  shown as two half days, one above the other. Day tabs and power levels fit on one line.
- Gaps: empty alarm and discovery placeholders no longer add space above the dashboard cards;
  temperature tiles fill the row.

### Fixes found by a test on a real stove (full ignition and shutdown cycle)

- Web interface: the power applied by the stove (`index_power_level`) is shown in percent, as the
  stove display does (it showed "100 / 10"); the button of the matching level is marked. Same in
  the History charts and table.
- History: the heating time is marked as approximate (records every 15 minutes).
- `set_ambiance_temp` and `set_chrono_temp`: an out of range temperature is reported in °C
  (`between 5.0 and 55.0 °C`) instead of tenths, and the answer shows the value sent after rounding
  to the tenth (`21.25` gives `21.3 °C`).
- Changes of the chrono program temperatures (DAT page 1) are logged, like those of page 0.
- The startup log gives the listening address of the web interface (`http://0.0.0.0:80/`) instead
  of a local URL.
- API documentation: meaning of `index_power_level`, rounding of temperatures.

## 2.2.0

### New: web interface

A single page embedded in the binary (no file to install, no external resource), served at `/` next
to the API and Swagger UI. English by default, or French; dark theme by default, light or following
the system; usable on a phone.

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
- Documentation split: a shorter README with screenshots, and `docs/CONFIGURATION.md`,
  `docs/API.md` (with a Home Assistant example) and `docs/BUILD.md`. The packages and archives
  include the configuration and API guides.

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
