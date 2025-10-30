pub mod api;
pub mod config;
pub mod db;
pub mod engine;
pub mod errors;
pub mod models;
pub mod utils;

use tokio::sync::{broadcast, mpsc};

use crate::models::domain::{EngineEvent, EngineRequest};

/// Application state shared across all handlers
#[derive(Clone)]
pub struct AppState {
    pub db: db::Db,
    pub engine_tx: mpsc::Sender<EngineRequest>,
    pub event_tx: broadcast::Sender<EngineEvent>,
}
