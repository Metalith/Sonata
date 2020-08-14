#[macro_use]
extern crate log;
extern crate ash;
extern crate cgmath;
extern crate winapi;

mod buffers;
mod commands;
pub mod device;
mod graphic_context;
pub mod model;
mod pipeline;
mod renderer;
mod renderpass;
mod sync;
mod utility;

pub use graphic_context::GraphicContext;
pub use renderer::Renderer;

pub trait VulkanObject {
    type Object;
    fn vulkan_object(&self) -> &Self::Object;
    fn cleanup(&self, _context: &GraphicContext);
}
