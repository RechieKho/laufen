use super::rendering_server;

pub struct BindGroupContext {
    pub bind_group: rendering_server::BindGroup,
    pub bind_group_layout: rendering_server::BindGroupLayout,
}

pub trait ToBindGroupContext {
    const BIND_GROUP_LAYOUT_DESCRIPTOR : rendering_server::BindGroupLayoutDescriptor<'static>;
    fn to_bind_group_context(&self, p_server: &rendering_server::RenderingServer, p_label: Option<&str>) -> BindGroupContext;

    fn create_bind_group_layout(&self, p_server: &rendering_server::RenderingServer) -> rendering_server::BindGroupLayout {
        p_server.create_bind_group_layout(&Self::BIND_GROUP_LAYOUT_DESCRIPTOR)
    }
}

impl rendering_server::SubmitToRenderPass for [&BindGroupContext] {
    fn submit<'a>(&self, p_render_pass: &mut rendering_server::RenderPass<'a>) {
        for(i, context) in self.iter().enumerate() {
            p_render_pass.set_bind_group(i as u32, &context.bind_group, &[]);
        }
    }
}
