mod buffers_readback;
mod builders;
mod creators;
mod errors;
mod plugin;
mod resources;

use buffers_readback::*;
use builders::*;
use errors::*;

pub mod prelude {
    pub use super::builders::{BindLayoutBuilder, PipelineBuilder};
    pub use super::creators::*;
    pub use super::errors::*;
    pub use super::plugin::*;
    pub use super::resources::*;
    pub use super::*;
}

pub type WgslBurritoPluginStr =
    plugin::WgslBurritoPlugin<&'static str, &'static str, &'static str, &'static str>;
pub type WgslMainBurritoStr = resources::WgslMainBurrito<&'static str>;
pub type WgslRenderBurritoStr =
    resources::WgslRenderBurrito<&'static str, &'static str, &'static str, &'static str>;
