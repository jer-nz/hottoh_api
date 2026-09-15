use crate::hottoh::config::{FeaturesConfig, HttpApiConfig, WebUiConfig};
use crate::hottoh::hottoh_const::{ChronoMode, StoveCommands};
use crate::hottoh::hottoh_structs::{DAT0Data, DAT1Data};
use crate::hottoh::http_module;
use crate::hottoh::shared_struct::{Bridge, ConnectionStatus, MAX_QUEUED};
use crate::hottoh::stats::{ProcessInfo, process_info};
use crate::hottoh::web_ui;
use actix_web::dev::Server;
use actix_web::{App, HttpResponse, HttpServer, ResponseError, middleware, web};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::ops::RangeInclusive;
use std::sync::Arc;
use thiserror::Error;
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

/// Firmware limits for temperatures, in tenths of °C
const FIRMWARE_TEMP_RANGE: RangeInclusive<i32> = 0..=9999;
/// Firmware limits for the power level
const FIRMWARE_POWER_RANGE: RangeInclusive<i32> = 0..=100;
/// Firmware limits for fan speeds
const FIRMWARE_FAN_RANGE: RangeInclusive<i32> = 0..=101;

/// API Error
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Too many requests waiting for the stove ({0}), retry later")]
    QueueFull(usize),
    #[error("Feature '{0}' is disabled: set {0} = true in the [features] section of config.ini")]
    FeatureDisabled(&'static str),
    #[error("Stove refused the request: {message}")]
    Stove { code: Option<i32>, message: String },
    #[error("Invalid answer from the stove: {0}")]
    BadAnswer(String),
    #[error("No answer from the stove: {0}")]
    StoveTimeout(String),
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        let body = json!({ "success": false, "error": self.to_string() });
        match self {
            ApiError::InvalidParameter(_) => {
                warn!("{}", self);
                HttpResponse::BadRequest().json(body)
            }
            ApiError::NotFound(_) => HttpResponse::NotFound().json(body),
            ApiError::QueueFull(_) => {
                warn!("{}", self);
                HttpResponse::ServiceUnavailable().json(body)
            }
            ApiError::FeatureDisabled(_) => HttpResponse::Forbidden().json(body),
            ApiError::Stove { code, .. } => HttpResponse::BadGateway().json(json!({
                "success": false,
                "error": self.to_string(),
                "error_code": code,
            })),
            ApiError::BadAnswer(_) => {
                warn!("{}", self);
                HttpResponse::BadGateway().json(body)
            }
            ApiError::StoveTimeout(_) => HttpResponse::GatewayTimeout().json(body),
        }
    }
}

type Shared = web::Data<Bridge>;

/// Stove setting and how to read its limits from a DAT page
type Target<Page, Limits> = (StoveCommands, fn(&Page) -> Limits);

#[derive(OpenApi)]
#[openapi(
    info(title = "HottoH API", version = env!("CARGO_PKG_VERSION")),
    paths(
        get_inf,
        get_dat0,
        get_dat1,
        get_dat2,
        get_status,
        get_request_status,
        get_requests,
        get_alarms,
        post_on_off,
        post_eco_mode,
        post_ambiance_temp,
        post_chrono_mode,
        post_chrono_temp,
        post_fan_speed,
        post_power_level
    ),
    components(schemas(DatPostBool, DatPostU32, DatPostAmbianceTemp, DatPostFanSpeed, DatPostChronoTemp)),
    tags((name = "hottoh", description = "Stove control API"))
)]
struct ApiDoc;

/// Boolean parameter
#[derive(Deserialize, ToSchema)]
struct DatPostBool {
    #[schema(example = true)]
    value: bool,
}

/// Integer parameter
#[derive(Deserialize, ToSchema)]
struct DatPostU32 {
    #[schema(example = 3)]
    value: u32,
}

/// Ambiance temperature
#[derive(Deserialize, ToSchema)]
struct DatPostAmbianceTemp {
    /// Ambiance number (1 or 2)
    #[schema(example = 1)]
    ambiance: u32,
    /// Temperature in °C (0.1 °C resolution)
    #[schema(example = 21.5)]
    value: f32,
}

