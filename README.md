# Hottoh API

[![Rust CI](https://github.com/jer-nz/hottoh_api/actions/workflows/ci.yml/badge.svg)](https://github.com/jer-nz/hottoh_api/actions/workflows/ci.yml)
[![Release](https://github.com/jer-nz/hottoh_api/actions/workflows/release.yml/badge.svg)](https://github.com/jer-nz/hottoh_api/actions/workflows/release.yml)

A Rust application for controlling Hottoh stoves via an HTTP API interface.

## Project Overview

Hottoh API is a bridge application that connects to pellet stoves via Hottoh proprietary TCP protocol and exposes their functionality through a RESTful HTTP API. This allows for remote control and monitoring of compatible stoves from any device that can make HTTP requests.

### Key Features

- TCP communication with stoves
- RESTful HTTP API with Swagger documentation
- Real-time monitoring of stove status
- Control of stove functions (power, temperature, fan speed, etc.)
- Configurable logging system
- Robust error handling and recovery
- Very low CPU and memory usage

## Getting Started

### Prerequisites

- Rust (edition 2021)
- Compatible Hottoh stove (default TCP port is 5001)

### Installation

1. Clone the repository:
   ```
   git clone https://github.com/jer-nz/hottoh_api.git
   cd hottoh_api
   ```

2. Build the project:
   ```
   cargo build --release
   ```

3. Configure the application by editing `config.ini`:
   ```ini
   [stove]
   ip = 192.168.1.100  # Replace with your stove's IP address
   port = 5001         # Replace with your stove's port

   [http_api]
   ip = 0.0.0.0        # Listen on all interfaces
   port = 3000         # Port for the HTTP API

   [log]
   level = info        # Log level (trace, debug, info, warn, error)
   directory = logs    # Directory for log files
   max_log_files = 10  # Maximum number of log files to keep
   ```

4. Run the application:
   ```
   ./target/release/hottoh_api config.ini
   ```
   or
   ```
   ./target/release/hottoh_api 
   ``` 
   If the config.ini is in the same folder.

## API Documentation

Once the application is running, you can access the Swagger UI documentation at:
```
http://localhost:3000/swagger-ui/
```

This provides interactive documentation for all available API endpoints.

### API Endpoints

#### GET Endpoints
- `GET /api/inf` - Get general information about the stove
- `GET /api/dat/0` - Get detailed stove data (page 0)
- `GET /api/dat/1` - Get detailed stove data (page 1)
- `GET /api/dat/2` - Get detailed stove data (page 2)

#### POST Endpoints
- `POST /api/dat/set_on_off` - Turn the stove on or off
- `POST /api/dat/set_eco_mode` - Activate or deactivate eco mode
- `POST /api/dat/set_power_level` - Set the power level (0-10)
- `POST /api/dat/set_ambiance_temp` - Set the ambient temperature
- `POST /api/dat/set_chrono_mode` - Activate or deactivate chrono mode
- `POST /api/dat/set_chrono_temp` - Set the chrono temperature
- `POST /api/dat/set_fan_speed` - Set the fan speed (0-5)

## Project Structure

- `src/main.rs` - Application entry point
- `src/hottoh/` - Main module directory
  - `config.rs` - Configuration handling
  - `http_api.rs` - HTTP API implementation
  - `logger.rs` - Logging system
  - `tcp_client.rs` - TCP communication with the stove
  - `tcp_client_structs.rs` - Data structures for TCP communication
  - `hottoh_const.rs` - Constants and enumerations
  - `hottoh_structs.rs` - Data structures for stove data
  - `shared_struct.rs` - Shared state between components

## Contributing

Contributions are welcome!

## License

[MIT License](LICENSE)
