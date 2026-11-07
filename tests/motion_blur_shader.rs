#[path = "../crates/horizon-client-render/src/gfx/world_post.rs"]
pub mod world_post;
#[test]
fn motion_blur_shader_validates() {
    let source = include_str!("../crates/horizon-client-render/src/world_post.wgsl");
    let module = naga::front::wgsl::parse_str(source).unwrap();
    naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    )
    .validate(&module)
    .unwrap();
    assert!(
        module
            .entry_points
            .iter()
            .any(|e| e.name == "fs_motion_blur")
    );
}
#[test]
fn motion_blur_compiles_to_gles310() {
    let source = include_str!("../crates/horizon-client-render/src/world_post.wgsl");
    let module = naga::front::wgsl::parse_str(source).unwrap();
    let info = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    )
    .validate(&module)
    .unwrap();
    let options = naga::back::glsl::Options::default();
    let pipe = naga::back::glsl::PipelineOptions {
        shader_stage: naga::ShaderStage::Fragment,
        entry_point: "fs_motion_blur".into(),
        multiview: None,
    };
    let mut output = String::new();
    naga::back::glsl::Writer::new(
        &mut output,
        &module,
        &info,
        &options,
        &pipe,
        Default::default(),
    )
    .unwrap()
    .write()
    .unwrap();
    assert!(output.starts_with("#version 310 es"));
}