/// Fan speed
#[derive(Deserialize, ToSchema)]
struct DatPostFanSpeed {
    /// Fan number (1 to 3)
    #[schema(example = 1)]
    fan: u32,
    /// Speed, from 0 to the maximum reported by the stove (`index_fan_N_set_max`)
    #[schema(example = 3)]
    value: u32,
}

/// Chrono program temperature
#[derive(Deserialize, ToSchema)]
struct DatPostChronoTemp {
    /// Program number (1 to 3)
    #[schema(example = 1)]
    chrono: u32,
    /// Temperature in °C (0.1 °C resolution)
    #[schema(example = 20.0)]
    value: f32,
}

/// `GET /api/status` body
#[derive(Serialize)]
struct StatusResponse<'a> {
    #[serde(flatten)]
    connection: &'a ConnectionStatus,
    pending_writes: usize,
    started_at: &'a str,
    uptime_s: u64,
    version: &'static str,
    process: ProcessInfo,
}

/// Returns the stove range when it is known and consistent, the firmware range otherwise
fn effective_range(
    stove: Option<(i32, i32)>,
    firmware: RangeInclusive<i32>,
) -> RangeInclusive<i32> {
    match stove {
        Some((min, max)) if max > 0 && min <= max => {
            min.max(*firmware.start())..=max.min(*firmware.end())
        }
        _ => firmware,
    }
}

fn check_range(name: &str, value: i32, range: &RangeInclusive<i32>) -> Result<(), ApiError> {
    if range.contains(&value) {
        Ok(())
    } else {
        Err(ApiError::InvalidParameter(format!(
            "{} must be between {} and {} (got {})",
            name,
            range.start(),
            range.end(),
            value
        )))
    }
}

fn tenths(value: f32) -> i32 {
    (value * 10.0).round() as i32
}

/// Converts °C to the tenths sent to the stove (rounded to the nearest tenth), checked against
/// `range` (tenths) with the error in °C
fn to_tenths(value: f32, range: &RangeInclusive<i32>) -> Result<i32, ApiError> {
    if !value.is_finite() {
        return Err(ApiError::InvalidParameter(
            "temperature must be a finite number".into(),
        ));
    }
    let rounded = tenths(value);
    if !range.contains(&rounded) {
        return Err(ApiError::InvalidParameter(format!(
            "temperature must be between {:.1} and {:.1} °C (got {})",
            f64::from(*range.start()) / 10.0,
            f64::from(*range.end()) / 10.0,
            value
        )));
    }
    Ok(rounded)
}

/// Stove limits in tenths of °C
fn tenths_range((min, max): (f32, f32)) -> (i32, i32) {
    (tenths(min), tenths(max))
}

/// Selects the entry for a 1-based number (ambiance, chrono program, fan)
fn pick<T: Copy>(name: &str, number: u32, choices: &[T]) -> Result<T, ApiError> {
    usize::try_from(number)
        .ok()
        .and_then(|n| n.checked_sub(1))
        .and_then(|index| choices.get(index))
        .copied()
        .ok_or_else(|| {
            ApiError::InvalidParameter(format!(
                "{} must be between 1 and {} (got {})",
                name,
                choices.len(),
                number
            ))
        })
}

/// Checks the value against the stove range (the firmware range until the stove has answered),
/// then queues the write. The outcome is available on `/api/request/{id}`.
fn queue_write(
    bridge: &Bridge,
    command: StoveCommands,
    label: &str,
    value: i32,
    stove_range: Option<(i32, i32)>,
    firmware_range: RangeInclusive<i32>,
) -> Result<HttpResponse, ApiError> {
    check_range(label, value, &effective_range(stove_range, firmware_range))?;
    let request_id = bridge
        .queue_write(command, value)
        .ok_or(ApiError::QueueFull(MAX_QUEUED))?;
    Ok(queued_response(
        request_id,
        command.name(),
        &value.to_string(),
    ))
}

/// Answer to a queued write: its id, to follow on `/api/request/{id}`
pub(crate) fn queued_response(request_id: u32, label: &str, shown_value: &str) -> HttpResponse {
    let message = format!(
        "Request added for command: {}, value: {}, id: {}",
        label, shown_value, request_id
    );
    info!("{}", message);
    HttpResponse::Ok().json(json!({
        "success": true,
        "message": message,
        "request_id": request_id,
        "status_url": format!("/api/request/{}", request_id),
    }))
}

