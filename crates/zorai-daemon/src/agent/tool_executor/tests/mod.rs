use super::*;

use crate::agent::{
    types::{AgentConfig, AgentEvent, ToolCall, ToolFunction},
    AgentEngine,
};
use crate::session_manager::SessionManager;
use base64::Engine;
use std::fs;
use std::sync::{Arc, Mutex};
use tempfile::tempdir;
use tokio::sync::broadcast;
use tokio::time::{timeout, Duration};

mod part1;
mod part10;
mod part2;
mod part3;
mod part4;
mod part5;
mod part6;
mod part7;
mod part8;
mod part9;

pub(super) fn current_dir_test_lock() -> &'static std::sync::Mutex<()> {
    crate::test_support::env_test_mutex()
}
