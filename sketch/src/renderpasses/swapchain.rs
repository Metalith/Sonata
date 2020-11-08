use std::sync::Arc;

use crate::{
    device::{Device, Surface, Window},
    VulkanObject,
};

use ash::{extensions::khr, version::DeviceV1_0, vk};

pub struct SwapChain {
    device: Arc<Device>,
    _surface: Arc<Surface>,
    swapchain_loader: khr::Swapchain,
    swapchain: vk::SwapchainKHR,
    surface_format: vk::SurfaceFormatKHR,
    extent: vk::Extent2D,
    images: Vec<vk::Image>,
    image_views: Vec<vk::ImageView>,
}

impl SwapChain {
    pub fn new(device: Arc<Device>, surface: Arc<Surface>, window: &Window, old_swapchain: Option<&Arc<SwapChain>>) -> Arc<Self> {
        let swapchain_support = Self::query_support(*device.physical_device().vk(), &surface);

        let surface_format = Self::choose_surface_format(swapchain_support.formats);
        let present_mode = Self::choose_present_mode(swapchain_support.present_modes);
        let extent = Self::choose_extent(swapchain_support.capabilities, window);

        let mut image_count = swapchain_support.capabilities.min_image_count + 1;
        if swapchain_support.capabilities.min_image_count > 0 && image_count > swapchain_support.capabilities.max_image_count {
            image_count = swapchain_support.capabilities.max_image_count;
        }

        let mut create_info_builder = vk::SwapchainCreateInfoKHR::builder()
            .surface(*surface.vk())
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT);

        let queue_indices = [device.physical_device().graphics_index(), device.physical_device().present_index()];
        create_info_builder = if device.physical_device().graphics_index() != device.physical_device().present_index() {
            create_info_builder.image_sharing_mode(vk::SharingMode::CONCURRENT).queue_family_indices(&queue_indices)
        } else {
            create_info_builder.image_sharing_mode(vk::SharingMode::EXCLUSIVE)
        };

        let create_info = if let Some(old_swapchain) = old_swapchain {
            create_info_builder
                .pre_transform(swapchain_support.capabilities.current_transform)
                .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                .present_mode(present_mode)
                .old_swapchain(*old_swapchain.vk())
                .clipped(true)
                .build()
        } else {
            create_info_builder
                .pre_transform(swapchain_support.capabilities.current_transform)
                .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                .present_mode(present_mode)
                .clipped(true)
                .build()
        };

        let swapchain_loader = khr::Swapchain::new(device.instance().vk(), device.vk());
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

            unsafe { image_views.push(device.vk().create_image_view(&image_create_info, None).unwrap()) };
        }

        Arc::new(SwapChain {
            device,
            _surface: surface,
            swapchain_loader,
            swapchain,
            surface_format,
            extent,
            images,
            image_views,
        })
    }

    pub fn query_support(device: vk::PhysicalDevice, surface: &Arc<Surface>) -> SwapChainSupportDetails {
        SwapChainSupportDetails {
            capabilities: unsafe { surface.get_loader().get_physical_device_surface_capabilities(device, *surface.vk()).unwrap() },
            formats: unsafe { surface.get_loader().get_physical_device_surface_formats(device, *surface.vk()).unwrap() },
            present_modes: unsafe { surface.get_loader().get_physical_device_surface_present_modes(device, *surface.vk()).unwrap() },
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
            if available_present_mode == vk::PresentModeKHR::IMMEDIATE {
                return available_present_mode;
            }
        }

        availabe_present_modes[0]
    }

    pub fn choose_extent(capabilities: vk::SurfaceCapabilitiesKHR, window: &Window) -> vk::Extent2D {
        if capabilities.current_extent.width != std::u32::MAX {
            capabilities.current_extent
        } else {
            let (width, height) = window.get_window_size();
            vk::Extent2D {
                width: width.max(capabilities.min_image_extent.width).min(capabilities.max_image_extent.width),
                height: height.max(capabilities.min_image_extent.height).min(capabilities.max_image_extent.height),
            }
        }
    }

    pub fn scissor(&self) -> vk::Rect2D {
        vk::Rect2D::builder().offset(vk::Offset2D { x: 0, y: 0 }).extent(*self.extent()).build()
    }

    pub fn viewport(&self) -> vk::Viewport {
        vk::Viewport::builder()
            .x(0f32)
            .y(0f32)
            .width(self.extent().width as f32)
            .height(self.extent().height as f32)
            .min_depth(0f32)
            .max_depth(1f32)
            .build()
    }

    pub fn extent(&self) -> &vk::Extent2D {
        &self.extent
    }

    pub fn surface_format(&self) -> &vk::SurfaceFormatKHR {
        &self.surface_format
    }

    pub fn images(&self) -> &Vec<vk::Image> {
        &self.images
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

    fn vk(&self) -> &Self::Object {
        &self.swapchain
    }
}

impl Drop for SwapChain {
    fn drop(&mut self) {
        trace!("Dropping Swapchain");
        unsafe {
            for &image_view in self.image_views.iter() {
                self.device.vk().destroy_image_view(image_view, None);
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
