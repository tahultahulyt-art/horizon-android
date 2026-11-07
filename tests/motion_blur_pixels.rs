use glam::Mat4;
use horizon_blur_tests::world_post::Post;
use std::sync::mpsc;
fn main() {
    let instance =
        wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle().with_env());
    let adapter = pollster::block_on(instance.request_adapter(&Default::default()))
        .expect("A host GPU/software adapter is required");
    println!("Host adapter: {:?}", adapter.get_info());
    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default())).unwrap();
    let (w, h) = (640u32, 320u32);
    let desc = |label, usage| wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage,
        view_formats: &[],
    };
    let input = device.create_texture(&desc(
        "Synthetic contrast chart",
        wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
    ));
    let mut pixels = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let rgb = if y < 100 {
                [90, 160, 210]
            } else if y > 245 {
                [60, 95, 48]
            } else if (x / 16 + y / 24) % 2 == 0 {
                [185, 142, 91]
            } else {
                [36, 28, 21]
            };
            let i = ((y * w + x) * 4) as usize;
            pixels[i..i + 3].copy_from_slice(&rgb);
            pixels[i + 3] = 255;
        }
    }
    queue.write_texture(
        input.as_image_copy(),
        &pixels,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(w * 4),
            rows_per_image: Some(h),
        },
        input.size(),
    );
    let mut post = Post::new(&device, wgpu::TextureFormat::Rgba8Unorm);
    assert!(!post.motion_blur);
    assert!(
        post.prepare(&device, [w, h], false).is_none(),
        "OFF must not force an offscreen world pass"
    );
    let projection = Mat4::perspective_rh(1.2, 2.0, 1024.0, 0.05);
    let old = projection * Mat4::from_rotation_y(0.0);
    let turned = projection * Mat4::from_rotation_y(0.05);
    let mut results = Vec::new();
    for mode in ["off", "on", "stationary", "reset"] {
        post.motion_blur = mode != "off";
        post.update_motion(true, old, &queue);
        if mode == "on" {
            post.update_motion(true, turned, &queue);
        }
        if mode == "stationary" {
            post.update_motion(true, old, &queue);
        }
        if mode == "reset" {
            post.update_motion(false, turned, &queue);
            post.update_motion(true, turned, &queue);
        }
        let output = device.create_texture(&desc(
            "Motion blur test output",
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        ));
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (w * h * 4) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let group = post.bind(&device, &input.create_view(&Default::default()));
        let mut enc = device.create_command_encoder(&Default::default());
        post.finish_with(&mut enc, &output.create_view(&Default::default()), &group);
        enc.copy_texture_to_buffer(
            output.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(w * 4),
                    rows_per_image: Some(h),
                },
            },
            output.size(),
        );
        queue.submit([enc.finish()]);
        let (tx, rx) = mpsc::channel();
        buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |r| tx.send(r).unwrap());
        device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
        rx.recv().unwrap().unwrap();
        let data = buffer.slice(..).get_mapped_range().unwrap().to_vec();
        if let Ok(folder) = std::env::var("HORIZON_BLUR_TEST_OUTPUT") {
            std::fs::create_dir_all(&folder).unwrap();
            std::fs::write(
                std::path::Path::new(&folder).join(format!("motion-blur-{mode}.rgba")),
                &data,
            )
            .unwrap();
        }
        results.push(data);
    }
    assert_ne!(
        results[0], results[1],
        "camera-turn blur must change pixels"
    );
    assert_eq!(
        results[0], results[2],
        "stationary camera must remain sharp"
    );
    assert_eq!(
        results[0], results[3],
        "title/pack reset must not smear the new view"
    );
    println!("Pixel tests passed: default OFF, turning blur, stationary sharp, reset sharp");
}
