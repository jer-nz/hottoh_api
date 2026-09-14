use crate::hottoh::config::AppConfig;
use crate::hottoh::hottoh_const::{ChronoMode, Command, CommandType, StoveCommands};
use crate::hottoh::shared_struct::SharedState;
use crate::hottoh::tcp_client::{QueuedWrite, WriteQueue, state_write};
use crate::hottoh::tcp_client_structs::Request;
use actix_web::{App, HttpResponse, HttpServer, ResponseError, middleware, web};
use log::{error, info, warn};
use serde::Deserialize;
use serde_json::json;
use std::ops::RangeInclusive;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, RwLock, RwLockReadGuard};
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
        }
    }
}

type State = web::Data<Arc<RwLock<SharedState>>>;
type Queue = web::Data<WriteQueue>;
type Counter = web::Data<Arc<AtomicU32>>;

fn read_state(state: &State) -> RwLockReadGuard<'_, SharedState> {
    state
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

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

/// Converts °C to the tenths sent to the stove
fn to_tenths(value: f32) -> Result<i32, ApiError> {
    if !value.is_finite() {
        return Err(ApiError::InvalidParameter(
            "temperature must be a finite number".into(),
        ));
    }
    Ok((value * 10.0).round() as i32)
}

fn tenths(value: f32) -> i32 {
    (value * 10.0).round() as i32
}

/// Queues a write and returns immediately (the outcome is available on `/api/request/{id}`)
fn enqueue_write(
    state: &State,
    queue: &Queue,
    counter: &Counter,
    command: StoveCommands,
    value: i32,
) -> HttpResponse {
    let request_id = counter.fetch_add(1, Ordering::SeqCst) % 100_000;
    let request = Request::new(
        request_id,
        Command::Dat,
        CommandType::Write,
        vec![(command as u32).to_string(), value.to_string()],
    );
    state_write(state).track_request(request_id, command.name(), &value.to_string());
    queue
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .push_back(QueuedWrite::new(request));

    let message = format!(
        "Request added for command: {}, value: {}, id: {}",
        command.name(),
        value,
        request_id
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
async fn get_inf(state: State) -> HttpResponse {
    HttpResponse::Ok().json(read_state(&state).get_inf())
}

/// DAT page 0: main stove data
#[utoipa::path(get, path = "/api/dat/0", responses((status = 200)), tag = "hottoh")]
async fn get_dat0(state: State) -> HttpResponse {
    HttpResponse::Ok().json(read_state(&state).get_dat0())
}

/// DAT page 1: chrono programs
#[utoipa::path(get, path = "/api/dat/1", responses((status = 200)), tag = "hottoh")]
async fn get_dat1(state: State) -> HttpResponse {
    HttpResponse::Ok().json(read_state(&state).get_dat1())
}

/// DAT page 2: hydraulic data and actual fan speeds
#[utoipa::path(get, path = "/api/dat/2", responses((status = 200)), tag = "hottoh")]
async fn get_dat2(state: State) -> HttpResponse {
    HttpResponse::Ok().json(read_state(&state).get_dat2())
}

/// Connection with the stove (connected, time of the last answer, last error)
#[utoipa::path(get, path = "/api/status", responses((status = 200)), tag = "hottoh")]
async fn get_status(state: State) -> HttpResponse {
    HttpResponse::Ok().json(read_state(&state).connection())
}

/// Outcome of a write request: pending, sent, ok, error (with the stove error code) or timeout
#[utoipa::path(
    get,
    path = "/api/request/{id}",
    params(("id" = u32, Path, description = "request_id returned by a POST")),
    responses((status = 200), (status = 404, description = "Unknown or expired request")),
    tag = "hottoh"
)]
async fn get_request_status(state: State, id: web::Path<u32>) -> Result<HttpResponse, ApiError> {
    let id = id.into_inner();
    read_state(&state)
        .get_request(id)
        .map(|status| HttpResponse::Ok().json(status))
        .ok_or_else(|| ApiError::NotFound(format!("request {}", id)))
}

/// Turns the stove on (`true`) or off (`false`)
#[utoipa::path(post, path = "/api/dat/set_on_off", request_body = DatPostBool,
    responses((status = 200), (status = 400)), tag = "hottoh")]
