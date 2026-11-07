//! Opt-in host probe. Exercises real NativeFrame, NativeSky and NativePost, not a stand-in.
use super::{
    native_guard,
    native_post::{NativeFrame, NativePost},
    native_sky::NativeSky,
};
use horizon_shader_api::{Pack, StageKind};
pub fn run(device: &wgpu::Device, queue: &wgpu::Queue, pack: &Pack) -> Result<Vec<u8>, String> {
    native_guard::attempt(device, || {
        let report = horizon_shader_api::check(pack).map_err(|e| format!("{e:?}"))?;
        let format = wgpu::TextureFormat::Rgba8Unorm;
        let frame = NativeFrame::new(device, None, Vec::new(), Default::default(), None);
        let mut inputs = horizon_shader_api::Frame::default();
        inputs.sun_direction = [1.0, 0.1, 0.0];
        queue.write_buffer(&frame.buffer, 0, bytemuck::bytes_of(&inputs));
        let globals = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let mut sky_pipeline = None;
        for stage in &report.stages {
            if stage.kind == StageKind::Sky {
                sky_pipeline = Some(NativeSky::new(
                    device,
                    &globals,
                    &frame,
                    stage,
                    &[],
                    format,
                )?);
            }
        }
        let mut g: super::Globals = bytemuck::Zeroable::zeroed();
        g.inverse_view = glam::Mat4::IDENTITY.to_cols_array_2d();
        g.misc = [0.0, 0.0, 1.0, 1.0];
        g.fog_color = [0.2, 0.4, 0.6, 1.0];
        g.sky_color = [0.5, 0.7, 1.0, 1.0];
        g.fog_ends = [64.0, 0.0, 0.0, 0.0];
        let globals_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: size_of::<super::Globals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&globals_buffer, 0, bytemuck::bytes_of(&g));
        let globals_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &globals,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: globals_buffer.as_entire_binding(),
            }],
        });
        let posts = report
            .stages
            .into_iter()
            .filter(|s| s.kind == StageKind::Post)
            .collect();
        let mut post = NativePost::new(device, posts, &report.manifest, &[], format, &frame)?;
        let texture = |format, usage| {
            device.create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width: 512,
                    height: 256,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage,
                view_formats: &[],
            })
        };
        let color = texture(
            format,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        );
        let mut pixels = vec![0u8; 512 * 256 * 4];
        for y in 0..256 {
            for x in 0..512 {
                let p = (y * 512 + x) * 4;
                pixels[p] = ((x % 256) as u8).saturating_add(0);
                pixels[p + 1] = (y as u8) / 2 + 64;
                pixels[p + 2] = 160 - ((x % 256) as u8) / 4;
                pixels[p + 3] = 255;
            }
        }
        queue.write_texture(
            color.as_image_copy(),
            &pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(2048),
                rows_per_image: Some(256),
            },
            color.size(),
        );
        let depth = texture(
            wgpu::TextureFormat::Depth32Float,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
        );
        let sky_output = texture(
            format,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        );
        let output = texture(
            format,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        );
        let mut encoder = device.create_command_encoder(&Default::default());
        {
            let view = depth.create_view(&Default::default());
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0.5),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });
        }
        {
            let view = sky_output.create_view(&Default::default());
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth.create_view(&Default::default()),
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });
            pass.set_bind_group(0, &globals_group, &[]);
            if let Some(sky) = sky_pipeline {
                sky.draw(&mut pass, &frame);
            }
        }
        post.run(
            device,
            &mut encoder,
            (
                &color.create_view(&Default::default()),
                &depth.create_view(&Default::default()),
            ),
            &output.create_view(&Default::default()),
            [512, 256],
            true,
            None,
        );
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 512 * 256 * 4,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        encoder.copy_texture_to_buffer(
            output.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(2048),
                    rows_per_image: Some(256),
                },
            },
            output.size(),
        );
        let sky_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 512 * 256 * 4,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        encoder.copy_texture_to_buffer(
            sky_output.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &sky_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(2048),
                    rows_per_image: Some(256),
                },
            },
            sky_output.size(),
        );
        queue.submit([encoder.finish()]);
        let (tx, rx) = std::sync::mpsc::channel();
        sky_buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |r| {
                let _ = tx.send(r);
            });
        device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|e| e.to_string())?;
        rx.recv()
            .map_err(|e| e.to_string())?
            .map_err(|e| e.to_string())?;
        let sky_pixels = sky_buffer
            .slice(..)
            .get_mapped_range()
            .map_err(|e| e.to_string())?
            .to_vec();
        assert!(sky_pixels.chunks_exact(4).all(|p| p[3] == 255));
        std::fs::write("/downloads/warm-lite-native-sky-gl.rgba", sky_pixels)
            .map_err(|e| e.to_string())?;
        let (tx, rx) = std::sync::mpsc::channel();
        buffer.slice(..).map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|e| e.to_string())?;
        rx.recv()
            .map_err(|e| e.to_string())?
            .map_err(|e| e.to_string())?;
        let result = buffer
            .slice(..)
            .get_mapped_range()
            .map_err(|e| e.to_string())?
            .to_vec();
        assert!(result.chunks_exact(4).all(|p| p[3] == 255));
        Ok(result)
    })
}
pub fn injected_panic_is_recoverable(device: &wgpu::Device) -> bool {
    native_guard::attempt::<()>(device, || panic!("injected shader activation failure")).is_err()
}
pub fn injected_validation_is_recoverable(device: &wgpu::Device) -> bool {
    native_guard::attempt(device, || {
        let _ = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 4,
            usage: wgpu::BufferUsages::empty(),
            mapped_at_creation: false,
        });
        Ok(())
    })
    .is_err()
}
