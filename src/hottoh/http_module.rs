//! Endpoints of the optional Wi-Fi module features: weekly chrono schedule, clocks, time zone,
//! data logger, cloud relay PIN and restart. Each one is gated by the `[features]` section of the
//! configuration and answers HTTP 403 when disabled.
//!
//! Reads are sent to the stove on demand and the handler waits for the answer; writes are
//! asynchronous like the DAT settings (`request_id`, then `GET /api/request/{id}`).

use crate::hottoh::config::FeaturesConfig;
use crate::hottoh::firmware_update::{UPDATE_HOST, firmware_exists, firmware_url, latest_firmware};
use crate::hottoh::hottoh_const::{Command, CommandType, TIME_ZONES, error_message};
use crate::hottoh::http_api::{ApiError, queued_response};
use crate::hottoh::module_data::{
    ClockInfo, CloudServer, DAYS, DatalogBounds, DatalogRecord, ProgramRange, SLOTS_PER_DAY,
    Server, Version, WeeklySchedule, WifiNetwork, balancer_from_params,
    cloud_last_upload_from_params, cloud_server_from_params, local_time, pin_from_params, quote,
    ranges_to_slots, slots_to_ranges, time_zone_from_answer,
};
use crate::hottoh::shared_struct::{Bridge, Job, MAX_QUEUED, RequestState, RequestStatus};
use actix_web::{HttpResponse, web};
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use utoipa::{IntoParams, OpenApi, ToSchema};

type Shared = web::Data<Bridge>;
type Features = web::Data<FeaturesConfig>;

/// Longest wait for the answer to a read made on demand (queue, retries included)
const READ_WAIT: Duration = Duration::from_secs(30);
/// `SCH R` returns the copy kept by the module and asks it to read the stove again: a second
/// read after this delay gets the current schedule
const SCHEDULE_REFRESH_DELAY: Duration = Duration::from_secs(2);
/// Records per `GET /api/datalog` (the firmware accepts 250, AppFire asks for 20 at a time; the
/// module has little memory)
const DATALOG_MAX_COUNT: u16 = 100;
const DATALOG_DEFAULT_COUNT: u16 = 60;
/// `MET` error code when no record is at or after the requested time
const DATALOG_NO_RECORD: i32 = 6;
/// Earliest accepted clock value (2020-01-01), to catch a value in milliseconds or a mistake
const MIN_UTC: u32 = 1_577_836_800;

#[derive(OpenApi)]
#[openapi(
    paths(
        get_features,
        get_schedule,
        post_schedule,
        get_clock,
        post_clock,
        get_timezone,
        post_timezone,
        get_datalog_info,
        get_datalog,
        post_datalog_clear,
        get_pin,
        post_pin,
        post_module_restart,
        get_wifi_scan,
        get_cloud,
        get_firmware
    ),
    components(schemas(
        FeaturesConfig,
        ScheduleBody,
        DayScheduleInput,
        ProgramRange,
        ClockBody,
        TimeZoneBody,
        PinBody,
        ClockInfo,
        DatalogBounds,
        DatalogRecord,
        WifiNetwork,
        Server,
        CloudServer
    )),
    tags((name = "module", description = "Optional Wi-Fi module features, see [features] in config.ini"))
)]
pub struct ModuleApiDoc;

fn require(enabled: bool, name: &'static str) -> Result<(), ApiError> {
    if enabled {
        Ok(())
    } else {
        Err(ApiError::FeatureDisabled(name))
    }
}

fn now_utc() -> u32 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| u32::try_from(d.as_secs()).unwrap_or(u32::MAX))
        .unwrap_or(0)
}

/// Queues a request, or `QueueFull`
fn queue(bridge: &Bridge, job: Job) -> Result<u32, ApiError> {
    bridge.queue_job(job).ok_or(ApiError::QueueFull(MAX_QUEUED))
}

/// Queues a write or command and answers with its `request_id`
fn queue_command(
    bridge: &Bridge,
    command: Command,
    command_type: CommandType,
    params: Vec<String>,
    label: &'static str,
    shown_value: String,
    expects_answer: bool,
) -> Result<HttpResponse, ApiError> {
    let request_id = queue(
        bridge,
        Job {
            command,
            command_type,
            params,
            label,
            shown_value: shown_value.clone(),
            expects_answer,
        },
    )?;
    Ok(queued_response(request_id, label, &shown_value))
}

