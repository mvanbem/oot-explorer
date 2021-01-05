use crate::shader_state::{ShaderState, TextureState};

#[derive(Clone)]
pub struct Batch {
    pub fragment_shader: String,
    pub vertex_data: Vec<u8>,
    pub translucent: bool,
    pub textures: Vec<TextureState>,
    pub z_upd: bool,
}

impl Batch {
    pub fn for_shader_state(shader_state: &ShaderState, translucent: bool) -> Batch {
        let mut textures = vec![];
        if let Some(texture) = shader_state.texture_0.as_ref() {
            textures.push(texture.clone());
            if let Some(texture) = shader_state.texture_1.as_ref() {
                textures.push(texture.clone());
            }
        }

        Batch {
            fragment_shader: shader_state.to_glsl(),
            vertex_data: vec![],
            translucent,
            textures,
            z_upd: shader_state.z_upd,
        }
    }
}