async fn post_on_off(
    request: web::Json<DatPostBool>,
    state: State,
    queue: Queue,
    counter: Counter,
) -> HttpResponse {
    let value = i32::from(request.value);
    enqueue_write(&state, &queue, &counter, StoveCommands::OnOff, value)
}

/// Activates or deactivates eco mode
#[utoipa::path(post, path = "/api/dat/set_eco_mode", request_body = DatPostBool,
    responses((status = 200), (status = 400)), tag = "hottoh")]
async fn post_eco_mode(
    request: web::Json<DatPostBool>,
    state: State,
    queue: Queue,
    counter: Counter,
) -> HttpResponse {
    let value = i32::from(request.value);
    enqueue_write(&state, &queue, &counter, StoveCommands::EcoMode, value)
}

/// Sets the ambiance temperature set point (°C), within the range reported by the stove
#[utoipa::path(post, path = "/api/dat/set_ambiance_temp", request_body = DatPostAmbianceTemp,
    responses((status = 200), (status = 400)), tag = "hottoh")]
async fn post_ambiance_temp(
    request: web::Json<DatPostAmbianceTemp>,
    state: State,
    queue: Queue,
    counter: Counter,
) -> Result<HttpResponse, ApiError> {
    let value = to_tenths(request.value)?;
    let (command, stove_range) = {
        let s = read_state(&state);
        let dat0 = s.dat0_if_received();
        match request.ambiance {
            1 => (
                StoveCommands::AmbianceTemperature1,
                dat0.map(|d| {
                    (
                        tenths(d.index_ambient_t1_set_min),
                        tenths(d.index_ambient_t1_set_max),
                    )
                }),
            ),
            2 => (
                StoveCommands::AmbianceTemperature2,
                dat0.map(|d| {
                    (
                        tenths(d.index_ambient_t2_set_min),
                        tenths(d.index_ambient_t2_set_max),
                    )
                }),
            ),
            _ => {
                return Err(ApiError::InvalidParameter("ambiance must be 1 or 2".into()));
            }
        }
    };
    let range = effective_range(stove_range, FIRMWARE_TEMP_RANGE);
    check_range("temperature (tenths of °C)", value, &range)?;
    Ok(enqueue_write(&state, &queue, &counter, command, value))
}

/// Activates (`true`) or deactivates (`false`) chrono mode
#[utoipa::path(post, path = "/api/dat/set_chrono_mode", request_body = DatPostBool,
    responses((status = 200), (status = 400)), tag = "hottoh")]
async fn post_chrono_mode(
    request: web::Json<DatPostBool>,
    state: State,
    queue: Queue,
    counter: Counter,
) -> HttpResponse {
    // The firmware expects 0 (manual) or 2 (chrono); 1 is refused with ERR;17.
    let value = ChronoMode::from_enabled(request.value) as i32;
    enqueue_write(&state, &queue, &counter, StoveCommands::ChronoOnOff, value)
}

/// Sets the temperature of a chrono program (°C), within the range reported by the stove
#[utoipa::path(post, path = "/api/dat/set_chrono_temp", request_body = DatPostChronoTemp,
    responses((status = 200), (status = 400)), tag = "hottoh")]
async fn post_chrono_temp(
    request: web::Json<DatPostChronoTemp>,
    state: State,
    queue: Queue,
    counter: Counter,
) -> Result<HttpResponse, ApiError> {
    let value = to_tenths(request.value)?;
    let (command, stove_range) = {
        let s = read_state(&state);
        let dat1 = s.dat1_if_received();
        match request.chrono {
            1 => (
                StoveCommands::ChronoTemperature1,
                dat1.map(|d| {
                    (
                        tenths(d.index_program_1_temp_min),
                        tenths(d.index_program_1_temp_max),
                    )
                }),
            ),
            2 => (
                StoveCommands::ChronoTemperature2,
                dat1.map(|d| {
                    (
                        tenths(d.index_program_2_temp_min),
                        tenths(d.index_program_2_temp_max),
                    )
                }),
            ),
            3 => (
                StoveCommands::ChronoTemperature3,
                dat1.map(|d| {
                    (
                        tenths(d.index_program_3_temp_min),
                        tenths(d.index_program_3_temp_max),
                    )
                }),
            ),
            _ => {
                return Err(ApiError::InvalidParameter(
                    "chrono must be between 1 and 3".into(),
                ));
            }
        }
    };
    let range = effective_range(stove_range, FIRMWARE_TEMP_RANGE);
    check_range("temperature (tenths of °C)", value, &range)?;
    Ok(enqueue_write(&state, &queue, &counter, command, value))
}