/// Sends a read on demand and waits for its outcome (`ok` or `error`, with the answer kept)
async fn read(
    bridge: &Bridge,
    command: Command,
    params: Vec<String>,
    label: &'static str,
) -> Result<RequestStatus, ApiError> {
    let request_id = queue(
        bridge,
        Job {
            command,
            command_type: CommandType::Read,
            params,
            label,
            shown_value: String::new(),
            expects_answer: true,
        },
    )?;
    let started = Instant::now();
    loop {
        let status = bridge.state().get_request(request_id).cloned();
        match status {
            Some(status) if matches!(status.status, RequestState::Ok | RequestState::Error) => {
                return Ok(status);
            }
            Some(status) if status.status == RequestState::Timeout => {
                return Err(ApiError::StoveTimeout(format!(
                    "{} (request {}): {}",
                    label,
                    request_id,
                    status.message.unwrap_or_default()
                )));
            }
            // Pushed out of the history by more recent requests
            None => {
                return Err(ApiError::StoveTimeout(format!(
                    "{} (request {}): outcome lost",
                    label, request_id
                )));
            }
            Some(_) if started.elapsed() >= READ_WAIT => {
                return Err(ApiError::StoveTimeout(format!(
                    "{} (request {}): still waiting after {} s, see /api/request/{}",
                    label,
                    request_id,
                    READ_WAIT.as_secs(),
                    request_id
                )));
            }
            Some(_) => actix_web::rt::time::sleep(Duration::from_millis(50)).await,
        }
    }
}

/// Answer of a read that must succeed
fn answer_ok(status: RequestStatus) -> Result<Vec<String>, ApiError> {
    match status.status {
        RequestState::Ok => Ok(status.answer),
        _ => Err(stove_error(status.error_code)),
    }
}

fn stove_error(code: Option<i32>) -> ApiError {
    ApiError::Stove {
        code,
        message: error_message(code).to_string(),
    }
}

fn bad_answer(error: impl std::fmt::Display) -> ApiError {
    ApiError::BadAnswer(error.to_string())
}

/// Enabled and disabled features
#[utoipa::path(get, path = "/api/features", responses((status = 200, body = FeaturesConfig)), tag = "module")]
pub async fn get_features(features: Features) -> HttpResponse {
    HttpResponse::Ok().json(features.get_ref())
}

/// Current weekly schedule, read twice (see `SCHEDULE_REFRESH_DELAY`)
async fn read_schedule(bridge: &Bridge) -> Result<WeeklySchedule, ApiError> {
    const LABEL: &str = "ChronoScheduleRead";
    answer_ok(read(bridge, Command::Sch, vec!["0".into()], LABEL).await?)?;
    actix_web::rt::time::sleep(SCHEDULE_REFRESH_DELAY).await;
    let answer = answer_ok(read(bridge, Command::Sch, vec!["0".into()], LABEL).await?)?;
    WeeklySchedule::from_params(&answer).map_err(bad_answer)
}

/// Schedule of one day
#[derive(Serialize)]
struct DayScheduleOutput {
    day: &'static str,
    /// Program of each half-hour slot from 00:00 (0 = none, 1 to 3 = chrono programs)
    slots: Vec<u8>,
    /// The same as time ranges, slots without program left out
    ranges: Vec<ProgramRange>,
}

/// Weekly chrono schedule: for each day, the chrono program (1 to 3, 0 = none) of every
/// half hour. The stove follows it when chrono mode is on (`/api/dat/set_chrono_mode`).
#[utoipa::path(get, path = "/api/chrono/schedule",
    responses((status = 200), (status = 403), (status = 502), (status = 503), (status = 504)),
    tag = "module")]
pub async fn get_schedule(bridge: Shared, features: Features) -> Result<HttpResponse, ApiError> {
    require(features.chrono_schedule_read, "chrono_schedule_read")?;
    let schedule = read_schedule(&bridge).await?;
    let days: Vec<DayScheduleOutput> = DAYS
        .iter()
        .zip(schedule.days.iter())
        .map(|(day, slots)| DayScheduleOutput {
            day,
            slots: slots.to_vec(),
            ranges: slots_to_ranges(slots),
        })
        .collect();
    Ok(HttpResponse::Ok().json(json!({ "slot_minutes": 30, "days": days })))
}

/// New schedule of one day: `ranges` or the 48 `slots`
#[derive(Deserialize, ToSchema)]
pub struct DayScheduleInput {
    /// `sunday` to `saturday`
    #[schema(example = "monday")]
    day: String,
    /// Program of each half-hour slot from 00:00, 48 values from 0 to 3
    slots: Option<Vec<u8>>,
    /// Program ranges; the rest of the day gets no program
    ranges: Option<Vec<ProgramRange>>,
}

/// Days to replace; days left out keep their current schedule
#[derive(Deserialize, ToSchema)]
pub struct ScheduleBody {
    days: Vec<DayScheduleInput>,
}