/// Module information
#[utoipa::path(get, path = "/api/inf", responses((status = 200)), tag = "hottoh")]
async fn get_inf(bridge: Shared) -> HttpResponse {
    HttpResponse::Ok().json(bridge.state().get_inf())
}

/// DAT page 0: main stove data
#[utoipa::path(get, path = "/api/dat/0", responses((status = 200)), tag = "hottoh")]
async fn get_dat0(bridge: Shared) -> HttpResponse {
    HttpResponse::Ok().json(bridge.state().get_dat0())
}

/// DAT page 1: chrono programs
#[utoipa::path(get, path = "/api/dat/1", responses((status = 200)), tag = "hottoh")]
async fn get_dat1(bridge: Shared) -> HttpResponse {
    HttpResponse::Ok().json(bridge.state().get_dat1())
}

/// DAT page 2: hydraulic data and actual fan speeds
#[utoipa::path(get, path = "/api/dat/2", responses((status = 200)), tag = "hottoh")]
async fn get_dat2(bridge: Shared) -> HttpResponse {
    HttpResponse::Ok().json(bridge.state().get_dat2())
}

/// Connection with the stove, counters since start and process resources
#[utoipa::path(get, path = "/api/status", responses((status = 200)), tag = "hottoh")]
async fn get_status(bridge: Shared) -> HttpResponse {
    let pending_writes = bridge.queue().len();
    let state = bridge.state();
    HttpResponse::Ok().json(StatusResponse {
        connection: state.connection(),
        pending_writes,
        started_at: bridge.started_at(),
        uptime_s: bridge.uptime().as_secs(),
        version: env!("CARGO_PKG_VERSION"),
        process: process_info(),
    })
}

/// Outcome of a queued request: pending, sent, ok, error (with the stove error code) or timeout
#[utoipa::path(
    get,
    path = "/api/request/{id}",
    params(("id" = u32, Path, description = "request_id returned by a POST")),
    responses((status = 200), (status = 404, description = "Unknown or expired request")),
    tag = "hottoh"
)]
async fn get_request_status(bridge: Shared, id: web::Path<u32>) -> Result<HttpResponse, ApiError> {
    let id = id.into_inner();
    bridge
        .state()
        .get_request(id)
        .map(|status| HttpResponse::Ok().json(status))
        .ok_or_else(|| ApiError::NotFound(format!("request {}", id)))
}

/// Last queued requests (writes and reads made on demand), newest first
#[utoipa::path(get, path = "/api/requests", responses((status = 200)), tag = "hottoh")]
async fn get_requests(bridge: Shared) -> HttpResponse {
    HttpResponse::Ok().json(bridge.state().recent_requests())
}

/// Turns the stove on (`true`) or off (`false`)
#[utoipa::path(post, path = "/api/dat/set_on_off", request_body = DatPostBool,
    responses((status = 200), (status = 400), (status = 503)), tag = "hottoh")]
async fn post_on_off(
    body: web::Json<DatPostBool>,
    bridge: Shared,
) -> Result<HttpResponse, ApiError> {
    let value = i32::from(body.value);
    queue_write(&bridge, StoveCommands::OnOff, "value", value, None, 0..=1)
}

/// Activates or deactivates eco mode
#[utoipa::path(post, path = "/api/dat/set_eco_mode", request_body = DatPostBool,
    responses((status = 200), (status = 400), (status = 503)), tag = "hottoh")]
async fn post_eco_mode(
    body: web::Json<DatPostBool>,
    bridge: Shared,
) -> Result<HttpResponse, ApiError> {
    let value = i32::from(body.value);
    queue_write(&bridge, StoveCommands::EcoMode, "value", value, None, 0..=1)
}

/// Sets the ambiance temperature set point (°C), within the range reported by the stove
#[utoipa::path(post, path = "/api/dat/set_ambiance_temp", request_body = DatPostAmbianceTemp,
    responses((status = 200), (status = 400), (status = 503)), tag = "hottoh")]
