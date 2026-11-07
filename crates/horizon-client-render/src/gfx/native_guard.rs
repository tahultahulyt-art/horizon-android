//! Catch recoverable native pack construction errors without leaving error scopes stacked.
/// Driver signals/OS kills cannot be caught here; the disk journal handles the next launch.
pub fn attempt<T>(
    device: &wgpu::Device,
    make: impl FnOnce() -> Result<T, String>,
) -> Result<T, String> {
    let validation = device.push_error_scope(wgpu::ErrorFilter::Validation);
    let internal = device.push_error_scope(wgpu::ErrorFilter::Internal);
    let memory = device.push_error_scope(wgpu::ErrorFilter::OutOfMemory);
    let result = crate::crash::contain(make).and_then(|r| r);
    let errors = [
        pollster::block_on(memory.pop()),
        pollster::block_on(internal.pop()),
        pollster::block_on(validation.pop()),
    ];
    if let Some(error) = errors.into_iter().flatten().next() {
        return Err(error.to_string());
    }
    result
}
