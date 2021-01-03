use crate::shader_state::{ShaderState, TextureDescriptor};

#[derive(Clone)]
pub struct Batch {
    pub fragment_shader: String,
    pub vertex_data: Vec<u8>,
    pub textures: Vec<TextureDescriptor>,
}

impl Batch {
    pub fn for_shader_state(shader_state: &ShaderState) -> Batch {
        let mut textures = vec![];
        if let Some(descriptor) = shader_state.texture_a.as_ref() {
            textures.push(descriptor.clone());
            if let Some(descriptor) = shader_state.texture_b.as_ref() {
                textures.push(descriptor.clone());
            }
        }

        Batch {
            fragment_shader: shader_state.to_glsl(),
            vertex_data: vec![],
            textures,
        }
    }
}
