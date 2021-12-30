use super::GraphicsContext;

pub struct Shader<'a> {
    pub module: wgpu::ShaderModule,
    pub vs_name: &'a str,
    pub fs_name: &'a str,
}

impl<'a> Shader<'a> {
    pub fn new<'temp>(
        gfx: &GraphicsContext,
        source: &'temp str,
        vs_name: &'a str,
        fs_name: &'a str,
    ) -> Self {
        let module = gfx
            .device
            .create_shader_module(&wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(source.into()),
            });
        Shader {
            module,
            vs_name,
            fs_name,
        }
    }
}
