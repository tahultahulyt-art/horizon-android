// Original MIT implementation. Inspired by BSL style, not BSL code.
fn sky(input: SkyInput) -> vec4<f32> {
    let day = smoothstep(-0.08, 0.2, frame.sun_direction.y);
    let horizon = pow(max(0.0, 1.0 - abs(input.direction.y)), 3.0);
    let sunset = (1.0 - smoothstep(0.05, 0.45, abs(frame.sun_direction.y))) * day;
    let strength = horizon * sunset * (1.0 - frame.rain) * 0.16;
    let allowed = select(0.0, 1.0, frame.dimension == 0);
    return vec4<f32>(mix(input.color.rgb, vec3<f32>(0.95, 0.60, 0.38), strength * allowed), input.color.a);
}
