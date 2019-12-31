use crate::PhysicalDevice;
use crate::Renderer;
use crate::Surface;
use crate::VulkanObject;

use ash::{extensions::khr, version::DeviceV1_0, vk, Device, Instance};

pub struct SwapChain {
    swapchain_loader: khr::Swapchain,
    swapchain: vk::SwapchainKHR,
    surface_format: vk::SurfaceFormatKHR,
    extent: vk::Extent2D,
    present_mode: vk::PresentModeKHR,
    images: Vec<vk::Image>,
    image_views: Vec<vk::ImageView>,
}

impl SwapChain {
    pub fn new(instance: &Instance, logical_device: &Device, physical_device: &PhysicalDevice, surface: &Surface, preferred_dimensions: [u32; 2]) -> Self {
        let swapchain_support = Self::query_support(*physical_device.vulkan_object(), surface);

        let surface_format = Self::choose_surface_format(swapchain_support.formats);
        let present_mode = Self::choose_present_mode(swapchain_support.present_modes);
        let extent = Self::choose_extent(swapchain_support.capabilities, preferred_dimensions);

        let mut image_count = swapchain_support.capabilities.min_image_count + 1;
        if swapchain_support.capabilities.min_image_count > 0 && image_count > swapchain_support.capabilities.max_image_count {
            image_count = swapchain_support.capabilities.max_image_count;
        }

        let mut create_info_builder = vk::SwapchainCreateInfoKHR::builder()
            .surface(*surface.vulkan_object())
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT);

        let queue_indices = [*physical_device.graphics_index(), *physical_device.present_index()];
        create_info_builder = if physical_device.graphics_index() != physical_device.present_index() {
            create_info_builder.image_sharing_mode(vk::SharingMode::CONCURRENT).queue_family_indices(&queue_indices)
        } else {
            create_info_builder.image_sharing_mode(vk::SharingMode::EXCLUSIVE)
        };

        let create_info = create_info_builder
            .pre_transform(swapchain_support.capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .build();

        let swapchain_loader = khr::Swapchain::new(instance, logical_device);
        let swapchain = unsafe { swapchain_loader.create_swapchain(&create_info, None).unwrap() };
        let images = unsafe { swapchain_loader.get_swapchain_images(swapchain).unwrap() };
        let mut image_views: Vec<vk::ImageView> = Vec::new();

        for &image in images.iter() {
            let image_create_info = vk::ImageViewCreateInfo::builder()
                .image(image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(surface_format.format)
                .components(
                    vk::ComponentMapping::builder()
                        .r(vk::ComponentSwizzle::IDENTITY)
                        .g(vk::ComponentSwizzle::IDENTITY)
                        .b(vk::ComponentSwizzle::IDENTITY)
                        .a(vk::ComponentSwizzle::IDENTITY)
                        .build(),
                )
                .subresource_range(
                    vk::ImageSubresourceRange::builder()
                        .base_mip_level(0)
                        .level_count(1)
                        .base_array_layer(0)
                        .layer_count(1)
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .build(),
                )
                .build();

            unsafe { image_views.push(logical_device.create_image_view(&image_create_info, None).unwrap()) };
        }

        SwapChain {
            swapchain_loader: swapchain_loader,
            swapchain: swapchain,
            surface_format: surface_format,
            extent: extent,
            present_mode: present_mode,
            images: images,
            image_views: image_views,
        }
    }

    pub fn query_support(device: vk::PhysicalDevice, surface: &Surface) -> SwapChainSupportDetails {
        SwapChainSupportDetails {
            capabilities: unsafe { surface.get_loader().get_physical_device_surface_capabilities(device, *surface.vulkan_object()).unwrap() },
            formats: unsafe { surface.get_loader().get_physical_device_surface_formats(device, *surface.vulkan_object()).unwrap() },
            present_modes: unsafe { surface.get_loader().get_physical_device_surface_present_modes(device, *surface.vulkan_object()).unwrap() },
        }
    }

    pub fn choose_surface_format(available_formats: Vec<vk::SurfaceFormatKHR>) -> vk::SurfaceFormatKHR {
        for &available_format in available_formats.iter() {
            if available_format.format == vk::Format::R8G8B8A8_UNORM && available_format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR {
                return available_format;
            }
        }

        available_formats[0]
    }

    pub fn choose_present_mode(availabe_present_modes: Vec<vk::PresentModeKHR>) -> vk::PresentModeKHR {
        for &available_present_mode in availabe_present_modes.iter() {
            if available_present_mode == vk::PresentModeKHR::MAILBOX {
                return available_present_mode;
            }
        }

        availabe_present_modes[0]
    }

    pub fn choose_extent(capabilities: vk::SurfaceCapabilitiesKHR, preferred_dimensions: [u32; 2]) -> vk::Extent2D {
        if capabilities.current_extent.width != std::u32::MAX {
            return capabilities.current_extent;
        } else {
            vk::Extent2D {
                width: preferred_dimensions[0].max(capabilities.min_image_extent.width).min(capabilities.max_image_extent.width),
                height: preferred_dimensions[1].max(capabilities.min_image_extent.height).min(capabilities.max_image_extent.height),
            }
        }
    }

    pub fn extent(&self) -> &vk::Extent2D {
        &self.extent
    }
    
    pub fn surface_format(&self) -> &vk::SurfaceFormatKHR {
        &self.surface_format
    }

    pub fn present_mode(&self) -> &vk::PresentModeKHR {
        &self.present_mode
    }

    pub fn image_views(&self) -> &Vec<vk::ImageView> {
        &self.image_views
    }   

    pub fn get_loader(&self) -> &khr::Swapchain {
        &self.swapchain_loader
    }
}

impl VulkanObject for SwapChain {
    type Object = vk::SwapchainKHR;

    fn vulkan_object(&self) -> &Self::Object {
        &self.swapchain
    }

    fn cleanup(&self, _renderer: &Renderer) {
        unsafe {
            for &image_view in self.image_views.iter() {
                _renderer.logical_device.vulkan_object().destroy_image_view(image_view, None);
            }
            self.swapchain_loader.destroy_swapchain(self.swapchain, None);
        }
    }
}

pub struct SwapChainSupportDetails {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}