async fn post_ambiance_temp(
    body: web::Json<DatPostAmbianceTemp>,
    bridge: Shared,
) -> Result<HttpResponse, ApiError> {
    let choices: [Target<DAT0Data, (f32, f32)>; 2] = [
        (StoveCommands::AmbianceTemperature1, |d| {
            (d.index_ambient_t1_set_min, d.index_ambient_t1_set_max)
        }),
        (StoveCommands::AmbianceTemperature2, |d| {
            (d.index_ambient_t2_set_min, d.index_ambient_t2_set_max)
        }),
    ];
    let (command, limits) = pick("ambiance", body.ambiance, &choices)?;
    let stove_range = bridge
        .state()
        .dat0_if_received()
        .map(|d| tenths_range(limits(d)));
    let range = effective_range(stove_range, FIRMWARE_TEMP_RANGE);
    let value = to_tenths(body.value, &range)?;
    queue_temperature(&bridge, command, value)
}

/// Queues a temperature already checked by `to_tenths`; the answer shows the value sent, in °C
fn queue_temperature(
    bridge: &Bridge,
    command: StoveCommands,
    tenths: i32,
) -> Result<HttpResponse, ApiError> {
    let request_id = bridge
        .queue_write(command, tenths)
        .ok_or(ApiError::QueueFull(MAX_QUEUED))?;
    Ok(queued_response(
        request_id,
        command.name(),
        &format!("{:.1} °C", f64::from(tenths) / 10.0),
    ))
}

/// Activates (`true`) or deactivates (`false`) chrono mode
#[utoipa::path(post, path = "/api/dat/set_chrono_mode", request_body = DatPostBool,
    responses((status = 200), (status = 400), (status = 503)), tag = "hottoh")]
async fn post_chrono_mode(
    body: web::Json<DatPostBool>,
    bridge: Shared,
) -> Result<HttpResponse, ApiError> {
    // The firmware expects 0 (manual) or 2 (chrono); 1 is refused with ERR;17.
    let value = ChronoMode::from_enabled(body.value) as i32;
    queue_write(
        &bridge,
        StoveCommands::ChronoOnOff,
        "value",
        value,
        None,
        0..=2,
    )
}

/// Sets the temperature of a chrono program (°C), within the range reported by the stove
#[utoipa::path(post, path = "/api/dat/set_chrono_temp", request_body = DatPostChronoTemp,
    responses((status = 200), (status = 400), (status = 503)), tag = "hottoh")]
async fn post_chrono_temp(
    body: web::Json<DatPostChronoTemp>,
    bridge: Shared,
) -> Result<HttpResponse, ApiError> {
    let choices: [Target<DAT1Data, (f32, f32)>; 3] = [
        (StoveCommands::ChronoTemperature1, |d| {
            (d.index_program_1_temp_min, d.index_program_1_temp_max)
        }),
        (StoveCommands::ChronoTemperature2, |d| {
            (d.index_program_2_temp_min, d.index_program_2_temp_max)
        }),
        (StoveCommands::ChronoTemperature3, |d| {
            (d.index_program_3_temp_min, d.index_program_3_temp_max)
        }),
    ];
    let (command, limits) = pick("chrono", body.chrono, &choices)?;
    let stove_range = bridge
        .state()
        .dat1_if_received()
        .map(|d| tenths_range(limits(d)));
    let range = effective_range(stove_range, FIRMWARE_TEMP_RANGE);
    let value = to_tenths(body.value, &range)?;
    queue_temperature(&bridge, command, value)
}

/// Sets a fan speed, from 0 to the maximum reported by the stove
#[utoipa::path(post, path = "/api/dat/set_fan_speed", request_body = DatPostFanSpeed,
    responses((status = 200), (status = 400), (status = 503)), tag = "hottoh")]
async fn post_fan_speed(
    body: web::Json<DatPostFanSpeed>,
    bridge: Shared,
) -> Result<HttpResponse, ApiError> {
    let choices: [Target<DAT0Data, u16>; 3] = [
        (StoveCommands::FanSpeed1, |d| d.index_fan_1_set_max),
        (StoveCommands::FanSpeed2, |d| d.index_fan_2_set_max),
        (StoveCommands::FanSpeed3, |d| d.index_fan_3_set_max),
    ];
    let (command, max) = pick("fan", body.fan, &choices)?;
    let value = i32::try_from(body.value).unwrap_or(i32::MAX);
    let stove_range = bridge
        .state()
        .dat0_if_received()
        .map(|d| (0, i32::from(max(d))));
    queue_write(
        &bridge,
        command,
        "fan speed",
        value,
        stove_range,
        FIRMWARE_FAN_RANGE,
    )
}