/// Slots of each day given in the body, by day index
fn parse_days(days: &[DayScheduleInput]) -> Result<[Option<[u8; SLOTS_PER_DAY]>; 7], ApiError> {
    let invalid = |message: String| ApiError::InvalidParameter(message);
    let mut parsed: [Option<[u8; SLOTS_PER_DAY]>; 7] = [None; 7];
    if days.is_empty() {
        return Err(invalid("days must not be empty".into()));
    }
    for input in days {
        let name = input.day.to_ascii_lowercase();
        let index = DAYS
            .iter()
            .position(|day| *day == name)
            .ok_or_else(|| invalid(format!("unknown day '{}'", input.day)))?;
        if parsed[index].is_some() {
            return Err(invalid(format!("day '{}' given twice", name)));
        }
        let slots = match (&input.slots, &input.ranges) {
            (Some(slots), None) => {
                let slots: [u8; SLOTS_PER_DAY] = slots.as_slice().try_into().map_err(|_| {
                    invalid(format!(
                        "{}: slots must have {} values (got {})",
                        name,
                        SLOTS_PER_DAY,
                        slots.len()
                    ))
                })?;
                if let Some(bad) = slots.iter().find(|&&p| p > 3) {
                    return Err(invalid(format!(
                        "{}: slot programs must be between 0 and 3 (got {})",
                        name, bad
                    )));
                }
                slots
            }
            (None, Some(ranges)) => {
                ranges_to_slots(ranges).map_err(|e| invalid(format!("{}: {}", name, e)))?
            }
            _ => {
                return Err(invalid(format!("{}: give either slots or ranges", name)));
            }
        };
        parsed[index] = Some(slots);
    }
    Ok(parsed)
}

/// Replaces the schedule of the given days. When some days are left out, the current schedule
/// is read first so that they are kept.
#[utoipa::path(post, path = "/api/chrono/schedule", request_body = ScheduleBody,
    responses((status = 200), (status = 400), (status = 403), (status = 502), (status = 503), (status = 504)),
    tag = "module")]
pub async fn post_schedule(
    body: web::Json<ScheduleBody>,
    bridge: Shared,
    features: Features,
) -> Result<HttpResponse, ApiError> {
    require(features.chrono_schedule_write, "chrono_schedule_write")?;
    let parsed = parse_days(&body.days)?;
    let mut schedule = if parsed.iter().all(Option::is_some) {
        WeeklySchedule::default()
    } else {
        read_schedule(&bridge).await?
    };
    let mut changed = Vec::new();
    for (index, slots) in parsed.iter().enumerate() {
        if let Some(slots) = slots {
            schedule.days[index] = *slots;
            changed.push(DAYS[index]);
        }
    }
    queue_command(
        &bridge,
        Command::Sch,
        CommandType::Write,
        schedule.to_params(),
        "ChronoSchedule",
        changed.join(","),
        true,
    )
}

/// Clock of the Wi-Fi module and of the stove, compared with the clock of the bridge
#[utoipa::path(get, path = "/api/clock",
    responses((status = 200, body = ClockInfo), (status = 403), (status = 502), (status = 503), (status = 504)),
    tag = "module")]
pub async fn get_clock(bridge: Shared, features: Features) -> Result<HttpResponse, ApiError> {
    require(features.clock_read, "clock_read")?;
    let answer = answer_ok(read(&bridge, Command::Clk, vec![], "ClockRead").await?)?;
    let clock = ClockInfo::from_params(&answer).map_err(bad_answer)?;
    let bridge_utc = now_utc();
    let offset =
        (clock.module_utc != 0).then(|| i64::from(clock.module_utc) - i64::from(bridge_utc));
    let mut body = serde_json::to_value(&clock).unwrap_or_default();
    body["bridge_utc"] = json!(bridge_utc);
    body["module_offset_s"] = json!(offset);
    Ok(HttpResponse::Ok().json(body))
}

/// Clock to set
#[derive(Deserialize, ToSchema)]
pub struct ClockBody {
    /// UTC seconds; the clock of the bridge when left out
    #[schema(example = 1757930400)]
    utc: Option<u32>,
}

/// Sets the clock of the module, which also sets the stove clock when the stove is connected.
/// Send `{}` to use the clock of the bridge (keep it synchronised with NTP).
#[utoipa::path(post, path = "/api/clock", request_body = ClockBody,
    responses((status = 200), (status = 400), (status = 403), (status = 503)), tag = "module")]
pub async fn post_clock(
    body: web::Json<ClockBody>,
    bridge: Shared,
    features: Features,
) -> Result<HttpResponse, ApiError> {
    require(features.clock_write, "clock_write")?;
    let utc = body.utc.unwrap_or_else(now_utc);
    if utc < MIN_UTC {
        return Err(ApiError::InvalidParameter(format!(
            "utc must be in seconds, after 2020-01-01 (got {})",
            utc
        )));
    }
    queue_command(
        &bridge,
        Command::Clk,
        CommandType::Write,
        vec![utc.to_string()],
        "Clock",
        utc.to_string(),
        true,
    )
}

/// Time zone of the module. `known` is false when the firmware has no rule for that name;
/// `available` lists the names accepted by `POST /api/timezone`.
#[utoipa::path(get, path = "/api/timezone",
    responses((status = 200), (status = 403), (status = 502), (status = 503), (status = 504)),
    tag = "module")]
