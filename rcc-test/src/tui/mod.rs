pub mod app;
pub mod event;
pub mod runner;

// UI modules
pub mod ui;
pub mod modals;
pub mod handlers;
pub mod executor;

pub use app::{TuiApp, AppMode, FocusedPane};
pub use runner::TuiRunner;