/// Sets the power level, within the range reported by the stove
#[utoipa::path(post, path = "/api/dat/set_power_level", request_body = DatPostU32,
    responses((status = 200), (status = 400), (status = 503)), tag = "hottoh")]
async fn post_power_level(
    body: web::Json<DatPostU32>,
    bridge: Shared,
) -> Result<HttpResponse, ApiError> {
    let value = i32::try_from(body.value).unwrap_or(i32::MAX);
    let stove_range = bridge
        .state()
        .dat0_if_received()
        .map(|d| (i32::from(d.index_power_min), i32::from(d.index_power_max)));
    queue_write(
        &bridge,
        StoveCommands::PowerLevel,
        "power level",
        value,
        stove_range,
        FIRMWARE_POWER_RANGE,
    )
}

/// Routes, JSON error handling and Swagger UI
pub(crate) fn configure(cfg: &mut web::ServiceConfig) {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(http_module::ModuleApiDoc::openapi());
    cfg.app_data(web::JsonConfig::default().error_handler(|err, _req| {
        let message = err.to_string();
        warn!("Invalid JSON body: {}", message);
        actix_web::error::InternalError::from_response(
            err,
            HttpResponse::BadRequest().json(json!({ "success": false, "error": message })),
        )
        .into()
    }))
    .service(SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", openapi))
    .route("/api/inf", web::get().to(get_inf))
    .route("/api/dat/0", web::get().to(get_dat0))
    .route("/api/dat/1", web::get().to(get_dat1))
    .route("/api/dat/2", web::get().to(get_dat2))
    .route("/api/status", web::get().to(get_status))
    .route("/api/request/{id}", web::get().to(get_request_status))
    .route("/api/requests", web::get().to(get_requests))
    .route("/api/alarms", web::get().to(get_alarms))
    .route("/api/dat/set_on_off", web::post().to(post_on_off))
    .route("/api/dat/set_eco_mode", web::post().to(post_eco_mode))
    .route(
        "/api/dat/set_ambiance_temp",
        web::post().to(post_ambiance_temp),
    )
    .route("/api/dat/set_chrono_mode", web::post().to(post_chrono_mode))
    .route("/api/dat/set_chrono_temp", web::post().to(post_chrono_temp))
    .route("/api/dat/set_fan_speed", web::post().to(post_fan_speed))
    .route("/api/dat/set_power_level", web::post().to(post_power_level))
    .configure(http_module::configure);
}

/// Binds and starts a server for the API, with the web interface when `with_ui` is set
fn api_server(
    address: &str,
    data: web::Data<Bridge>,
    features: web::Data<FeaturesConfig>,
    with_ui: bool,
) -> std::io::Result<Server> {
    Ok(HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .app_data(data.clone())
            .app_data(features.clone())
            .configure(configure)
            .configure(|cfg| {
                if with_ui {
                    web_ui::configure(cfg);
                }
            })
    })
    .shutdown_timeout(5)
    .bind(address)?
    .run())
}