pub async fn get_timezone(bridge: Shared, features: Features) -> Result<HttpResponse, ApiError> {
    require(features.timezone_read, "timezone_read")?;
    let status = read(&bridge, Command::Tmz, vec![], "TimeZoneRead").await?;
    let code = match status.status {
        RequestState::Ok => None,
        _ => status.error_code,
    };
    let (zone, known) = match time_zone_from_answer(code, &status.answer) {
        Ok(zone) => zone,
        Err(_) if status.status == RequestState::Error => return Err(stove_error(code)),
        Err(e) => return Err(bad_answer(e)),
    };
    Ok(HttpResponse::Ok()
        .json(json!({ "zone": zone, "known": known, "available": TIME_ZONES.as_slice() })))
}

/// Time zone to set
#[derive(Deserialize, ToSchema)]
pub struct TimeZoneBody {
    /// One of the names offered by AppFire (`UTC` or `Europe/...`)
    #[schema(example = "Europe/Paris")]
    zone: String,
}

/// Changes the time zone of the module; the module then sets the stove clock to the new local
/// time
#[utoipa::path(post, path = "/api/timezone", request_body = TimeZoneBody,
    responses((status = 200), (status = 400), (status = 403), (status = 503)), tag = "module")]
pub async fn post_timezone(
    body: web::Json<TimeZoneBody>,
    bridge: Shared,
    features: Features,
) -> Result<HttpResponse, ApiError> {
    require(features.timezone_write, "timezone_write")?;
    if !TIME_ZONES.contains(&body.zone.as_str()) {
        return Err(ApiError::InvalidParameter(format!(
            "unknown time zone '{}': expected one of {}",
            body.zone,
            TIME_ZONES.join(", ")
        )));
    }
    queue_command(
        &bridge,
        Command::Tmz,
        CommandType::Write,
        vec![quote(&body.zone)],
        "TimeZone",
        body.zone.clone(),
        true,
    )
}

/// Time span covered by the data logger of the module
#[utoipa::path(get, path = "/api/datalog/info",
    responses((status = 200, body = DatalogBounds), (status = 403), (status = 502), (status = 503), (status = 504)),
    tag = "module")]
pub async fn get_datalog_info(
    bridge: Shared,
    features: Features,
) -> Result<HttpResponse, ApiError> {
    require(features.datalog_read, "datalog_read")?;
    let answer = answer_ok(read(&bridge, Command::Me0, vec![], "DatalogInfo").await?)?;
    let bounds = DatalogBounds::from_params(&answer).map_err(bad_answer)?;
    Ok(HttpResponse::Ok().json(bounds))
}

/// Records to read
#[derive(Deserialize, IntoParams)]
pub struct DatalogQuery {
    /// UTC seconds of the first record (default: one hour ago)
    from: Option<u32>,
    /// Number of records, 1 to 100 (default 60)
    count: Option<u16>,
}

/// Records of the data logger from the first one at or after `from`, oldest first, up to the
/// newest. To read further, ask again from the `utc` of the last record plus one.
#[utoipa::path(get, path = "/api/datalog", params(DatalogQuery),
    responses((status = 200), (status = 400), (status = 403), (status = 502), (status = 503), (status = 504)),
    tag = "module")]
pub async fn get_datalog(
    query: web::Query<DatalogQuery>,
    bridge: Shared,
    features: Features,
) -> Result<HttpResponse, ApiError> {
    require(features.datalog_read, "datalog_read")?;
    let from = query.from.unwrap_or_else(|| now_utc().saturating_sub(3600));
    let count = query.count.unwrap_or(DATALOG_DEFAULT_COUNT);
    if !(1..=DATALOG_MAX_COUNT).contains(&count) {
        return Err(ApiError::InvalidParameter(format!(
            "count must be between 1 and {} (got {})",
            DATALOG_MAX_COUNT, count
        )));
    }
    let status = read(
        &bridge,
        Command::Met,
        vec![from.to_string(), count.to_string()],
        "Datalog",
    )
    .await?;
    // The firmware answers ERR;6 when no record is at or after `from`
    let records =
        if status.status == RequestState::Error && status.error_code == Some(DATALOG_NO_RECORD) {
            Vec::new()
        } else {
            DatalogRecord::list_from_params(&answer_ok(status)?).map_err(bad_answer)?
        };
    // Serialized directly: going through `json!` would print the f32 temperatures as f64
    // (22.100000381...)
    Ok(HttpResponse::Ok().json(DatalogResponse {
        from,
        count: records.len(),
        records,
    }))
}

/// `GET /api/datalog` body
#[derive(Serialize)]
struct DatalogResponse {
    from: u32,
    count: usize,
    records: Vec<DatalogRecord>,
}

/// Deletes every record of the data logger
#[utoipa::path(post, path = "/api/datalog/clear",
    responses((status = 200), (status = 403), (status = 503)), tag = "module")]
pub async fn post_datalog_clear(
    bridge: Shared,
    features: Features,
) -> Result<HttpResponse, ApiError> {
    require(features.datalog_clear, "datalog_clear")?;
    queue_command(
        &bridge,
        Command::Mec,
        CommandType::Execute,
        vec![],
        "DatalogClear",
        String::new(),
        true,
    )
}

