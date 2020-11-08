#[macro_use]
extern crate log;
extern crate ash;
extern crate ultraviolet as uv;
extern crate winapi;

mod buffers;
mod commands;
mod constants;
pub mod device;
mod graphic_context;
pub mod models;
mod pipelines;
mod renderer;
mod renderpasses;
mod sync;
mod utilities;

pub use graphic_context::GraphicContext;
pub use renderer::Renderer;

pub trait VulkanObject {
    type Object;
    fn vk(&self) -> &Self::Object;
}
