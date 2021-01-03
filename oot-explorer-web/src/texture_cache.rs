use oot_explorer_core::fs::LazyFileSystem;
use oot_explorer_gl::shader_state::TextureDescriptor;
use oot_explorer_gl::texture::{self, DecodedTexture};
use scoped_owner::ScopedOwner;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use web_sys::{WebGl2RenderingContext, WebGl2RenderingContext as Gl, WebGlTexture};

pub fn opaque_key(descriptor: &TextureDescriptor) -> u32 {
    let mut hasher = DefaultHasher::new();
    descriptor.hash(&mut hasher);
    let hash = hasher.finish();
    ((hash >> 32) ^ hash) as u32
}

fn calc_levels(mut width: usize, mut height: usize) -> i32 {
    let mut result = 1;
    while width > 1 || height > 1 {
        result += 1;
        width = (width / 2).max(1);
        height = (height / 2).max(1);
    }
    result
}

fn create_gl_texture(gl: &WebGl2RenderingContext, decoded: DecodedTexture) -> WebGlTexture {
    let texture = gl.create_texture().unwrap();
    gl.bind_texture(Gl::TEXTURE_2D, Some(&texture));

    gl.tex_storage_2d(
        Gl::TEXTURE_2D,
        calc_levels(decoded.width, decoded.height),
        Gl::RGBA8,
        decoded.width as i32,
        decoded.height as i32,
    );
    gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_u8_array(
        Gl::TEXTURE_2D,
        0,
        0,
        0,
        decoded.width as i32,
        decoded.height as i32,
        Gl::RGBA,
        Gl::UNSIGNED_BYTE,
        Some(&decoded.data),
    )
    .unwrap();
    gl.generate_mipmap(Gl::TEXTURE_2D);

    texture
}

#[derive(Clone, Default)]
pub struct TextureCache {
    map: HashMap<u32, Option<WebGlTexture>>,
}

impl TextureCache {
    pub fn new() -> TextureCache {
        TextureCache::default()
    }

    /// This method returns `Option` because some textures cannot be loaded. Failed load attempts
    /// will be cached.
    pub fn get_or_decode<'a>(
        &mut self,
        gl: &WebGl2RenderingContext,
        scope: &'a ScopedOwner,
        fs: &mut LazyFileSystem<'a>,
        descriptor: &TextureDescriptor,
    ) -> Option<&WebGlTexture> {
        self.map
            .entry(opaque_key(descriptor))
            .or_insert_with(|| match texture::decode(scope, fs, descriptor) {
                Ok(decoded) => Some(create_gl_texture(gl, decoded)),
                Err(e) => {
                    eprintln!(
                        "texture decode error for texture at {:?}: {}",
                        descriptor.source.src(),
                        e,
                    );
                    None
                }
            })
            .as_ref()
    }

    pub fn get_with_key(&self, key: u32) -> Option<&WebGlTexture> {
        self.map
            .get(&key)
            .and_then(|cached_value| cached_value.as_ref())
    }
}