/// Security PIN of the cloud relay, that AppFire uses in cloud mode. Anyone who knows it and
/// the module id can control the stove through the HottoH cloud.
#[utoipa::path(get, path = "/api/pin",
    responses((status = 200), (status = 403), (status = 502), (status = 503), (status = 504)),
    tag = "module")]
pub async fn get_pin(bridge: Shared, features: Features) -> Result<HttpResponse, ApiError> {
    require(features.pin_read, "pin_read")?;
    let answer = answer_ok(read(&bridge, Command::Pin, vec![], "PinRead").await?)?;
    let pin = pin_from_params(&answer).map_err(bad_answer)?;
    info!("Cloud relay PIN read through the API");
    Ok(HttpResponse::Ok().json(json!({ "pin": pin })))
}

/// New PIN
#[derive(Deserialize, ToSchema)]
pub struct PinBody {
    /// 5 to 10 letters or digits
    #[schema(example = "a1b2c3")]
    pin: String,
}

/// Changes the security PIN of the cloud relay. AppFire in cloud mode keeps the old PIN and
/// must be paired again.
#[utoipa::path(post, path = "/api/pin", request_body = PinBody,
    responses((status = 200), (status = 400), (status = 403), (status = 503)), tag = "module")]
pub async fn post_pin(
    body: web::Json<PinBody>,
    bridge: Shared,
    features: Features,
) -> Result<HttpResponse, ApiError> {
    require(features.pin_write, "pin_write")?;
    if !(5..=10).contains(&body.pin.len()) || !body.pin.bytes().all(|b| b.is_ascii_alphanumeric()) {
        return Err(ApiError::InvalidParameter(
            "pin must have 5 to 10 letters or digits".into(),
        ));
    }
    queue_command(
        &bridge,
        Command::Pin,
        CommandType::Write,
        vec![quote(&body.pin)],
        "Pin",
        "[redacted]".into(),
        true,
    )
}

/// Restarts the Wi-Fi module. It does not answer: the request is `ok` once sent, and the data
/// is unavailable for about 10 to 30 s.
#[utoipa::path(post, path = "/api/module/restart",
    responses((status = 200), (status = 403), (status = 503)), tag = "module")]
pub async fn post_module_restart(
    bridge: Shared,
    features: Features,
) -> Result<HttpResponse, ApiError> {
    require(features.module_restart, "module_restart")?;
    queue_command(
        &bridge,
        Command::Rst,
        CommandType::Execute,
        vec![],
        "ModuleRestart",
        String::new(),
        false,
    )
}

/// Networks seen by the Wi-Fi module, strongest first. The module does not tell which one it
/// uses, nor its Wi-Fi password (no command reads them back). The stove link is suspended during
/// the scan (a few seconds).
#[utoipa::path(get, path = "/api/wifi/scan",
    responses((status = 200, body = Vec<WifiNetwork>), (status = 403), (status = 502), (status = 503), (status = 504)),
    tag = "module")]
pub async fn get_wifi_scan(bridge: Shared, features: Features) -> Result<HttpResponse, ApiError> {
    require(features.wifi_scan, "wifi_scan")?;
    let answer = answer_ok(read(&bridge, Command::Scn, vec![], "WifiScan").await?)?;
    let mut networks = WifiNetwork::list_from_params(&answer).map_err(bad_answer)?;
    networks.sort_by_key(|n| std::cmp::Reverse(n.rssi));
    Ok(HttpResponse::Ok().json(networks))
}

/// `GET /api/cloud` body
#[derive(Serialize)]
struct CloudResponse {
    /// Balancer of the HottoH relay, through which AppFire reaches the stove in cloud mode
    relay_balancer: Server,
    /// 4-noks cloud the module used to upload its data logger to (no longer done by 10.5.0)
    cloud_server: CloudServer,
    cloud_last_upload_utc: u32,
    cloud_last_upload_time: Option<String>,
}

/// Cloud servers configured in the module
#[utoipa::path(get, path = "/api/cloud",
    responses((status = 200), (status = 403), (status = 502), (status = 503), (status = 504)),
    tag = "module")]
pub async fn get_cloud(bridge: Shared, features: Features) -> Result<HttpResponse, ApiError> {
    require(features.cloud_read, "cloud_read")?;
    let balancer = answer_ok(read(&bridge, Command::Bal, vec![], "CloudBalancer").await?)?;
    let server = answer_ok(read(&bridge, Command::Clu, vec![], "CloudServer").await?)?;
    let upload = answer_ok(read(&bridge, Command::Crw, vec![], "CloudLastUpload").await?)?;
    let cloud_last_upload_utc = cloud_last_upload_from_params(&upload).map_err(bad_answer)?;
    Ok(HttpResponse::Ok().json(CloudResponse {
        relay_balancer: balancer_from_params(&balancer).map_err(bad_answer)?,
        cloud_server: cloud_server_from_params(&server).map_err(bad_answer)?,
        cloud_last_upload_utc,
        cloud_last_upload_time: local_time(cloud_last_upload_utc),
    }))
}

