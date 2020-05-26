#[macro_use]
extern crate log;
extern crate cgmath;
extern crate ash;

mod buffers;
mod commands;
mod device;
mod graphic_context;
pub mod model;
mod pipeline;
mod renderer;
mod renderpass;
mod sync;
mod utility;

use graphic_context::GraphicContext;
pub use renderer::Renderer;

trait VulkanObject {
    type Object;
    fn vulkan_object(&self) -> &Self::Object;
    fn cleanup(&self, _context: &GraphicContext);
}
