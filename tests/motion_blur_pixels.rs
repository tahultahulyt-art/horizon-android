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
    // A contrast chart; `shift` slides it sideways like a camera turn would.
    let chart = |shift: u32| {
        let mut pixels = vec![0u8; (w * h * 4) as usize];
        for y in 0..h {
            for x in 0..w {
                let sx = x.wrapping_add(shift);
                let rgb = if y < 100 {
                    [90, 160, 210]
                } else if y > 245 {
                    [60, 95, 48]
                } else if (sx / 16 + y / 24) % 2 == 0 {
                    [185, 142, 91]
                } else {
                    [36, 28, 21]
                };
                let i = ((y * w + x) * 4) as usize;
                pixels[i..i + 3].copy_from_slice(&rgb);
                pixels[i + 3] = 255;
            }
        }
        pixels
    };
    let input = |shift: u32| {
        let texture = device.create_texture(&desc(
            "Synthetic contrast chart",
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        ));
        queue.write_texture(
            texture.as_image_copy(),
            &chart(shift),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(w * 4),
                rows_per_image: Some(h),
            },
            texture.size(),
        );
        texture.create_view(&Default::default())
    };
    let view_a = input(0);
    // Shift 8 flips the checker parity on half the columns, so the two views differ.
    let view_b = input(8);
    let mut post = Post::new(&device, wgpu::TextureFormat::Rgba8Unorm);
    assert!(!post.motion_blur);
    assert!(
        post.prepare(&device, [w, h], false).is_none(),
        "OFF must not force an offscreen world pass"
    );
    let projection = Mat4::perspective_rh(1.2, 2.0, 1024.0, 0.05);
    let old = projection * Mat4::from_rotation_y(0.0);
    let turned = projection * Mat4::from_rotation_y(0.05);
    let flipped = projection * Mat4::from_rotation_y(0.05 + std::f32::consts::PI);
    let cam = glam::DVec3::ZERO;
    let capture = |post: &mut Post, source: &wgpu::TextureView, label: &str| {
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
        let mut enc = device.create_command_encoder(&Default::default());
        if post.blur_active() {
            let out = output.create_view(&Default::default());
            post.blur_frame(&device, &queue, &mut enc, source, &out);
        } else {
            let group = post.bind(&device, source);
            post.finish_with(&mut enc, &output.create_view(&Default::default()), &group);
        }
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
                std::path::Path::new(&folder).join(format!("motion-blur-{label}.rgba")),
                &data,
            )
            .unwrap();
        }
        data
    };
    let max_diff = |a: &[u8], b: &[u8]| {
        a.iter().zip(b).map(|(x, y)| x.abs_diff(*y)).max().unwrap_or(0)
    };
    // Blur OFF: plain copies.
    post.motion_blur = false;
    post.update_motion(true, old, cam, &queue);
    let ref_a = capture(&mut post, &view_a, "off-a");
    let ref_b = capture(&mut post, &view_b, "off-b");
    // Blur ON: the first frame has no history and stays sharp...
    post.motion_blur = true;
    post.update_motion(true, old, cam, &queue);
    post.prepare(&device, [w, h], false);
    let first = capture(&mut post, &view_a, "on-first");
    // ...and the turn blends the new view with the previous one.
    post.update_motion(true, turned, cam, &queue);
    let blended = capture(&mut post, &view_b, "on-turned");
    // A stationary camera converges back to sharp.
    for _ in 0..12 {
        post.update_motion(true, turned, cam, &queue);
        capture(&mut post, &view_b, "stationary-settle");
    }
    post.update_motion(true, turned, cam, &queue);
    let steady = capture(&mut post, &view_b, "stationary");
    // A title/pack frame drops the history; the next frame is sharp.
    post.update_motion(false, turned, cam, &queue);
    post.update_motion(true, turned, cam, &queue);
    let reset = capture(&mut post, &view_b, "reset");
    // A camera cut (F5's flip, a teleport) must not smear the old view in.
    post.update_motion(true, turned, cam, &queue);
    capture(&mut post, &view_b, "cut-warmup");
    post.update_motion(true, flipped, cam, &queue);
    let cut = capture(&mut post, &view_b, "cut");
    post.update_motion(true, flipped, cam, &queue);
    capture(&mut post, &view_b, "jump-warmup");
    post.update_motion(true, flipped, cam + glam::DVec3::new(10.0, 0.0, 0.0), &queue);
    let jumped = capture(&mut post, &view_b, "jump");
    assert_eq!(first, ref_a, "the first frame has no history: sharp");
    assert!(
        max_diff(&blended, &ref_b) > 8,
        "turning blends the new view with the previous frame"
    );
    assert!(
        max_diff(&steady, &ref_b) <= 1,
        "a stationary camera converges to sharp (diff {})",
        max_diff(&steady, &ref_b)
    );
    assert_eq!(reset, ref_b, "title/pack reset must not smear the new view");
    assert_eq!(cut, ref_b, "a camera flip must not smear the old view in");
    assert_eq!(jumped, ref_b, "a teleport must not smear the old view in");
    println!(
        "Pixel tests passed: default OFF, first frame sharp, turning blends, stationary sharp, gap/flip/teleport resets sharp"
    );
}