/// A firmware check stays valid this long, unless `refresh=true`
const FIRMWARE_CHECK_TTL: Duration = Duration::from_secs(6 * 3600);

/// Last firmware check: installed version it was made for, time, answer
static FIRMWARE_CHECK: Mutex<Option<(String, Instant, serde_json::Value)>> = Mutex::new(None);

#[derive(Deserialize, IntoParams)]
pub struct FirmwareQuery {
    /// Ask the update server again instead of using the last check (kept 6 hours)
    refresh: Option<bool>,
}

/// Installed module firmware and newest version published on the HottoH update server
#[utoipa::path(get, path = "/api/firmware", params(FirmwareQuery),
    responses((status = 200), (status = 403), (status = 502), (status = 503)), tag = "module")]
pub async fn get_firmware(
    query: web::Query<FirmwareQuery>,
    bridge: Shared,
    features: Features,
) -> Result<HttpResponse, ApiError> {
    require(features.firmware_update_check, "firmware_update_check")?;
    let installed_text = bridge
        .state()
        .inf_if_received()
        .map(|inf| inf.version.clone())
        .ok_or_else(|| {
            ApiError::StoveTimeout("module version not received yet (see /api/status)".into())
        })?;
    let installed = Version::parse(&installed_text).ok_or_else(|| {
        ApiError::BadAnswer(format!("unexpected firmware version '{}'", installed_text))
    })?;
    if !query.refresh.unwrap_or(false)
        && let Some((version, at, body)) = FIRMWARE_CHECK
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .as_ref()
        && *version == installed_text
        && at.elapsed() < FIRMWARE_CHECK_TTL
    {
        return Ok(HttpResponse::Ok().json(body));
    }
    let latest = web::block(move || latest_firmware(installed, firmware_exists))
        .await
        .map_err(|e| ApiError::BadAnswer(e.to_string()))?
        .map_err(|e| ApiError::BadAnswer(format!("update check failed: {}", e)))?;
    let update_available = latest > installed;
    let body = json!({
        "installed": installed.to_string(),
        "latest_available": latest.to_string(),
        "update_available": update_available,
        "download_url": update_available.then(|| firmware_url(latest)),
        "source": UPDATE_HOST,
        "checked_at": crate::hottoh::hottoh_structs::now(),
    });
    info!(
        "Firmware check: installed {}, latest published {}",
        installed, latest
    );
    *FIRMWARE_CHECK.lock().unwrap_or_else(|p| p.into_inner()) =
        Some((installed_text, Instant::now(), body.clone()));
    Ok(HttpResponse::Ok().json(body))
}

