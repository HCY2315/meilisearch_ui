// History module - re-exports from storage.rs where the actual implementation lives
// HistoryItem type is defined in types.rs

pub use crate::storage::{load_search_history, save_search_history};

pub fn init() {
    // no-op for skeleton
}
