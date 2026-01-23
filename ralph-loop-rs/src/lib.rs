//! Ralph Loop: A concurrent Rust application that runs Claude Code in a loop
//! with real-time context monitoring.
//!
//! This crate provides the core functionality for running Claude as a subprocess
//! and monitoring its output for completion promises and context limits.

pub mod agent;
pub mod config;
pub mod error;
pub mod loop_controller;
pub mod monitor;
pub mod process;
pub mod state;
pub mod token_counter;

pub use agent::{Agent, AgentResult, ClaudeAgent, ExitReason};
pub use config::Config;
pub use error::{RalphError, Result};
pub use loop_controller::{LoopController, LoopResult};