/// Runs the HTTP server until SIGINT or SIGTERM (handled by actix-web), and the web interface on
/// its own address when `[web_ui]` gives one.
///
/// `desktop` (no configuration file): when the port is taken by another hottoh_api, its
/// interface is opened and this one stops; when it is taken by another program, the next ports
/// are tried.
pub async fn start_http_server(
    config: &HttpApiConfig,
    web_ui: &WebUiConfig,
    features: FeaturesConfig,
    bridge: Arc<Bridge>,
    desktop: bool,
) -> std::io::Result<()> {
    let ui_address = web_ui.separate_address(config);
    let ui_with_api = web_ui.enabled && ui_address.is_none();
    let open_browser = web_ui.enabled && web_ui.open_browser.unwrap_or(desktop);
    let data = web::Data::from(bridge);
    let features = web::Data::new(features);

    let mut port = config.port;
    let (api, http_address) = loop {
        let address = format!("{}:{}", config.ip, port);
        match api_server(&address, data.clone(), features.clone(), ui_with_api) {
            Ok(server) => break (server, address),
            Err(e) if desktop && e.kind() == std::io::ErrorKind::AddrInUse => {
                if is_hottoh_api(&local_url(&address)) {
                    info!(
                        "hottoh_api already runs on {}: opening its interface",
                        address
                    );
                    open_in_browser(&local_url(&address));
                    return Ok(());
                }
                if port >= config.port.saturating_add(10) {
                    return Err(e);
                }
                warn!(
                    "Port {} is used by another program, trying {}",
                    port,
                    port + 1
                );
                port += 1;
            }
            Err(e) => return Err(e),
        }
    };
    info!("HTTP server on {}", http_address);
    let Some(ui_address) = ui_address else {
        if ui_with_api {
            info!("Web interface on http://{}/", http_address);
            if open_browser {
                open_in_browser(&local_url(&http_address));
            }
        } else {
            info!("Web interface disabled");
        }
        return api.await;
    };
    // The API is served on the address of the interface too: same origin, no CORS
    let ui = api_server(&ui_address, data, features, true)?;
    info!("Web interface (with the API) on http://{}/", ui_address);
    if open_browser {
        open_in_browser(&local_url(&ui_address));
    }
    let ui_handle = ui.handle();
    let ui_task = actix_web::rt::spawn(ui);
    let result = api.await;
    ui_handle.stop(true).await;
    let ui_result = ui_task
        .await
        .unwrap_or_else(|e| Err(std::io::Error::other(e.to_string())));
    result.and(ui_result)
}

/// URL to open on this computer for a listening address (`0.0.0.0` becomes `127.0.0.1`)
fn local_url(address: &str) -> String {
    let (ip, port) = address.rsplit_once(':').unwrap_or((address, ""));
    let host = match ip {
        "0.0.0.0" | "::" | "[::]" => "127.0.0.1",
        other => other,
    };
    format!("http://{}:{}/", host, port)
}

/// Whether a hottoh_api answers at this URL (`GET /api/status`)
fn is_hottoh_api(url: &str) -> bool {
    use std::io::{Read, Write};
    let Some(authority) = url
        .strip_prefix("http://")
        .and_then(|rest| rest.split('/').next())
    else {
        return false;
    };
    let Some(address) = std::net::ToSocketAddrs::to_socket_addrs(authority)
        .ok()
        .and_then(|mut a| a.next())
    else {
        return false;
    };
    let timeout = std::time::Duration::from_secs(2);
    let Ok(mut stream) = std::net::TcpStream::connect_timeout(&address, timeout) else {
        return false;
    };
    let _ = stream.set_read_timeout(Some(timeout));
    let request = format!(
        "GET /api/status HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
        authority
    );
    if stream.write_all(request.as_bytes()).is_err() {
        return false;
    }
    let mut answer = String::new();
    let _ = stream.take(64 * 1024).read_to_string(&mut answer);
    answer.contains("\"stove_address\"")
}

/// Opens a URL in the default browser of the system
fn open_in_browser(url: &str) {
    let mut command = if cfg!(windows) {
        let mut c = std::process::Command::new("rundll32");
        c.args(["url.dll,FileProtocolHandler", url]);
        c
    } else if cfg!(target_os = "macos") {
        let mut c = std::process::Command::new("open");
        c.arg(url);
        c
    } else {
        let mut c = std::process::Command::new("xdg-open");
        c.arg(url);
        c
    };
    match command.spawn() {
        Ok(_) => info!("Opening {} in the browser", url),
        Err(e) => warn!("Cannot open the browser ({}): open {} yourself", e, url),
    }
}

