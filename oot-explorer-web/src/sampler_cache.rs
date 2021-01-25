use oot_explorer_gl::shader_state::TextureParams;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use wasm_bindgen::UnwrapThrowExt;
use web_sys::{WebGl2RenderingContext, WebGl2RenderingContext as Gl, WebGlSampler};

pub fn opaque_key(params: &TextureParams) -> u32 {
    let mut hasher = DefaultHasher::new();
    params.s.hash(&mut hasher);
    params.t.hash(&mut hasher);
    let hash = hasher.finish();
    ((hash >> 32) ^ hash) as u32
}

fn create_gl_sampler(gl: &WebGl2RenderingContext, _params: &TextureParams) -> WebGlSampler {
    let sampler = gl.create_sampler().unwrap_throw();
    gl.sampler_parameteri(&sampler, Gl::TEXTURE_MAG_FILTER, Gl::NEAREST as i32);
    gl.sampler_parameteri(&sampler, Gl::TEXTURE_MIN_FILTER, Gl::NEAREST as i32);

    gl.sampler_parameteri(&sampler, Gl::TEXTURE_WRAP_S, Gl::CLAMP_TO_EDGE as i32);
    gl.sampler_parameteri(&sampler, Gl::TEXTURE_WRAP_T, Gl::CLAMP_TO_EDGE as i32);

    sampler
}

#[derive(Clone, Default)]
pub struct SamplerCache {
    map: HashMap<u32, WebGlSampler>,
}

impl SamplerCache {
    pub fn new() -> SamplerCache {
        SamplerCache::default()
    }

    pub fn get_or_create<'a>(
        &mut self,
        gl: &WebGl2RenderingContext,
        params: &TextureParams,
    ) -> &WebGlSampler {
        self.map
            .entry(opaque_key(params))
            .or_insert_with(|| create_gl_sampler(gl, params))
    }

    pub fn get_with_key(&self, key: u32) -> Option<&WebGlSampler> {
        self.map.get(&key)
    }
}
