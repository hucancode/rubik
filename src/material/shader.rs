use std::borrow::Cow;
use wgpu::{Device, ShaderModule};
pub struct Shader {
    pub module: ShaderModule,
}

impl Shader {
    pub fn new(device: &Device, source: &str) -> Self {
        Self {
            module: device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(source)),
            }),
        }
    }
}
