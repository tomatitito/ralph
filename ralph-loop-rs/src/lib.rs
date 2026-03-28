//! Ralph Loop: A concurrent Rust application that runs coding agents in a loop
//! with real-time context monitoring.
//!
//! This crate provides the core functionality for running a supported coding
//! agent as a subprocess and monitoring its output for completion promises and
//! context limits.

pub mod agent;
pub mod config;
pub mod error;
pub mod json_events;
pub mod loop_controller;
pub mod monitor;
pub mod process;
pub mod state;
pub mod token_counter;
pub mod transcript;

pub use agent::{Agent, AgentResult, CliAgent, ExitReason};
pub use config::{AgentProvider, Config};
pub use error::{RalphError, Result};
pub use loop_controller::{LoopController, LoopResult};
