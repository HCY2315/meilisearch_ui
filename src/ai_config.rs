// AI Config module - re-exports from storage.rs where the actual implementation lives
// AiConfig type is defined in types.rs

pub use crate::storage::{load_ai_config, save_ai_config};

pub fn init() {
    // no-op for skeleton
}
