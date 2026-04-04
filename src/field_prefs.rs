// Field preferences module - re-exports from storage.rs where the actual implementation lives

pub use crate::storage::{load_field_labels, save_field_labels};

pub fn init() {
    // no-op for skeleton
}
