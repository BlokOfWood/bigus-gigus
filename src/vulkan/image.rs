use std::ffi::c_void;

use ash::vk::{
    AccessFlags, BorderColor, BufferUsageFlags, CommandPool, CompareOp, DependencyFlags,
    DeviceMemory, Extent3D, Filter, Format, FormatFeatureFlags, Image, ImageAspectFlags, ImageBlit,
    ImageCreateInfo, ImageLayout, ImageMemoryBarrier, ImageSubresourceLayers,
    ImageSubresourceRange, ImageTiling, ImageType, ImageUsageFlags, ImageView, ImageViewCreateInfo,
    ImageViewType, MemoryAllocateInfo, MemoryMapFlags, MemoryPropertyFlags, Offset3D,
    PipelineStageFlags, SampleCountFlags, Sampler, SamplerAddressMode, SamplerCreateInfo,
    SamplerMipmapMode, SharingMode, FALSE, LOD_CLAMP_NONE, QUEUE_FAMILY_IGNORED, TRUE,
};

use super::device_and_queues::BigusDevice;

impl BigusDevice {
    pub fn create_texture_image(
        &self,
        image_path: &str,
        command_pool: &CommandPool,
    ) -> (Image, u32) {
        let open_image = image::open(image_path).unwrap();

        let image_extent = [open_image.width(), open_image.height()];
        let mip_levels = image_extent[0].max(image_extent[1]).ilog2();

        let image_data = open_image.to_rgba8();

        let (staging_buffer, staging_buffer_memory) = self.create_buffer(
            image_data.len() as u64,
            BufferUsageFlags::TRANSFER_SRC,
            MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
        );

        unsafe {
            let data = self
                .dev
                .map_memory(
                    staging_buffer_memory,
                    0,
                    image_data.len() as u64,
                    MemoryMapFlags::empty(),
                )
                .unwrap();
            data.copy_from(image_data.as_ptr() as *mut c_void, image_data.len());

            self.dev.unmap_memory(staging_buffer_memory);
        };

        drop(open_image);

        let (texture_image, _texture_image_memory) = self.create_image(
            image_extent,
            mip_levels,
            SampleCountFlags::TYPE_1,
            Format::R8G8B8A8_SRGB,
            ImageTiling::OPTIMAL,
            ImageUsageFlags::TRANSFER_SRC
                | ImageUsageFlags::TRANSFER_DST
                | ImageUsageFlags::SAMPLED,
            MemoryPropertyFlags::DEVICE_LOCAL,
        );

        self.transition_image_layout(
            command_pool,
            ImageLayout::UNDEFINED,
            ImageLayout::TRANSFER_DST_OPTIMAL,
            texture_image,
            mip_levels,
        );

        self.copy_buffer_to_image(
            staging_buffer,
            texture_image,
            command_pool,
            image_extent[0],
            image_extent[1],
        );

        self.generate_mipmaps(
            command_pool,
            texture_image,
            Format::R8G8B8A8_SRGB,
            image_extent,
            mip_levels,
        );

        /*  self.transition_image_layout(
            command_pool,
            ImageLayout::TRANSFER_DST_OPTIMAL,
            ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            texture_image,
            mip_levels,
        ); */

        unsafe {
            self.dev.destroy_buffer(staging_buffer, None);
            self.dev.free_memory(staging_buffer_memory, None);
        };

        (texture_image, mip_levels)
    }

