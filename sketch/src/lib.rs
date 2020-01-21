#[macro_use]
extern crate log;

mod buffers;
mod commands;
mod device;
pub mod model;
mod pipeline;
mod renderer;
mod renderpass;
mod utility;

pub use renderer::Renderer;

trait VulkanObject {
    type Object;
    fn vulkan_object(&self) -> &Self::Object;
    fn cleanup(&self, _renderer: &Renderer);
}
