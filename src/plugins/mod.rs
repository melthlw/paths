pub mod external;
pub mod features;
pub mod manager;
pub mod manifest;
pub mod modules;
pub mod registry;
pub mod traits;
pub mod ui;

pub use manager::PluginManager;
pub use traits::{PluginContext, PluginRenderContext};
