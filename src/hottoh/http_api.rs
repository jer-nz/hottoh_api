use crate::hottoh::config::AppConfig;
use crate::hottoh::hottoh_const::{Command, CommandType, StoveCommands};
use crate::hottoh::shared_struct::SharedState;
use crate::hottoh::tcp_client_structs::Request;
use actix_web::{middleware, web, App, HttpResponse, HttpServer, ResponseError};
use log::{debug, error, info, warn};
use serde::Deserialize;
use serde_json::json;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex, RwLock};
use thiserror::Error;
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

/// API Error
#[derive(Error, Debug)]
pub enum ApiError {
    /// Invalid parameter
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// Internal error
    #[error("Internal error: {0}")]
    InternalError(String),

    /// Lock error
    #[error("Lock error: {0}")]
    LockError(String),
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        let error_json = json!({
            "error": self.to_string()
        });

        match self {
            ApiError::InvalidParameter(_) => {
                warn!("{}", self);
                HttpResponse::BadRequest().json(error_json)
            }
            ApiError::InternalError(_) => {
                error!("{}", self);
                HttpResponse::InternalServerError().json(error_json)
            }
            ApiError::LockError(_) => {
                error!("{}", self);
                HttpResponse::InternalServerError().json(error_json)
            }
        }
    }
}

/// API Documentation
#[derive(OpenApi)]
#[openapi(
    paths(
        get_inf,
        get_dat0,
        get_dat1,
        get_dat2,
        post_on_off,
        post_eco_mode,
        post_ambiance_temp,
        post_chrono_mode,
        post_chrono_temp,
        post_fan_speed,
        post_power_level
    ),
    components(
        schemas(DatPostBool, DatPostU32, DatPostAmbianceTemp, DatPostFanSpeed, DatPostChronoTemp)
    ),
    tags(
        (name = "hottoh", description = "Stove control API")
    )
)]
struct ApiDoc;

/// Boolean parameters for commands
#[derive(Deserialize, ToSchema)]
struct DatPostBool {
    /// Boolean value (true/false)
    ///
    /// Example: `true` to activate, `false` to deactivate
    #[schema(example = "true")]
    value: bool,
}

/// Integer parameters for commands
#[derive(Deserialize, ToSchema)]
struct DatPostU32 {
    /// Integer value
    ///
    /// Example: `5` for the power level
    #[schema(example = "5")]
    value: u32,
}

/// Parameters for ambiance temperature
#[derive(Deserialize, ToSchema)]
struct DatPostAmbianceTemp {
    /// Ambiance number (1 or 2)
    ///
    /// Example: `1` for the first ambiance, `2` for the second
    #[schema(example = "1")]
    ambiance: u32,
    /// Temperature in degrees Celsius
    ///
    /// Example: `21.5` for 21.5°C
    #[schema(example = "21.5")]
    value: f32,
}

/// Parameters for fan speed
#[derive(Deserialize, ToSchema)]
struct DatPostFanSpeed {
    /// Fan number (1-3)
    ///
    /// Example: `1` for the first fan
    #[schema(example = "1")]
    fan: u32,
    /// Speed (0-5)
    ///
    /// Example: `3` for a medium speed
    #[schema(example = "3")]
    value: u32,
}

/// Parameters for chrono temperature
#[derive(Deserialize, ToSchema)]
struct DatPostChronoTemp {
    /// Chrono number (1-3)
    ///
    /// Example: `1` for the first chrono
    #[schema(example = "1")]
    chrono: u32,
    /// Temperature in degrees Celsius
    ///
    /// Example: `20.0` for 20°C
    #[schema(example = "20.0")]
    value: f32,
}

/// Retrieves general information
#[utoipa::path(
    get,
    path = "/api/inf",
    responses(
        (status = 200, description = "Information retrieved successfully"),
        (status = 500, description = "Internal server error")
    ),
    tag = "hottoh"
)]
async fn get_inf(
    data: web::Data<Arc<RwLock<SharedState>>>,
) -> Result<web::Json<serde_json::Value>, ApiError> {
    match data.read() {
        Ok(state) => {
            let inf_clone = state.get_inf().clone();
            Ok(web::Json(json!(inf_clone)))
        }
        Err(e) => Err(ApiError::LockError(format!(
            "Failed to read shared state: {}",
            e
        ))),
    }
}