/// Routes of the module features
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/api/features", web::get().to(get_features))
        .route("/api/chrono/schedule", web::get().to(get_schedule))
        .route("/api/chrono/schedule", web::post().to(post_schedule))
        .route("/api/clock", web::get().to(get_clock))
        .route("/api/clock", web::post().to(post_clock))
        .route("/api/timezone", web::get().to(get_timezone))
        .route("/api/timezone", web::post().to(post_timezone))
        .route("/api/datalog/info", web::get().to(get_datalog_info))
        .route("/api/datalog", web::get().to(get_datalog))
        .route("/api/datalog/clear", web::post().to(post_datalog_clear))
        .route("/api/pin", web::get().to(get_pin))
        .route("/api/pin", web::post().to(post_pin))
        .route("/api/module/restart", web::post().to(post_module_restart))
        .route("/api/wifi/scan", web::get().to(get_wifi_scan))
        .route("/api/cloud", web::get().to(get_cloud))
        .route("/api/firmware", web::get().to(get_firmware));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hottoh::http_api::configure as configure_all;
    use actix_web::App;
    use actix_web::http::StatusCode;
    use actix_web::test as atest;
    use serde_json::Value;
    use std::sync::Arc;

    fn all_enabled() -> FeaturesConfig {
        FeaturesConfig {
            clock_write: true,
            timezone_write: true,
            datalog_clear: true,
            pin_write: true,
            module_restart: true,
            wifi_scan: true,
            ..FeaturesConfig::default()
        }
    }

    async fn call(
        bridge: &Arc<Bridge>,
        features: FeaturesConfig,
        request: atest::TestRequest,
    ) -> (StatusCode, Value) {
        let app = atest::init_service(
            App::new()
                .app_data(web::Data::from(Arc::clone(bridge)))
                .app_data(web::Data::new(features))
                .configure(configure_all),
        )
        .await;
        let response = atest::call_service(&app, request.to_request()).await;
        let status = response.status();
        (status, atest::read_body_json(response).await)
    }

    async fn post(
        bridge: &Arc<Bridge>,
        features: FeaturesConfig,
        path: &str,
        body: Value,
    ) -> (StatusCode, Value) {
        call(
            bridge,
            features,
            atest::TestRequest::post().uri(path).set_json(body),
        )
        .await
    }

    #[actix_web::test]
    async fn risky_features_are_disabled_by_default() {
        let bridge = Arc::new(Bridge::new());
        for (path, body) in [
            ("/api/clock", json!({})),
            ("/api/timezone", json!({"zone": "Europe/Paris"})),
            ("/api/datalog/clear", json!({})),
            ("/api/pin", json!({"pin": "12345"})),
            ("/api/module/restart", json!({})),
        ] {
            let (status, response) = post(&bridge, FeaturesConfig::default(), path, body).await;
            assert_eq!(status, StatusCode::FORBIDDEN, "{}", path);
            assert!(response["error"].as_str().unwrap().contains("[features]"));
        }
        let disabled = FeaturesConfig {
            pin_read: false,
            ..FeaturesConfig::default()
        };
        let (status, _) = call(&bridge, disabled, atest::TestRequest::get().uri("/api/pin")).await;
        assert_eq!(status, StatusCode::FORBIDDEN);
        let (status, _) = call(
            &bridge,
            FeaturesConfig::default(),
            atest::TestRequest::get().uri("/api/wifi/scan"),
        )
        .await;
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert!(bridge.queue().is_empty());

        let (status, body) = call(
            &bridge,
            FeaturesConfig::default(),
            atest::TestRequest::get().uri("/api/features"),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["chrono_schedule_read"], true);
        assert_eq!(body["pin_read"], true);
        assert_eq!(body["pin_write"], false);
    }

    #[actix_web::test]
    async fn enabled_commands_are_queued_with_firmware_parameters() {
        let bridge = Arc::new(Bridge::new());
        let features = all_enabled();
        for (path, body) in [
            ("/api/clock", json!({"utc": 1757930400})),
            ("/api/timezone", json!({"zone": "Europe/Paris"})),
            ("/api/pin", json!({"pin": "abc123"})),
            ("/api/datalog/clear", json!({})),
            ("/api/module/restart", json!({})),
        ] {
            let (status, response) = post(&bridge, features.clone(), path, body).await;
            assert_eq!(status, StatusCode::OK, "{} {}", path, response);
            assert!(response["request_id"].is_u64());
            assert!(!response.to_string().contains("abc123"));
        }
        let queue = bridge.queue();
        let sent: Vec<_> = queue
            .iter()
            .map(|q| {
                (
                    q.request.get_command(),
                    q.request.get_command_type(),
                    q.request.get_params().to_vec(),
                    q.expects_answer,
                )
            })
            .collect();
        assert_eq!(
            sent,
            vec![
                (
                    Command::Clk,
                    CommandType::Write,
                    vec!["1757930400".to_string()],
                    true
                ),
                (
                    Command::Tmz,
                    CommandType::Write,
                    vec![r#"\"Europe/Paris\""#.to_string()],
                    true
                ),
                (
                    Command::Pin,
                    CommandType::Write,
                    vec![r#"\"abc123\""#.to_string()],
                    true
                ),
                (Command::Mec, CommandType::Execute, vec![], true),
                (Command::Rst, CommandType::Execute, vec![], false),
            ]
        );
        let pin_id = queue[2].request.get_req_id();
        drop(queue);
        assert_eq!(
            bridge.state().get_request(pin_id).unwrap().value,
            "[redacted]"
        );
    }

    #[actix_web::test]
    async fn invalid_bodies_are_refused() {
        let bridge = Arc::new(Bridge::new());
        let features = all_enabled();
        let full_day = |day: &str| json!({"day": day, "slots": vec![0; 48]});
        for (path, body) in [
            ("/api/clock", json!({"utc": 1757930400000u64})),
            ("/api/clock", json!({"utc": 12})),
            ("/api/timezone", json!({"zone": "Mars/Olympus"})),
            ("/api/pin", json!({"pin": "1234"})),
            ("/api/pin", json!({"pin": "12345;ERR"})),
            ("/api/chrono/schedule", json!({"days": []})),
            (
                "/api/chrono/schedule",
                json!({"days": [{"day": "funday", "slots": vec![0; 48]}]}),
            ),
            (
                "/api/chrono/schedule",
                json!({"days": [full_day("monday"), full_day("Monday")]}),
            ),
            (
                "/api/chrono/schedule",
                json!({"days": [{"day": "monday", "slots": vec![0; 47]}]}),
            ),
            (
                "/api/chrono/schedule",
                json!({"days": [{"day": "monday", "slots": vec![4; 48]}]}),
            ),
            ("/api/chrono/schedule", json!({"days": [{"day": "monday"}]})),
            (
                "/api/chrono/schedule",
                json!({"days": [{"day": "monday", "ranges": [{"start": "07:00", "end": "06:00", "program": 1}]}]}),
            ),
        ] {
            let (status, response) = post(&bridge, features.clone(), path, body.clone()).await;
            assert_eq!(
                status,
                StatusCode::BAD_REQUEST,
                "{} {} {}",
                path,
                body,
                response
            );
        }
        let (status, _) = call(
            &bridge,
            features,
            atest::TestRequest::get().uri("/api/datalog?count=101"),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(bridge.queue().is_empty());
    }

    #[actix_web::test]
    async fn full_week_is_written_without_reading() {
        let bridge = Arc::new(Bridge::new());
        let mut days: Vec<Value> = DAYS
            .iter()
            .map(|day| json!({"day": day, "slots": vec![0; 48]}))
            .collect();
        days[1] =
            json!({"day": "monday", "ranges": [{"start": "06:30", "end": "08:00", "program": 2}]});
        let (status, response) = post(
            &bridge,
            FeaturesConfig::default(),
            "/api/chrono/schedule",
            json!({ "days": days }),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{}", response);
        let queue = bridge.queue();
        assert_eq!(queue.len(), 1);
        let params = queue[0].request.get_params();
        assert_eq!(params.len(), 337);
        let monday = &params[1 + SLOTS_PER_DAY..1 + 2 * SLOTS_PER_DAY];
        assert_eq!(monday[12], "0");
        assert!(monday[13..16].iter().all(|p| p == "2"));
        assert_eq!(monday[16], "0");
    }

    /// Outcome, error code and answer parameters of a fake read
    type FakeAnswer = (RequestState, Option<i32>, Vec<String>);

    /// Answers the reads queued on the bridge like the stove would, from a separate thread
    fn fake_answers(bridge: Arc<Bridge>, answer: fn(Command) -> FakeAnswer) {
        std::thread::spawn(move || {
            for _ in 0..400 {
                let next = bridge.queue().pop_front();
                if let Some(queued) = next {
                    let id = queued.request.get_req_id();
                    let (state, code, params) = answer(queued.request.get_command());
                    let mut shared = bridge.state_mut();
                    shared.set_answer(id, params);
                    shared.update_request(id, state, code, None);
                }
                std::thread::sleep(Duration::from_millis(10));
            }
        });
    }

    fn fields(s: &str) -> Vec<String> {
        s.split(';').map(str::to_string).collect()
    }

    #[actix_web::test]
    async fn reads_are_decoded() {
        let bridge = Arc::new(Bridge::new());
        fake_answers(Arc::clone(&bridge), |command| match command {
            Command::Clk => (RequestState::Ok, None, fields("1757930400;1757937600")),
            Command::Tmz => (
                RequestState::Error,
                Some(8),
                fields(r#"ERR;8;\"Europe/Nowhere\""#),
            ),
            Command::Me0 => (RequestState::Ok, None, fields("1757000000;1757930400")),
            Command::Met => (
                RequestState::Ok,
                None,
                fields("1757930400;3;221;400;1450;8"),
            ),
            Command::Pin => (RequestState::Error, Some(16), fields("ERR;16")),
            Command::Scn => (
                RequestState::Ok,
                None,
                fields(
                    r#"AA:00:00:00:00:01;\"weak\";-80;WPA2-PSK;AA:00:00:00:00:02;\"strong\";-40;WPA2-PSK"#,
                ),
            ),
            Command::Bal => (
                RequestState::Ok,
                None,
                fields(r#"\"app1.hottoh.it\";50612"#),
            ),
            Command::Clu => (RequestState::Ok, None, fields(r#"\"http://x\";\"/p\";80"#)),
            Command::Crw => (RequestState::Ok, None, fields("0")),
            _ => (RequestState::Timeout, None, vec![]),
        });
        let features = all_enabled();
        let get = |path: &'static str| {
            let bridge = Arc::clone(&bridge);
            let features = features.clone();
            async move { call(&bridge, features, atest::TestRequest::get().uri(path)).await }
        };

        let (status, body) = get("/api/clock").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["stove_time"], "2025-09-15T12:00:00");
        assert!(body["module_offset_s"].is_i64());

        let (status, body) = get("/api/timezone").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["zone"], "Europe/Nowhere");
        assert_eq!(body["known"], false);
        assert_eq!(body["available"][0], "UTC");

        let (status, body) = get("/api/datalog/info").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["last_utc"], 1757930400);

        let (status, body) = get("/api/datalog?from=1757930000&count=5").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["count"], 1);
        assert_eq!(body["records"][0]["smoke_temp"], 145.0);
        assert_eq!(body["records"][0]["room_temp"], 22.1);
        assert_eq!(body["records"][0]["state"], "Power");

        let (status, body) = get("/api/pin").await;
        assert_eq!(status, StatusCode::BAD_GATEWAY);
        assert_eq!(body["error_code"], 16);

        let (status, body) = get("/api/wifi/scan").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body[0]["ssid"], "strong");

        let (status, body) = get("/api/cloud").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["relay_balancer"]["port"], 50612);
        assert!(body["cloud_last_upload_time"].is_null());
    }
}