/// Sets a fan speed, from 0 to the maximum reported by the stove
#[utoipa::path(post, path = "/api/dat/set_fan_speed", request_body = DatPostFanSpeed,
    responses((status = 200), (status = 400)), tag = "hottoh")]
async fn post_fan_speed(
    request: web::Json<DatPostFanSpeed>,
    state: State,
    queue: Queue,
    counter: Counter,
) -> Result<HttpResponse, ApiError> {
    let (command, stove_max) = {
        let s = read_state(&state);
        let dat0 = s.dat0_if_received();
        match request.fan {
            1 => (
                StoveCommands::FanSpeed1,
                dat0.map(|d| d.index_fan_1_set_max),
            ),
            2 => (
                StoveCommands::FanSpeed2,
                dat0.map(|d| d.index_fan_2_set_max),
            ),
            3 => (
                StoveCommands::FanSpeed3,
                dat0.map(|d| d.index_fan_3_set_max),
            ),
            _ => {
                return Err(ApiError::InvalidParameter(
                    "fan must be between 1 and 3".into(),
                ));
            }
        }
    };
    let value = i32::try_from(request.value).unwrap_or(i32::MAX);
    let range = effective_range(stove_max.map(|max| (0, i32::from(max))), FIRMWARE_FAN_RANGE);
    check_range("fan speed", value, &range)?;
    Ok(enqueue_write(&state, &queue, &counter, command, value))
}

/// Sets the power level, within the range reported by the stove
#[utoipa::path(post, path = "/api/dat/set_power_level", request_body = DatPostU32,
    responses((status = 200), (status = 400)), tag = "hottoh")]
async fn post_power_level(
    request: web::Json<DatPostU32>,
    state: State,
    queue: Queue,
    counter: Counter,
) -> Result<HttpResponse, ApiError> {
    let stove_range = read_state(&state)
        .dat0_if_received()
        .map(|d| (i32::from(d.index_power_min), i32::from(d.index_power_max)));
    let value = i32::try_from(request.value).unwrap_or(i32::MAX);
    let range = effective_range(stove_range, FIRMWARE_POWER_RANGE);
    check_range("power level", value, &range)?;
    Ok(enqueue_write(
        &state,
        &queue,
        &counter,
        StoveCommands::PowerLevel,
        value,
    ))
}

/// Starts the HTTP server
pub async fn start_http_server(
    config: &AppConfig,
    shared_state: Arc<RwLock<SharedState>>,
    writes: WriteQueue,
    request_id: Arc<AtomicU32>,
) -> std::io::Result<()> {
    let http_address = format!("{}:{}", config.http_api.ip, config.http_api.port);
    info!("Starting HTTP server on {}", http_address);

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .app_data(web::Data::new(shared_state.clone()))
            .app_data(web::Data::new(writes.clone()))
            .app_data(web::Data::new(request_id.clone()))
            .app_data(web::JsonConfig::default().error_handler(|err, _req| {
                let message = err.to_string();
                error!("Invalid JSON body: {}", message);
                actix_web::error::InternalError::from_response(
                    err,
                    HttpResponse::BadRequest().json(json!({ "success": false, "error": message })),
                )
                .into()
            }))
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", ApiDoc::openapi()),
            )
            .route("/api/inf", web::get().to(get_inf))
            .route("/api/dat/0", web::get().to(get_dat0))
            .route("/api/dat/1", web::get().to(get_dat1))
            .route("/api/dat/2", web::get().to(get_dat2))
            .route("/api/status", web::get().to(get_status))
            .route("/api/request/{id}", web::get().to(get_request_status))
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
    })
    .bind(&http_address)?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(to_tenths(21.3).unwrap(), 213);
        assert_eq!(to_tenths(21.25).unwrap(), 213);
        assert_eq!(to_tenths(19.94).unwrap(), 199);
        assert!(to_tenths(f32::NAN).is_err());
    }
}
