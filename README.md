# Hottoh API

[![Rust CI](https://github.com/yourusername/hottoh_api/actions/workflows/ci.yml/badge.svg)](https://github.com/yourusername/hottoh_api/actions/workflows/ci.yml)
[![Release](https://github.com/yourusername/hottoh_api/actions/workflows/release.yml/badge.svg)](https://github.com/yourusername/hottoh_api/actions/workflows/release.yml)

A Rust application for controlling stoves via TCP communication with an HTTP API interface.

## Project Overview

Hottoh API is a bridge application that connects to stoves (heating devices) via TCP protocol and exposes their functionality through a RESTful HTTP API. This allows for remote control and monitoring of compatible stoves from any device that can make HTTP requests.

### Key Features

- TCP communication with stoves
- RESTful HTTP API with Swagger documentation
- Real-time monitoring of stove status
- Control of stove functions (power, temperature, fan speed, etc.)
- Configurable logging system
- Robust error handling and recovery

## Getting Started

### Prerequisites

- Rust (edition 2021)
- Compatible stove with TCP connectivity

### Installation

1. Clone the repository:
   ```
   git clone https://github.com/yourusername/hottoh_api.git
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
   port = 8080         # Replace with your stove's port

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

## API Documentation

Once the application is running, you can access the Swagger UI documentation at:
```
http://localhost:3000/swagger-ui/
```

This provides interactive documentation for all available API endpoints.

### Example API Endpoints

- `GET /api/inf` - Get general information about the stove
- `GET /api/dat/0` - Get detailed stove data (page 0)
- `POST /api/dat/set_on_off` - Turn the stove on or off
- `POST /api/dat/set_power_level` - Set the power level
- `POST /api/dat/set_ambiance_temp` - Set the ambient temperature

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

## CI/CD

Ce projet utilise GitHub Actions pour l'intégration continue et le déploiement continu :

### Workflows CI

- **Rust CI** (`ci.yml`) : Exécuté à chaque push et pull request sur la branche main
  - Vérifie la compilation du code sur toutes les plateformes (Linux, Windows, macOS)
  - Exécute les tests unitaires
  - Vérifie le formatage du code avec rustfmt
  - Analyse le code avec clippy pour détecter les problèmes potentiels

### Workflows de Release

- **Release** (`release.yml`) : Exécuté lorsqu'un tag est poussé (format `v*`)
  - Compile des binaires pour plusieurs plateformes :
    - Linux (x86_64, aarch64)
    - Windows (x86_64)
    - macOS (x86_64, aarch64)
  - Crée une release GitHub avec les binaires compilés
  - Génère un changelog basé sur les commits

## Contributing

Contributions are welcome! Here are some guidelines:

1. **Code Style**: Follow Rust's official style guidelines. Le projet utilise rustfmt avec la configuration dans `.rustfmt.toml`.
2. **Documentation**: Add documentation comments (///) to all public items.
3. **Error Handling**: Use proper error handling instead of `unwrap()` or `expect()`.
4. **Testing**: Add tests for new functionality.
5. **CI Checks**: Ensure all CI checks pass before submitting your PR.
6. **Pull Requests**: Submit PRs with clear descriptions of changes.

Tous les pull requests sont automatiquement vérifiés par notre CI pour s'assurer que le code compile sur toutes les plateformes cibles et respecte nos standards de qualité.

## License

[MIT License](LICENSE)

## Acknowledgments

- Thanks to all contributors who have helped with this project.
- Special thanks to the Rust community for their excellent documentation and tools.