use horizon_client_render::{
    hud::{HudAtlas, HudBatch},
    waypoint_labels,
};
fn main() {
    let root = std::env::args().nth(1).expect("resource root");
    let atlas = HudAtlas::build(std::path::Path::new(&root)).unwrap();
    let mut batch = HudBatch::new([1280, 720]);
    for (name, distance, color, at) in [
        ("home", 127.0, 0x55FFFF, [210.0, 90.0]),
        ("mine", 1250.0, 0xFF77BB, [90.0, 150.0]),
        ("far_base_with_long_name", 973.0, 0xFFFF55, [420.0, 225.0]),
        ("spawn", 17.0, 0x55FF55, [2.0, 5.0]),
    ] {
        waypoint_labels::label(&mut batch, &atlas, name, distance, color, at);
    }
    let vertices: Vec<_> = batch
        .vertices
        .iter()
        .map(|v| serde_json::json!({"pos":v.pos,"uv":v.uv,"color":v.color}))
        .collect();
    std::fs::write(
        "/tmp/wp-vertices.json",
        serde_json::to_vec(
            &serde_json::json!({"width":batch.width,"height":batch.height,"vertices":vertices}),
        )
        .unwrap(),
    )
    .unwrap();
    let mut encoder = png::Encoder::new(
        std::fs::File::create("/tmp/wp-atlas.png").unwrap(),
        atlas.width,
        atlas.height,
    );
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .unwrap()
        .write_image_data(&atlas.pixels)
        .unwrap();
    // Cost benchmark for the shared projection helper, empty vs 32 points. Not a phone FPS measurement.
    let matrix = glam::Mat4::perspective_rh(90f32.to_radians(), 2.0, 0.05, 4096.0);
    let start = std::time::Instant::now();
    for _ in 0..100000 {
        for i in 0..32 {
            std::hint::black_box(waypoint_labels::project(
                glam::Vec3::new(i as f32, 0.0, -100.0),
                matrix,
                427.0,
                240.0,
            ));
        }
    }
    println!(
        "32 point projections: {:.3} us/frame (debug desktop CPU, not GPU/phone FPS)",
        start.elapsed().as_secs_f64() * 1e6 / 100000.0
    );
}