    fn generate_mipmaps(
        &self,
        command_pool: &CommandPool,
        image: Image,
        image_format: Format,
        extent: [u32; 2],
        mip_levels: u32,
    ) {
        let format_properties = unsafe {
            self.instance
                .get_physical_device_format_properties(self.phys_dev, image_format)
        };

        if !format_properties
            .optimal_tiling_features
            .contains(FormatFeatureFlags::SAMPLED_IMAGE_FILTER_LINEAR)
        {
            panic!("Texture image format does not support linear blitting!")
        }

        let command_buffer = self.begin_single_time_commands(command_pool);

        let mut barrier = ImageMemoryBarrier {
            image,
            src_queue_family_index: QUEUE_FAMILY_IGNORED,
            dst_queue_family_index: QUEUE_FAMILY_IGNORED,
            subresource_range: ImageSubresourceRange {
                aspect_mask: ImageAspectFlags::COLOR,
                base_array_layer: 0,
                level_count: 1,
                layer_count: 1,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut mip_width = extent[0];
        let mut mip_height = extent[1];

        for i in 1..mip_levels {
            barrier.subresource_range.base_mip_level = i - 1;
            barrier.old_layout = ImageLayout::TRANSFER_DST_OPTIMAL;
            barrier.new_layout = ImageLayout::TRANSFER_SRC_OPTIMAL;
            barrier.src_access_mask = AccessFlags::TRANSFER_WRITE;
            barrier.dst_access_mask = AccessFlags::TRANSFER_READ;

            unsafe {
                self.dev.cmd_pipeline_barrier(
                    command_buffer,
                    PipelineStageFlags::TRANSFER,
                    PipelineStageFlags::TRANSFER,
                    DependencyFlags::empty(),
                    &[],
                    &[],
                    &[barrier],
                );

                let new_mip_width: i32 = if mip_width > 1 {
                    mip_width as i32 / 2
                } else {
                    1
                };
                let new_mip_height: i32 = if mip_height > 1 {
                    mip_height as i32 / 2
                } else {
                    1
                };

                self.dev.cmd_blit_image(
                    command_buffer,
                    image,
                    ImageLayout::TRANSFER_SRC_OPTIMAL,
                    image,
                    ImageLayout::TRANSFER_DST_OPTIMAL,
                    &[ImageBlit {
                        src_offsets: [
                            Offset3D { x: 0, y: 0, z: 0 },
                            Offset3D {
                                x: mip_width as i32,
                                y: mip_height as i32,
                                z: 1,
                            },
                        ],
                        src_subresource: ImageSubresourceLayers {
                            aspect_mask: ImageAspectFlags::COLOR,
                            mip_level: i - 1,
                            base_array_layer: 0,
                            layer_count: 1,
                        },
                        dst_offsets: [
                            Offset3D { x: 0, y: 0, z: 0 },
                            Offset3D {
                                x: new_mip_width,
                                y: new_mip_height,
                                z: 1,
                            },
                        ],
                        dst_subresource: ImageSubresourceLayers {
                            aspect_mask: ImageAspectFlags::COLOR,
                            mip_level: i,
                            base_array_layer: 0,
                            layer_count: 1,
                        },
                    }],
                    Filter::LINEAR,
                );

                barrier.old_layout = ImageLayout::TRANSFER_SRC_OPTIMAL;
                barrier.new_layout = ImageLayout::SHADER_READ_ONLY_OPTIMAL;
                barrier.src_access_mask = AccessFlags::TRANSFER_READ;
                barrier.dst_access_mask = AccessFlags::SHADER_READ;

                self.dev.cmd_pipeline_barrier(
                    command_buffer,
                    PipelineStageFlags::TRANSFER,
                    PipelineStageFlags::FRAGMENT_SHADER,
                    DependencyFlags::empty(),
                    &[],
                    &[],
                    &[barrier],
                );

                mip_width = new_mip_width as u32;
                mip_height = new_mip_height as u32;

                barrier.subresource_range.base_mip_level = mip_levels - 1;
                barrier.old_layout = ImageLayout::TRANSFER_DST_OPTIMAL;
                barrier.new_layout = ImageLayout::SHADER_READ_ONLY_OPTIMAL;
                barrier.src_access_mask = AccessFlags::TRANSFER_WRITE;
                barrier.dst_access_mask = AccessFlags::SHADER_READ;

                self.dev.cmd_pipeline_barrier(
                    command_buffer,
                    PipelineStageFlags::TRANSFER,
                    PipelineStageFlags::FRAGMENT_SHADER,
                    DependencyFlags::empty(),
                    &[],
                    &[],
                    &[barrier],
                );
            };
        }

        self.end_single_time_commands(command_pool, command_buffer);
    }

    fn transition_image_layout(
        &self,
        command_pool: &CommandPool,
        old_layout: ImageLayout,
        new_layout: ImageLayout,
        image: Image,
        mip_levels: u32,
    ) {
        let command_buffer = self.begin_single_time_commands(command_pool);

        let barrier = ImageMemoryBarrier {
            old_layout,
            new_layout,
            src_queue_family_index: QUEUE_FAMILY_IGNORED,
            dst_queue_family_index: QUEUE_FAMILY_IGNORED,
            image,
            subresource_range: ImageSubresourceRange {
                aspect_mask: ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: mip_levels,
                base_array_layer: 0,
                layer_count: 1,
            },
            ..Default::default()
        };

        if old_layout == ImageLayout::UNDEFINED && new_layout == ImageLayout::TRANSFER_DST_OPTIMAL {
            unsafe {
                self.dev.cmd_pipeline_barrier(
                    command_buffer,
                    PipelineStageFlags::TOP_OF_PIPE,
                    PipelineStageFlags::TRANSFER,
                    DependencyFlags::empty(),
                    &[],
                    &[],
                    &[barrier
                        .src_access_mask(AccessFlags::empty())
                        .dst_access_mask(AccessFlags::TRANSFER_WRITE)],
                )
            };
        } else {
            unsafe {
                self.dev.cmd_pipeline_barrier(
                    command_buffer,
                    PipelineStageFlags::TRANSFER,
                    PipelineStageFlags::FRAGMENT_SHADER,
                    DependencyFlags::empty(),
                    &[],
                    &[],
                    &[barrier
                        .src_access_mask(AccessFlags::TRANSFER_WRITE)
                        .dst_access_mask(AccessFlags::SHADER_READ)],
                )
            }
        }

        self.end_single_time_commands(command_pool, command_buffer);
    }

    pub fn create_texture_image_view(&self, image: Image, mip_levels: u32) -> ImageView {
        self.create_image_view(
            image,
            Format::R8G8B8A8_SRGB,
            ImageAspectFlags::COLOR,
            mip_levels,
        )
    }

    pub fn create_texture_sampler(&self) -> Sampler {
        unsafe {
            self.dev
                .create_sampler(
                    &SamplerCreateInfo {
                        mag_filter: Filter::LINEAR,
                        min_filter: Filter::LINEAR,
                        address_mode_u: SamplerAddressMode::REPEAT,
                        address_mode_v: SamplerAddressMode::REPEAT,
                        address_mode_w: SamplerAddressMode::REPEAT,
                        anisotropy_enable: TRUE,
                        max_anisotropy: self.phys_dev_capabilities.limits.max_sampler_anisotropy,
                        border_color: BorderColor::INT_OPAQUE_BLACK,
                        unnormalized_coordinates: FALSE,
                        compare_enable: FALSE,
                        compare_op: CompareOp::ALWAYS,
                        mipmap_mode: SamplerMipmapMode::LINEAR,
                        max_lod: LOD_CLAMP_NONE,
                        ..Default::default()
                    },
                    None,
                )
                .unwrap()
        }
    }

    pub fn create_image(
        &self,
        extent: [u32; 2],
        mip_levels: u32,
        samples: SampleCountFlags,
        format: Format,
        tiling: ImageTiling,
        usage: ImageUsageFlags,
        properties: MemoryPropertyFlags,
    ) -> (Image, DeviceMemory) {
        let image_info = ImageCreateInfo {
            image_type: ImageType::TYPE_2D,
            extent: Extent3D {
                width: extent[0],
                height: extent[1],
                depth: 1,
            },
            mip_levels,
            array_layers: 1,
            format,
            tiling,
            initial_layout: ImageLayout::UNDEFINED,
            usage,
            samples,
            sharing_mode: SharingMode::EXCLUSIVE,
            ..Default::default()
        };

        let image = unsafe { self.dev.create_image(&image_info, None).unwrap() };

        let mem_requirements = unsafe { self.dev.get_image_memory_requirements(image) };

        let alloc_info = MemoryAllocateInfo {
            allocation_size: mem_requirements.size,
            memory_type_index: self.find_memory_type(
                &self.instance,
                mem_requirements.memory_type_bits,
                properties,
            ),
            ..Default::default()
        };

        let device_memory = unsafe { self.dev.allocate_memory(&alloc_info, None).unwrap() };

        unsafe { self.dev.bind_image_memory(image, device_memory, 0).unwrap() }

        (image, device_memory)
    }

    pub fn create_image_view(
        &self,
        image: Image,
        format: Format,
        aspect_flags: ImageAspectFlags,
        mip_levels: u32,
    ) -> ImageView {
        let view_info = ImageViewCreateInfo {
            image,
            view_type: ImageViewType::TYPE_2D,
            format,
            subresource_range: ImageSubresourceRange {
                aspect_mask: aspect_flags,
                base_mip_level: 0,
                level_count: mip_levels,
                base_array_layer: 0,
                layer_count: 1,
            },
            ..Default::default()
        };

        return unsafe { self.dev.create_image_view(&view_info, None).unwrap() };
    }

    pub fn create_image_views(
        &self,
        swap_chain_images: Vec<Image>,
        swap_chain_image_format: Format,
        mip_levels: u32,
    ) -> Vec<ImageView> {
        let mut image_views: Vec<ImageView> = Vec::with_capacity(swap_chain_images.len());

        for image in swap_chain_images {
            image_views.push(self.create_image_view(
                image,
                swap_chain_image_format,
                ImageAspectFlags::COLOR,
                mip_levels,
            ));
        }

        image_views
    }

    pub fn create_color_resources(
        &self,
        swap_chain_extent: [u32; 2],
        swap_chain_image_format: Format,
    ) -> (Image, ImageView, DeviceMemory) {
        let color_format = swap_chain_image_format;

        let (color_image, color_image_memory) = self.create_image(
            swap_chain_extent,
            1,
            self.max_sample_count,
            color_format,
            ImageTiling::OPTIMAL,
            ImageUsageFlags::TRANSIENT_ATTACHMENT | ImageUsageFlags::COLOR_ATTACHMENT,
            MemoryPropertyFlags::DEVICE_LOCAL,
        );

        let color_image_view = self.create_image_view(color_image, color_format, ImageAspectFlags::COLOR, 1);

        return (color_image, color_image_view, color_image_memory);
    }

    pub fn create_depth_resources(
        &self,
        swap_chain_extent: [u32; 2],
    ) -> (Image, ImageView, DeviceMemory, Format) {
        let image_format = self.find_depth_format();

        let (depth_image, depth_image_memory) = self.create_image(
            swap_chain_extent,
            1,
            self.max_sample_count,
            image_format,
            ImageTiling::OPTIMAL,
            ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            MemoryPropertyFlags::DEVICE_LOCAL,
        );
        let depth_image_view =
            self.create_image_view(depth_image, image_format, ImageAspectFlags::DEPTH, 1);

        (
            depth_image,
            depth_image_view,
            depth_image_memory,
            image_format,
        )
    }

    fn find_depth_format(&self) -> Format {
        self.find_supported_format(
            vec![
                Format::D32_SFLOAT,
                Format::D32_SFLOAT_S8_UINT,
                Format::D24_UNORM_S8_UINT,
            ],
            ImageTiling::OPTIMAL,
            FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
        )
    }
}
