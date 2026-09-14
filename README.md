# Hottoh API

[![Rust CI](https://github.com/jer-nz/hottoh_api/actions/workflows/ci.yml/badge.svg)](https://github.com/jer-nz/hottoh_api/actions/workflows/ci.yml)
[![Release](https://github.com/jer-nz/hottoh_api/actions/workflows/release.yml/badge.svg)](https://github.com/jer-nz/hottoh_api/actions/workflows/release.yml)

A Rust application for controlling HottoH stoves (CMG, Edilkamin and more) through an HTTP API.

## Project Overview

Hottoh API is a bridge between the local TCP protocol of the HottoH Wi-Fi module
("Wifier 2.0", port 5001) and a RESTful HTTP API, usable from Home Assistant or any HTTP client.

This project is based on the work done by benlbrm on the [hottohpy](https://github.com/benlbrm/hottohpy)
project. Special thanks to him for his work. Since 0.2.0 the protocol handling has been checked
against the module firmware 10.5.0 (field order, integer types, write ranges and error codes).

### Key Features

- Robust TCP link: frame reassembly, CRC and length checks, automatic reconnection
- Built for long-running use: bounded queues, panics logged and supervised, periodic statistics
  (counters, latency, memory, threads, file descriptors)
- RESTful HTTP API with Swagger documentation
- Asynchronous writes with status tracking (`/api/request/{id}`)
- Connection monitoring (`/api/status`)
- Very low CPU and memory usage (about 10 MiB)

## Important: one client at a time

The Wi-Fi module accepts **a single TCP client**: it closes its listening socket once a client is
connected. While hottoh_api runs, other local tools (or a second hottoh_api instance) are refused.
The HottoH mobile app is not affected: it goes through the vendor cloud relay.

## Getting Started

### Prerequisites

- Rust 1.88 or later (edition 2024)
- A stove with a HottoH Wi-Fi module reachable on the LAN (TCP port 5001)
- `curl` at build time (utoipa-swagger-ui downloads Swagger UI)

### Installation

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
```

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

## Protocol notes (firmware 10.5.0)

- Frame: `#<id:5><desc:4><params length:4 hex><CMD:3><R|W|E><params;...;><CRC16:4 hex>\n`,
  CRC-16/CCITT-FALSE over everything between `#` and the CRC.
- The module handles one frame per TCP read and answers nothing to an invalid frame, an unknown
  command or an unknown DAT page: hottoh_api keeps a single request in flight with a 5 s timeout.
- Writes: `DAT W <index>;<value>;` answered by `OK;` or `ERR;<code>;`. Chrono mode is `0`
  (manual) or `2` (chrono). Indexes 12 and 13 are acknowledged but ignored by this firmware.
- In DAT page 0, `index_fan_N` repeats the fan set point; actual fan speeds are in page 2
  (`index_fan_N_speed`).

## Logs and stability

- Stove state changes (state, on/off, modes, set points, Modbus link) are logged at `info`.
- A `Stats` line every `stats_interval_s`: requests, answers, timeouts, invalid frames, write
  outcomes and average latency over the period; reconnections and maximum latency since start;
  RSS, threads and open file descriptors (watch these for leaks).
- Panics are written to the log with a backtrace. If the TCP worker thread dies, the process exits
  (code 70) so that the service manager restarts it instead of serving frozen data.
- The server stops cleanly on SIGINT and SIGTERM.

## Project Structure

- `src/main.rs` - Application entry point
- `src/hottoh/` - Main module directory
  - `config.rs` - Configuration handling
  - `http_api.rs` - HTTP API implementation
  - `logger.rs` - Logging system
  - `tcp_client.rs` - TCP worker (connection, polling, writes, reconnection)
  - `tcp_client_structs.rs` - Frame encoding/decoding and reassembly
  - `hottoh_const.rs` - Commands, states and write indexes
  - `hottoh_structs.rs` - Data pages and CRC
  - `shared_struct.rs` - State shared between the TCP worker and the HTTP API (`Bridge`)
  - `stats.rs` - Periodic statistics line and process metrics

## Contributing

Contributions are welcome!

## License

[MIT License](LICENSE)