/// Current and past alarms of the stove, newest first
#[utoipa::path(get, path = "/api/alarms", responses((status = 200)), tag = "hottoh")]
async fn get_alarms(bridge: Shared) -> HttpResponse {
    let state = bridge.state();
    HttpResponse::Ok().json(json!({
        "current": state.alarms().current(),
        "events": state.alarms().newest_first(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hottoh::shared_struct::RequestState;
    use actix_web::http::StatusCode;
    use actix_web::test as atest;
    use serde_json::Value;

    #[test]
    fn stove_range_is_used_when_known() {
        assert_eq!(effective_range(Some((1, 5)), FIRMWARE_POWER_RANGE), 1..=5);
        assert_eq!(effective_range(None, FIRMWARE_POWER_RANGE), 0..=100);
        // Default (never received) or inconsistent values fall back to the firmware range
        assert_eq!(effective_range(Some((0, 0)), FIRMWARE_FAN_RANGE), 0..=101);
        assert_eq!(effective_range(Some((9, 3)), FIRMWARE_FAN_RANGE), 0..=101);
        // Negative minimum is clamped: the firmware refuses negative temperatures
        assert_eq!(
            effective_range(Some((-50, 300)), FIRMWARE_TEMP_RANGE),
            0..=300
        );
    }

    #[test]
    fn temperatures_are_rounded_to_tenths() {
        let range = 50..=550;
        assert_eq!(to_tenths(21.3, &range).unwrap(), 213);
        assert_eq!(to_tenths(21.25, &range).unwrap(), 213);
        assert_eq!(to_tenths(19.94, &range).unwrap(), 199);
        assert!(to_tenths(f32::NAN, &range).is_err());
        // Limits are checked after rounding, and reported in °C
        assert_eq!(to_tenths(55.04, &range).unwrap(), 550);
        let error = to_tenths(55.5, &range).unwrap_err().to_string();
        assert_eq!(
            error,
            "Invalid parameter: temperature must be between 5.0 and 55.0 °C (got 55.5)"
        );
    }

    #[actix_web::test]
    async fn temperature_answer_shows_the_value_sent() {
        let bridge = bridge_with_dat0();
        let (status, body) = post(
            &bridge,
            "/api/dat/set_ambiance_temp",
            json!({"ambiance": 1, "value": 21.25}),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert!(
            body["message"].as_str().unwrap().contains("value: 21.3 °C"),
            "{}",
            body
        );
        assert_eq!(bridge.queue()[0].request.get_params(), ["3", "213"]);
    }

    #[test]
    fn local_urls() {
        assert_eq!(local_url("0.0.0.0:3000"), "http://127.0.0.1:3000/");
        assert_eq!(local_url("192.168.1.2:80"), "http://192.168.1.2:80/");
    }

    #[actix_web::test]
    async fn alarms_are_listed() {
        let bridge = bridge_with_dat0();
        let (status, body) = call(&bridge, atest::TestRequest::get().uri("/api/alarms")).await;
        assert_eq!(status, StatusCode::OK);
        assert!(body["current"].is_null());
        let raw = "0;9;0;1;33;60;1;0;2;215;220;50;300;-15;0;0;0;0;0;0;0;1450;\
                   3;3;1;5;1200;3;3;5;0;0;0;0;0;0";
        let fields: Vec<String> = raw.split(';').map(str::to_string).collect();
        assert!(
            bridge
                .state_mut()
                .set_dat0(DAT0Data::from_slice(&fields).unwrap())
        );
        let (_, body) = call(&bridge, atest::TestRequest::get().uri("/api/alarms")).await;
        assert_eq!(body["current"]["state"], "IgnitionFailed");
        assert_eq!(body["events"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn pick_is_one_based() {
        let choices = ['a', 'b'];
        assert_eq!(pick("x", 1, &choices).unwrap(), 'a');
        assert_eq!(pick("x", 2, &choices).unwrap(), 'b');
        assert!(pick("x", 0, &choices).is_err());
        assert!(pick("x", 3, &choices).is_err());
    }

    /// Bridge whose stove has already sent DAT0 (power 1..=5, fans up to 5)
    fn bridge_with_dat0() -> Arc<Bridge> {
        let bridge = Arc::new(Bridge::new());
        let raw = "0;9;0;1;33;8;1;0;2;215;220;50;300;-15;0;0;0;0;0;0;0;1450;\
                   3;3;1;5;1200;3;3;5;0;0;0;0;0;0";
        let fields: Vec<String> = raw.split(';').map(str::to_string).collect();
        bridge
            .state_mut()
            .set_dat0(DAT0Data::from_slice(&fields).unwrap());
        bridge
    }

    async fn call(bridge: &Arc<Bridge>, request: atest::TestRequest) -> (StatusCode, Value) {
        let app = atest::init_service(
            App::new()
                .app_data(web::Data::from(Arc::clone(bridge)))
                .configure(configure),
        )
        .await;
        let response = atest::call_service(&app, request.to_request()).await;
        let status = response.status();
        (status, atest::read_body_json(response).await)
    }

    async fn post(bridge: &Arc<Bridge>, path: &str, body: Value) -> (StatusCode, Value) {
        call(bridge, atest::TestRequest::post().uri(path).set_json(body)).await
    }

    #[actix_web::test]
    async fn write_is_queued_and_tracked() {
        let bridge = bridge_with_dat0();
        let (status, body) = post(&bridge, "/api/dat/set_power_level", json!({"value": 3})).await;
        assert_eq!(status, StatusCode::OK);
        let id = body["request_id"].as_u64().unwrap();
        assert_eq!(bridge.queue()[0].request.get_params(), ["2", "3"]);

        let (status, body) = call(
            &bridge,
            atest::TestRequest::get().uri(&format!("/api/request/{}", id)),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"], "pending");
        assert_eq!(
            bridge.state().get_request(id as u32).unwrap().status,
            RequestState::Pending
        );
    }

    #[actix_web::test]
    async fn invalid_writes_are_refused_before_the_queue() {
        let bridge = bridge_with_dat0();
        for (path, body) in [
            ("/api/dat/set_power_level", json!({"value": 6})),
            ("/api/dat/set_fan_speed", json!({"fan": 4, "value": 1})),
            ("/api/dat/set_fan_speed", json!({"fan": 1, "value": 6})),
            (
                "/api/dat/set_ambiance_temp",
                json!({"ambiance": 1, "value": 30.1}),
            ),
            (
                "/api/dat/set_ambiance_temp",
                json!({"ambiance": 3, "value": 20}),
            ),
            (
                "/api/dat/set_chrono_temp",
                json!({"chrono": 0, "value": 20}),
            ),
            ("/api/dat/set_power_level", json!({"value": -1})),
            ("/api/dat/set_on_off", json!({"value": "yes"})),
            ("/api/dat/set_eco_mode", json!({})),
        ] {
            let (status, response) = post(&bridge, path, body.clone()).await;
            assert_eq!(status, StatusCode::BAD_REQUEST, "{} {}", path, body);
            assert_eq!(response["success"], false);
        }
        assert!(bridge.queue().is_empty());
    }

    #[actix_web::test]
    async fn chrono_mode_sends_firmware_values() {
        let bridge = Arc::new(Bridge::new());
        post(&bridge, "/api/dat/set_chrono_mode", json!({"value": true})).await;
        post(&bridge, "/api/dat/set_chrono_mode", json!({"value": false})).await;
        let writes = bridge.queue();
        assert_eq!(writes[0].request.get_params(), ["8", "2"]);
        assert_eq!(writes[1].request.get_params(), ["8", "0"]);
    }

    #[actix_web::test]
    async fn full_queue_answers_503() {
        let bridge = Arc::new(Bridge::new());
        for _ in 0..MAX_QUEUED {
            bridge.queue_write(StoveCommands::EcoMode, 1).unwrap();
        }
        let (status, body) = post(&bridge, "/api/dat/set_eco_mode", json!({"value": true})).await;
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(body["success"], false);
    }

    #[actix_web::test]
    async fn status_and_unknown_request() {
        let bridge = Arc::new(Bridge::new());
        let (status, body) = call(&bridge, atest::TestRequest::get().uri("/api/status")).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["connected"], false);
        assert_eq!(body["pending_writes"], 0);
        assert!(body["stats"]["timeouts"].is_u64());
        assert!(body["uptime_s"].is_u64());

        let (status, _) = call(&bridge, atest::TestRequest::get().uri("/api/request/42")).await;
        assert_eq!(status, StatusCode::NOT_FOUND);

        bridge.queue_write(StoveCommands::EcoMode, 1).unwrap();
        bridge.queue_write(StoveCommands::PowerLevel, 3).unwrap();
        let (status, body) = call(&bridge, atest::TestRequest::get().uri("/api/requests")).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body.as_array().unwrap().len(), 2);
        assert_eq!(body[0]["command"], "PowerLevel");
        assert_eq!(body[1]["status"], "pending");
    }
}
