//! Hottoh API - A Rust library for controlling stoves via TCP/IP
//!
//! This module provides functionality to connect to and control stoves
//! that implement the Hottoh protocol. It includes TCP client functionality,
//! data structures for representing stove state, and an HTTP API for remote control.

/// Configuration handling for the application
pub mod config;
/// Check of the HottoH update server for a newer module firmware
pub mod firmware_update;
/// Constants used throughout the application
pub mod hottoh_const;
/// Data structures for representing stove data
pub mod hottoh_structs;
/// HTTP API for remote control of the stove
pub mod http_api;
/// HTTP endpoints of the optional Wi-Fi module features
pub mod http_module;
/// Logging functionality
pub mod logger;
/// Answers of the module commands (schedule, clock, time zone, data logger, PIN)
pub mod module_data;
/// Shared state between components
pub mod shared_struct;
/// Periodic statistics and process metrics
pub mod stats;
/// TCP client for communicating with the stove
pub mod tcp_client;
/// Data structures for TCP client requests and responses
pub mod tcp_client_structs;