/// Retrieves DAT0 data
#[utoipa::path(
    get,
    path = "/api/dat/0",
    responses(
        (status = 200, description = "DAT0 data retrieved successfully"),
        (status = 500, description = "Internal server error")
    ),
    tag = "hottoh"
)]
async fn get_dat0(
    data: web::Data<Arc<RwLock<SharedState>>>,
) -> Result<web::Json<serde_json::Value>, ApiError> {
    match data.read() {
        Ok(state) => {
            let dat0_clone = state.get_dat0().clone();
            Ok(web::Json(json!(dat0_clone)))
        }
        Err(e) => Err(ApiError::LockError(format!(
            "Failed to read shared state: {}",
            e
        ))),
    }
}

/// Retrieves DAT1 data
#[utoipa::path(
    get,
    path = "/api/dat/1",
    responses(
        (status = 200, description = "DAT1 data retrieved successfully"),
        (status = 500, description = "Internal server error")
    ),
    tag = "hottoh"
)]
async fn get_dat1(
    data: web::Data<Arc<RwLock<SharedState>>>,
) -> Result<web::Json<serde_json::Value>, ApiError> {
    match data.read() {
        Ok(state) => {
            let dat1_clone = state.get_dat1().clone();
            Ok(web::Json(json!(dat1_clone)))
        }
        Err(e) => Err(ApiError::LockError(format!(
            "Failed to read shared state: {}",
            e
        ))),
    }
}

/// Retrieves DAT2 data
#[utoipa::path(
    get,
    path = "/api/dat/2",
    responses(
        (status = 200, description = "DAT2 data retrieved successfully"),
        (status = 500, description = "Internal server error")
    ),
    tag = "hottoh"
)]
async fn get_dat2(
    data: web::Data<Arc<RwLock<SharedState>>>,
) -> Result<web::Json<serde_json::Value>, ApiError> {
    match data.read() {
        Ok(state) => {
            let dat2_clone = state.get_dat2().clone();
            Ok(web::Json(json!(dat2_clone)))
        }
        Err(e) => Err(ApiError::LockError(format!(
            "Failed to read shared state: {}",
            e
        ))),
    }
}

/// Turns the stove on or off
///
/// Request example:
/// ```json
/// {
///   "value": true
/// }
/// ```
/// - `true`: Turns the stove on
/// - `false`: Turns the stove off
#[utoipa::path(
    post,
    path = "/api/dat/set_on_off",
    request_body = DatPostBool,
    responses(
        (status = 200, description = "Stove turned on or off successfully"),
        (status = 500, description = "Internal server error")
    ),
    tag = "hottoh"
)]
async fn post_on_off(
    request: web::Json<DatPostBool>,
    request_queue: web::Data<Arc<RwLock<VecDeque<Request>>>>,
    request_id_counter: web::Data<Arc<Mutex<u32>>>,
) -> Result<HttpResponse, ApiError> {
    let value = if request.value { 1 } else { 0 };
    handle_request(
        request_queue,
        request_id_counter,
        StoveCommands::OnOff as u32,
        value,
    )
    .await
}

/// Activates or deactivates eco mode
///
/// Request example:
/// ```json
/// {
///   "value": true
/// }
/// ```
/// - `true`: Activates eco mode (energy saving)
/// - `false`: Deactivates eco mode
#[utoipa::path(
    post,
    path = "/api/dat/set_eco_mode",
    request_body = DatPostBool,
    responses(
        (status = 200, description = "Eco mode set successfully"),
        (status = 500, description = "Internal server error")
    ),
    tag = "hottoh"
)]
async fn post_eco_mode(
    request: web::Json<DatPostBool>,
    request_queue: web::Data<Arc<RwLock<VecDeque<Request>>>>,
    request_id_counter: web::Data<Arc<Mutex<u32>>>,
) -> Result<HttpResponse, ApiError> {
    let value = if request.value { 1 } else { 0 };
    handle_request(
        request_queue,
        request_id_counter,
        StoveCommands::EcoMode as u32,
        value,
    )
    .await
}

/// Sets the ambiance temperature
///
/// Request example:
/// ```json
/// {
///   "ambiance": 1,
///   "value": 21.5
/// }
/// ```
/// - `ambiance`: Ambiance number (1 or 2)
/// - `value`: Temperature in degrees Celsius (ex: 21.5 for 21.5°C)
#[utoipa::path(
    post,
    path = "/api/dat/set_ambiance_temp",
    request_body = DatPostAmbianceTemp,
    responses(
        (status = 200, description = "Ambiance temperature set successfully"),
        (status = 400, description = "Invalid parameters"),
        (status = 500, description = "Internal server error")
    ),
    tag = "hottoh"
)]
async fn post_ambiance_temp(
    request: web::Json<DatPostAmbianceTemp>,
    request_queue: web::Data<Arc<RwLock<VecDeque<Request>>>>,
    request_id_counter: web::Data<Arc<Mutex<u32>>>,
) -> Result<HttpResponse, ApiError> {
    // Validation
    if request.value.is_nan() || request.value.is_infinite() {
        return Err(ApiError::InvalidParameter(
            "Temperature cannot be NaN or infinite".into(),
        ));
    }

    let command = match request.ambiance {
        1 => StoveCommands::AmbianceTemperature1,
        2 => StoveCommands::AmbianceTemperature2,
        _ => {
            return Err(ApiError::InvalidParameter(
                "Ambiance number must be 1 or 2".into(),
            ))
        }
    };

    handle_request(
        request_queue,
        request_id_counter,
        command as u32,
        (request.value * 10.0) as i32,
    )
    .await
}

