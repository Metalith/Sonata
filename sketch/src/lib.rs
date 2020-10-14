#[macro_use]
extern crate log;
extern crate ash;
extern crate ultraviolet as uv;
extern crate winapi;

mod buffers;
mod commands;
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
    fn vulkan_object(&self) -> &Self::Object;
    fn cleanup(&self, _context: &GraphicContext);
}
