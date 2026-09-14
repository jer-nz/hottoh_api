# Changelog

## 0.2.0

Protocol handling checked against the HottoH Wi-Fi module firmware 10.5.0.

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

- A TCP read ending in the middle of a frame could panic and stop the TCP thread for good, leaving
  the API serving stale data.
- `set_chrono_mode` sent `true`/`false` instead of a number and did nothing; it now sends `2`/`0`.
- Stove errors (`ERR;17;`, `ERR;19;`) were silently dropped.
- Temperatures are rounded to the tenth instead of truncated.
- The logger handle was dropped at startup, which could stop buffered log writes.

### New

- `GET /api/request/{id}`: outcome of a write (`pending`, `sent`, `ok`, `error`, `timeout`).
- `GET /api/status`: connection state, last answer, last error.
- Writes are sent before the periodic reads and retried up to 3 times.
- Reconnection after 3 unanswered requests; connect timeout; configurable `poll_interval_ms`.
- Unit tests (CRC, frames, pages, write answers, reassembly, ranges).
- Dependencies updated; `crc-any` and `strum` removed; `Cargo.lock` versioned; CI moved from the
  unmaintained `actions-rs` actions to `dtolnay/rust-toolchain`.
