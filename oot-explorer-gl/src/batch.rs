use crate::shader_state::ShaderState;

#[derive(Clone)]
pub struct Batch {
    pub fragment_shader: String,
    pub vertex_data: Vec<u8>,
}
impl Batch {
    pub fn for_shader_state(shader_state: &ShaderState) -> Batch {
        Batch {
            fragment_shader: shader_state.to_glsl(),
            vertex_data: vec![],
        }
    }
    pub fn fragment_shader(&self) -> &str {
        &self.fragment_shader
    }
    pub fn vertex_data(&self) -> &[u8] {
        &self.vertex_data
    }
}
