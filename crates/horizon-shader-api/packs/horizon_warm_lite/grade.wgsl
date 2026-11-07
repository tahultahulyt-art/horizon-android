// Original MIT implementation. Inspired by BSL style, not BSL code.
fn post(input: PostInput) -> vec4<f32> {
    let c = max(input.color.rgb, vec3<f32>(0.0));
    // Mild bounded shoulder on SDR highlights, preserve black and avoid crushed night scenes.
    let shoulder = c / (vec3<f32>(1.0) + max(c - vec3<f32>(0.72), vec3<f32>(0.0)) * 0.15);
    let grey = dot(shoulder, vec3<f32>(0.2126, 0.7152, 0.0722));
    let rich = mix(vec3<f32>(grey), shoulder, 1.05);
    return vec4<f32>(clamp(rich * vec3<f32>(1.015, 1.0, 0.99), vec3<f32>(0.0), vec3<f32>(1.0)), input.color.a);
}