/// Activates or deactivates chrono mode
///
/// Request example:
/// ```json
/// {
///   "value": true
/// }
/// ```
/// - `true`: Activates chrono mode (schedule programming)
/// - `false`: Deactivates chrono mode
#[utoipa::path(
    post,
    path = "/api/dat/set_chrono_mode",
    request_body = DatPostBool,
    responses(
        (status = 200, description = "Chrono mode set successfully"),
        (status = 500, description = "Internal server error")
    ),
    tag = "hottoh"
)]
async fn post_chrono_mode(
    request: web::Json<DatPostBool>,
    request_queue: web::Data<Arc<RwLock<VecDeque<Request>>>>,
    request_id_counter: web::Data<Arc<Mutex<u32>>>,
) -> Result<HttpResponse, ApiError> {
    handle_request(
        request_queue,
        request_id_counter,
        StoveCommands::ChronoOnOff as u32,
        request.value,
    )
    .await
}

/// Sets the chrono temperature
///
/// Request example:
/// ```json
/// {
///   "chrono": 1,
///   "value": 20.0
/// }
/// ```
/// - `chrono`: Chrono number (1, 2 or 3)
/// - `value`: Temperature in degrees Celsius (ex: 20.0 for 20°C)
#[utoipa::path(
    post,
    path = "/api/dat/set_chrono_temp",
    request_body = DatPostChronoTemp,
    responses(
        (status = 200, description = "Chrono temperature set successfully"),
        (status = 400, description = "Invalid parameters"),
        (status = 500, description = "Internal server error")
    ),
    tag = "hottoh"
)]
async fn post_chrono_temp(
    request: web::Json<DatPostChronoTemp>,
    request_queue: web::Data<Arc<RwLock<VecDeque<Request>>>>,
    request_id_counter: web::Data<Arc<Mutex<u32>>>,
) -> Result<HttpResponse, ApiError> {
    // Validation
    if request.value.is_nan() || request.value.is_infinite() {
        return Err(ApiError::InvalidParameter(
            "Temperature cannot be NaN or infinite".into(),
        ));
    }

    let command = match request.chrono {
        1 => StoveCommands::ChronoTemperature1,
        2 => StoveCommands::ChronoTemperature2,
        3 => StoveCommands::ChronoTemperature3,
        _ => {
            return Err(ApiError::InvalidParameter(
                "Chrono number must be between 1 and 3".into(),
            ))
        }
    };

    handle_request(
        request_queue,
        request_id_counter,
        command as u32,
        (request.value * 10.0) as i32,
    )
    .await
}

/// Sets the fan speed
///
/// Request example:
/// ```json
/// {
///   "fan": 1,
///   "value": 3
/// }
/// ```
/// - `fan`: Fan number (1, 2 or 3)
/// - `value`: Speed (0 to 5, where 0 = off and 5 = maximum speed)
#[utoipa::path(
    post,
    path = "/api/dat/set_fan_speed",
    request_body = DatPostFanSpeed,
    responses(
        (status = 200, description = "Fan speed set successfully"),
        (status = 400, description = "Invalid parameters"),
        (status = 500, description = "Internal server error")
    ),
    tag = "hottoh"
)]
async fn post_fan_speed(
    request: web::Json<DatPostFanSpeed>,
    request_queue: web::Data<Arc<RwLock<VecDeque<Request>>>>,
    request_id_counter: web::Data<Arc<Mutex<u32>>>,
) -> Result<HttpResponse, ApiError> {
    // Validation
    if request.value > 5 {
        return Err(ApiError::InvalidParameter(
            "Fan speed must be between 0 and 5".into(),
        ));
    }

    let command = match request.fan {
        1 => StoveCommands::FanSpeed1,
        2 => StoveCommands::FanSpeed2,
        3 => StoveCommands::FanSpeed3,
        _ => {
            return Err(ApiError::InvalidParameter(
                "Fan number must be between 1 and 3".into(),
            ))
        }
    };

    handle_request(
        request_queue,
        request_id_counter,
        command as u32,
        request.value,
    )
    .await
}

