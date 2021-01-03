use oot_explorer_gl::shader_state::TextureUsage;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use web_sys::{WebGl2RenderingContext, WebGl2RenderingContext as Gl, WebGlSampler};

pub fn opaque_key(usage: &TextureUsage) -> u32 {
    let mut hasher = DefaultHasher::new();
    usage.params_s.hash(&mut hasher);
    usage.params_t.hash(&mut hasher);
    let hash = hasher.finish();
    ((hash >> 32) ^ hash) as u32
}

fn create_gl_sampler(gl: &WebGl2RenderingContext, usage: &TextureUsage) -> WebGlSampler {
    let sampler = gl.create_sampler().unwrap();
    gl.sampler_parameteri(&sampler, Gl::TEXTURE_MAG_FILTER, Gl::LINEAR as i32);
    gl.sampler_parameteri(
        &sampler,
        Gl::TEXTURE_MIN_FILTER,
        Gl::LINEAR_MIPMAP_LINEAR as i32,
    );

    if usage.params_s.mirror {
        gl.sampler_parameteri(&sampler, Gl::TEXTURE_WRAP_S, Gl::MIRRORED_REPEAT as i32);
    } else {
        gl.sampler_parameteri(&sampler, Gl::TEXTURE_WRAP_S, Gl::REPEAT as i32);
    }
    if usage.params_t.mirror {
        gl.sampler_parameteri(&sampler, Gl::TEXTURE_WRAP_T, Gl::MIRRORED_REPEAT as i32);
    } else {
        gl.sampler_parameteri(&sampler, Gl::TEXTURE_WRAP_T, Gl::REPEAT as i32);
    }

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
        usage: &TextureUsage,
    ) -> &WebGlSampler {
        self.map
            .entry(opaque_key(usage))
            .or_insert_with(|| create_gl_sampler(gl, usage))
    }

    pub fn get_with_key(&self, key: u32) -> Option<&WebGlSampler> {
        self.map.get(&key)
    }
}
