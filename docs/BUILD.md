# Building and development

## Prerequisites

- Rust 1.88 or later (edition 2024)
- `curl` at build time: `utoipa-swagger-ui` downloads Swagger UI
- For testing against a real stove: a HottoH Wi-Fi module reachable on the network (TCP port 5001)

## Build and run

```sh
git clone https://github.com/jer-nz/hottoh_api.git
cd hottoh_api
cargo build --release
./target/release/hottoh_api config.ini
```

Without argument, `config.ini` is looked for in the working directory and next to the program;
without any, the program runs as a standalone app (see [Configuration](CONFIGURATION.md)).

Remember that the Wi-Fi module accepts a single local client: stop any other hottoh_api instance
(a service, for example) before running one from source.

## Checks

The CI runs the same commands on every pull request:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --all-features --locked
```

The tests cover the CRC, frames, data pages, write answers, reassembly and ranges, the TCP worker
against a fake Wi-Fi module (split and noisy answers, silent stove, closed connection, refused
writes), the HTTP handlers and the web routes. The CI also checks the minimum Rust version, builds
the static Linux binary and installs the `.apk` and `.deb` packages in clean Alpine and Debian
containers.

## Web interface

`web/index.html`, `web/app.css` and `web/app.js` are plain files, without framework or build step,
embedded in the binary at compile time (`include_str!` in `src/hottoh/web_ui.rs`): **any change
needs a rebuild**. The page loads nothing from outside (strict Content Security Policy) and only
talks to the API of its own origin.

To work on the interface without a stove, serve `web/` from any small HTTP server that answers the
`/api/...` endpoints with recorded JSON (routes `/`, `/ui/app.css` and `/ui/app.js`).

## Project structure

- `src/main.rs` - Entry point: configuration, logger, TCP worker, HTTP servers
- `src/hottoh/`
  - `alarms.rs` - Alarm history (`alarms.json` in the log directory)
  - `config.rs` - Configuration handling
  - `discovery.rs` - Search of the stove on the local network
  - `firmware_update.rs` - Check of the HottoH update server
  - `hottoh_const.rs` - Commands, states and write indexes
  - `hottoh_structs.rs` - Data pages and CRC
  - `http_api.rs` - HTTP API (data pages and settings)
  - `http_module.rs` - HTTP endpoints of the module features
  - `logger.rs` - Logging
  - `module_data.rs` - Schedule, clock, time zone, data logger, PIN, Wi-Fi scan and cloud answers
  - `shared_struct.rs` - State shared between the TCP worker and the HTTP API (`Bridge`)
  - `stats.rs` - Periodic statistics line and process metrics
  - `tcp_client.rs` - TCP worker (connection, polling, writes, reconnection)
  - `tcp_client_structs.rs` - Frame encoding/decoding and reassembly
  - `web_ui.rs` - Web interface routes
- `web/` - Web interface, embedded in the binary
- `packaging/` - nFPM configuration, OpenRC and systemd services, package scripts
- `docs/` - Documentation and screenshots

## Protocol notes (firmware 10.5.0)

- Frame: `#<id:5><desc:4><params length:4 hex><CMD:3><R|W|E><params;...;><CRC16:4 hex>\n`,
  CRC-16/CCITT-FALSE over everything between `#` and the CRC.
- The module accepts a single TCP client: once a client is connected, it stops listening on port
  5001 until that client disconnects.
- The module handles one frame per TCP read and answers nothing to an invalid frame, an unknown
  command or an unknown DAT page: hottoh_api keeps a single request in flight with a 5 s timeout,
  and reconnects after 3 unanswered requests.
- Writes: `DAT W <index>;<value>;` answered by `OK;` or `ERR;<code>;`. Chrono mode is `0` (manual)
  or `2` (chrono). Indexes 12 and 13 are acknowledged but ignored by this firmware.
- In DAT page 0, `index_fan_N` repeats the fan set point; actual fan speeds are in page 2
  (`index_fan_N_speed`).
- Module commands: `SCH` (schedule: `0;` then 7 × 48 programs, Sunday first), `CLK` (`R`:
  `<module utc>;<stove local time>;`, `W`: `<utc>;`), `TMZ` and `PIN` (strings quoted as `\"...\"`),
  `ME0` (`<first utc>;<last utc>;`), `MET` (`<from utc>;<count>;`, 6 fields per record, `ERR;6;`
  when no record), `MEC` (clear), `RST` (restart, no answer). Error codes: `16` data unavailable,
  `17` missing or out of range, `18` value refused by the module.

## Release files

Each release provides:

| File | Content |
|---|---|
| `hottoh-api_<version>_<arch>.apk` | Alpine: `/usr/bin/hottoh_api`, OpenRC service, `/etc/hottoh_api/config.ini` |
| `hottoh-api_<version>_<arch>.deb` | Debian/Ubuntu: same, with a systemd unit |
| `hottoh_api-<version>-linux-<arch>.tar.gz` | Static binary, sample configuration, OpenRC and systemd files |
| `hottoh_api-windows-amd64.exe`, `hottoh_api-macos-arm64` | Binary only |
| `SHA256SUMS` | Checksums |

Linux binaries are statically linked (musl, amd64 and arm64). The packages
([nFPM](https://nfpm.goreleaser.com/), `packaging/package.sh`) create a `hottoh` system user and
log to `/var/log/hottoh_api/`. The service is not started at installation; it is restarted on
upgrade if it was running, and a modified `config.ini` is kept.

## Publishing a release

1. Set the version in `Cargo.toml` and describe it in a `## <version>` section of `CHANGELOG.md`.
2. Open a pull request to `main`. Merge once the CI is green.
3. Optionally, run the *Release* workflow by hand (Actions → Release → Run workflow): it builds every
   file and attaches them to the run, without publishing anything.
4. Tag `main` with the version and push the tag:

   ```sh
   git tag v2.3.0
   git push origin v2.3.0
   ```

The *Release* workflow then builds the binaries (static Linux amd64 and arm64, Windows, macOS) and
the packages, tests them and publishes the GitHub release with the changelog section as notes. It
stops if the tag does not match the version in `Cargo.toml`.
