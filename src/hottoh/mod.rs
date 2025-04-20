//! Hottoh API - A Rust library for controlling stoves via TCP/IP
//!
//! This module provides functionality to connect to and control stoves
//! that implement the Hottoh protocol. It includes TCP client functionality,
//! data structures for representing stove state, and an HTTP API for remote control.

/// Configuration handling for the application
pub mod config;
/// Constants used throughout the application
pub mod hottoh_const;
/// Data structures for representing stove data
pub mod hottoh_structs;
/// HTTP API for remote control of the stove
pub mod http_api;
/// Logging functionality
pub mod logger;
/// Shared state between components
pub mod shared_struct;
/// TCP client for communicating with the stove
pub mod tcp_client;
/// Data structures for TCP client requests and responses
pub mod tcp_client_structs;