/// Sets the power level
///
/// Request example:
/// ```json
/// {
///   "value": 5
/// }
/// ```
/// - `value`: Power level (0 to 10, where 0 = minimum and 10 = maximum)
#[utoipa::path(
    post,
    path = "/api/dat/set_power_level",
    request_body = DatPostU32,
    responses(
        (status = 200, description = "Power level set successfully"),
        (status = 400, description = "Invalid parameters"),
        (status = 500, description = "Internal server error")
    ),
    tag = "hottoh"
)]
async fn post_power_level(
    request: web::Json<DatPostU32>,
    request_queue: web::Data<Arc<RwLock<VecDeque<Request>>>>,
    request_id_counter: web::Data<Arc<Mutex<u32>>>,
) -> Result<HttpResponse, ApiError> {
    // Validation
    if request.value > 10 {
        return Err(ApiError::InvalidParameter(
            "Power level must be between 0 and 10".into(),
        ));
    }

    handle_request(
        request_queue,
        request_id_counter,
        StoveCommands::PowerLevel as u32,
        request.value,
    )
    .await
}

/// Starts the HTTP server
pub async fn start_http_server(
    request_queue: Arc<RwLock<VecDeque<Request>>>,
    shared_state: Arc<RwLock<SharedState>>,
    request_id_counter: Arc<Mutex<u32>>,
    config: Arc<RwLock<AppConfig>>,
) -> std::io::Result<()> {
    // Extract necessary information from the config and release the lock
    // before asynchronous operations
    let http_address = {
        let cfg = config.read().expect("Cannot read config in http thread.");
        format!("{}:{}", cfg.http_api.ip, cfg.http_api.port)
    };

    info!("Starting HTTP server on {}", http_address);

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .app_data(web::Data::new(request_queue.clone()))
            .app_data(web::Data::new(shared_state.clone()))
            .app_data(web::Data::new(request_id_counter.clone()))
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", ApiDoc::openapi()),
            )
            .route("/api/inf", web::get().to(get_inf))
            .route("/api/dat/0", web::get().to(get_dat0))
            .route("/api/dat/1", web::get().to(get_dat1))
            .route("/api/dat/2", web::get().to(get_dat2))
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

/// Handles a request and adds it to the queue
async fn handle_request(
    request_queue: web::Data<Arc<RwLock<VecDeque<Request>>>>,
    request_id_counter: web::Data<Arc<Mutex<u32>>>,
    action: u32,
    value: impl ToString,
) -> Result<HttpResponse, ApiError> {
    let mut id_lock = match request_id_counter.lock() {
        Ok(lock) => lock,
        Err(e) => {
            error!("Failed to lock request ID counter: {}", e);
            return Err(ApiError::InternalError(
                "Failed to lock request ID counter".into(),
            ));
        }
    };

    let request_id = *id_lock;
    let new_request = Request::new(
        request_id,
        Command::Dat,
        CommandType::Write,
        vec![action.to_string(), value.to_string()],
    );

    match request_queue.write() {
        Ok(mut queue) => {
            queue.push_back(new_request);
            *id_lock = (*id_lock + 1) % 100000;
            // Convert the action to StoveCommands to get the command name
            let command_name = match action {
                0 => "OnOff",
                1 => "EcoMode",
                2 => "PowerLevel",
                3 => "AmbianceTemperature1",
                4 => "AmbianceTemperature2",
                5 => "FanSpeed1",
                6 => "FanSpeed2",
                7 => "FanSpeed3",
                8 => "ChronoOnOff",
                9 => "ChronoTemperature1",
                10 => "ChronoTemperature2",
                11 => "ChronoTemperature3",
                12 => "SanTemperature",
                13 => "PufTemperature",
                14 => "BoilerTemperature",
                15 => "HottohSetRecipe",
                16 => "HottohSetPelSetpoint",
                _ => "Unknown",
            };

            debug!(
                "Request added for command: {}, value: {}, id: {}",
                command_name,
                value.to_string(),
                request_id
            );
            Ok(HttpResponse::Ok().json(json!({
                "success": true,
                "message": format!("Request added for command: {}, value: {}, id: {}", command_name, value.to_string(), request_id),
                "request_id": request_id
            })))
        }
        Err(e) => {
            error!("Failed to lock request queue: {}", e);
            Err(ApiError::LockError("Failed to lock request queue".into()))
        }
    }
}
