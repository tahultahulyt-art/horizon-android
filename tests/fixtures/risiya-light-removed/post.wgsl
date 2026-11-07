// MIT. Original small colour grade using Horizon's public native pack API.
fn post(input: PostInput) -> vec4<f32> {
    let grey = dot(input.color.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
    let vivid = mix(vec3<f32>(grey), input.color.rgb, 1.06);
    return vec4<f32>(clamp(vivid * vec3<f32>(1.015, 1.0, 0.985), vec3<f32>(0.0), vec3<f32>(1.0)), input.color.a);
}
