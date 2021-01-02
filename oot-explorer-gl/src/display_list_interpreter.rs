use byteorder::{NativeEndian, WriteBytesExt};
use oot_explorer_core::gbi::{
    DisplayList, GeometryMode, Instruction, LitVertex, OtherModeH, Qu0_16, Qu10_2, UnlitVertex,
};
use oot_explorer_core::segment::{SegmentAddr, SegmentCtx};
use oot_explorer_core::slice::Slice;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Debug;
use std::io::Write;

use crate::batch::Batch;
use crate::rcp::{
    CombinerState, RcpState, RdpOtherMode, RspTextureState, TextureSource, TileAttributes,
    TileDimensions, Tmem, TmemRegion, TmemSource,
};
use crate::shader_state::{ShaderState, TextureDescriptor};
use crate::{FLAGS_LIT, FLAGS_UNLIT};

#[derive(Clone)]
pub struct DisplayListInterpreter {
    total_dlists: usize,
    total_instructions: usize,
    unmapped_calls: BTreeMap<SegmentAddr, usize>,
    unmapped_textures: BTreeMap<SegmentAddr, usize>,
    max_depth: usize,
    total_lit_verts: usize,
    total_unlit_verts: usize,
    unique_textures: HashSet<TextureDescriptor>,

    batches_by_shader_state: HashMap<ShaderState, Batch>,
}

impl DisplayListInterpreter {
    pub fn new() -> DisplayListInterpreter {
        DisplayListInterpreter {
            total_dlists: 0,
            total_instructions: 0,
            unmapped_calls: BTreeMap::new(),
            unmapped_textures: BTreeMap::new(),
            max_depth: 0,
            total_lit_verts: 0,
            total_unlit_verts: 0,
            unique_textures: HashSet::new(),

            batches_by_shader_state: HashMap::new(),
        }
    }

    pub fn clear_batches(&mut self) {
        self.batches_by_shader_state.clear();
    }

    pub fn interpret<'a>(&mut self, ctx: &SegmentCtx<'a>, dlist: DisplayList<'a>) {
        self.interpret_internal(
            ctx,
            dlist,
            &mut RcpState {
                vertex_slots: [None; 32],
                geometry_mode: GeometryMode::default(),
                rdp_half_1: None,
                rdp_other_mode: RdpOtherMode {
                    hi: OtherModeH::CYC_2CYCLE | OtherModeH::TT_RGBA16,
                },
                combiner: None,
                texture_src: None,
                tiles: Default::default(),
                rsp_texture_state: RspTextureState {
                    max_lod: 0,
                    tile: 0,
                    enable: true,
                    scale_s: Qu0_16(0x8000),
                    scale_t: Qu0_16(0x8000),
                },
                tmem: Tmem::default(),
            },
            1,
        );
    }

    fn interpret_internal<'a>(
        &mut self,
        ctx: &SegmentCtx<'a>,
        dlist: DisplayList<'a>,
        state: &mut RcpState,
        depth: usize,
    ) {
        self.total_dlists += 1;
        self.max_depth = self.max_depth.max(depth);

        dlist.parse(|instruction| {
            self.total_instructions += 1;
            match instruction {
                // 0x00
                Instruction::Noop { .. } => {
                    panic!("Wasn't expecting a Noop instruction. May be interpreting zeroes.");
                }
                // 0x01
                Instruction::Vtx { count, index, ptr } => {
                    // TODO: Use more RCP state. Matrix stack!

                    // TODO: Dedupe vertices (at least by address, but
                    // maybe by value?).

                    let mut index = index as usize;
                    if state.geometry_mode.test(GeometryMode::LIGHTING) {
                        // Lit vertices
                        self.total_lit_verts += 1;
                        let vertices = Slice::<'a, LitVertex<'a>>::new(
                            ctx.resolve(ptr).unwrap(),
                            count as usize,
                        );
                        for vertex in vertices {
                            state.vertex_slots[index] =
                                Some(DisplayListInterpreter::transform_and_encode_lit_vertex(
                                    &vertex,
                                    state.rsp_texture_state.scale_s,
                                    state.rsp_texture_state.scale_t,
                                ));
                            index = (index + 1) & 0x1f;
                        }
                    } else {
                        // Unlit vertices
                        self.total_unlit_verts += 1;
                        let vertices = Slice::<'a, UnlitVertex<'a>>::new(
                            ctx.resolve(ptr).unwrap(),
                            count as usize,
                        );
                        for vertex in vertices {
                            state.vertex_slots[index] =
                                Some(DisplayListInterpreter::transform_and_encode_unlit_vertex(
                                    &vertex,
                                    state.rsp_texture_state.scale_s,
                                    state.rsp_texture_state.scale_t,
                                ));
                            index = (index + 1) & 0x1f;
                        }
                    }
                }
                // 0x03
                Instruction::CullDl { .. } => {
                    // TODO: Implement box culling.
                }
                // 0x04
                Instruction::BranchZ { .. } => {
                    let addr = SegmentAddr(state.rdp_half_1.unwrap());
                    if let Ok(data) = ctx.resolve(addr) {
                        self.interpret_internal(ctx, DisplayList::new(data), state, depth + 1);
                    } else {
                        *self.unmapped_calls.entry(addr).or_default() += 1;
                    }
                }
                // 0x05
                Instruction::Tri1 { index } => {
                    self.process_triangle(state, index);
                }
                // 0x06
                Instruction::Tri2 { index_a, index_b } => {
                    self.process_triangle(state, index_a);
                    self.process_triangle(state, index_b);
                }
                // 0xd7
                Instruction::Texture {
                    max_lod,
                    tile,
                    enable,
                    scale_s,
                    scale_t,
                } => {
                    state.rsp_texture_state = RspTextureState {
                        max_lod,
                        tile,
                        enable,
                        scale_s,
                        scale_t,
                    };
                }
                // 0xd9
                Instruction::GeometryMode {
                    clear_bits,
                    set_bits,
                } => {
                    state.geometry_mode &= !clear_bits;
                    state.geometry_mode |= set_bits;
                }
                // 0xda
                Instruction::Mtx { .. } => {
                    // TODO: track matrix stack state
                }
                // 0xde
                Instruction::Dl { jump, ptr } => {
                    if let Ok(data) = ctx.resolve(ptr) {
                        self.interpret_internal(ctx, DisplayList::new(data), state, depth + 1);
                    } else {
                        *self.unmapped_calls.entry(ptr).or_default() += 1;
                    }
                    if jump {
                        return;
                    }
                }
                // 0xdf
                Instruction::EndDl => {
                    // DisplayList::parse() should have already interpreted these to end parsing.
                    unreachable!()
                }
                // 0xe1
                Instruction::RdpHalf1 { word } => state.rdp_half_1 = Some(word),
                // 0xe2
                Instruction::SetOtherModeL { .. } => {
                    // TODO: track any relevant bits in the low other mode word
                }
                // 0xe3
                Instruction::SetOtherModeH {
                    clear_bits,
                    set_bits,
                } => {
                    state.rdp_other_mode.hi &= !clear_bits;
                    state.rdp_other_mode.hi |= set_bits;
                }
                // 0xe6
                Instruction::RdpLoadSync => (),
                // 0xe7
                Instruction::RdpPipeSync => (),
                // 0xe8
                Instruction::RdpTileSync => (),
                // 0xf0
                Instruction::LoadTlut { tile, count } => {
                    state.tiles[tile as usize].dimensions = None;

                    if let Some(texture_src) = state.texture_src.as_ref() {
                        let range = 256..256 + count;
                        let source = TmemSource::LoadTlut {
                            ptr: texture_src.ptr,
                            count,
                        };
                        state.tmem.overlay(TmemRegion { range, source });
                    } else {
                        // Source attributes are not sufficiently defined to determine the
                        // destination range. Invalidate the whole TMEM.
                        state.tmem = Tmem::default();
                    }
                }
                // 0xf2
                Instruction::SetTileSize {
                    start_s,
                    start_t,
                    tile,
                    end_s,
                    end_t,
                } => {
                    state.tiles[tile as usize].dimensions = Some(TileDimensions {
                        s: start_s..end_s,
                        t: start_t..end_t,
                    });
                }
                // 0xf3
                Instruction::LoadBlock {
                    start_s,
                    start_t,
                    tile,
                    texels,
                    dxt,
                } => {
                    // Haven't figured out how to handle these yet.
                    assert_eq!(start_s, Qu10_2(0));
                    assert_eq!(start_t, Qu10_2(0));

                    state.tiles[tile as usize].dimensions = None;

                    let tile = &state.tiles[tile as usize];
                    let tile_attributes = tile.attributes.as_ref();
                    if let (Some(texture_src), Some(tile_attributes)) =
                        (state.texture_src.as_ref(), tile_attributes)
                    {
                        let len: u16 = texels / texture_src.depth.texels_per_tmem_word::<u16>();
                        let range = tile_attributes.addr..tile_attributes.addr + len;
                        let source = TmemSource::LoadBlock {
                            src_ptr: texture_src.ptr,
                            src_format: texture_src.format,
                            src_depth: texture_src.depth,
                            load_dxt: dxt,
                            load_texels: texels,
                            load_format: tile_attributes.format,
                            load_depth: tile_attributes.depth,
                        };
                        state.tmem.overlay(TmemRegion { range, source });
                    } else {
                        // Tile parameters and/or source attributes are not sufficiently defined to
                        // determine the destination range. Invalidate the whole TMEM.
                        state.tmem = Tmem::default();
                    }
                }
                // 0xf5
                Instruction::SetTile {
                    format,
                    depth,
                    stride,
                    addr,
                    tile,
                    palette,
                    clamp_t,
                    mirror_t,
                    mask_t,
                    shift_t,
                    clamp_s,
                    mirror_s,
                    mask_s,
                    shift_s,
                } => {
                    state.tiles[(tile & 0x7) as usize].attributes = Some(TileAttributes {
                        format,
                        depth,
                        stride,
                        addr,
                        palette,
                        clamp_t,
                        mirror_t,
                        mask_t,
                        shift_t,
                        clamp_s,
                        mirror_s,
                        mask_s,
                        shift_s,
                    });
                }
                // 0xfa
                Instruction::SetPrimColor { .. } => {
                    // TODO: Track the primitive color
                }
                // 0xfb
                Instruction::SetEnvColor { .. } => {
                    // TODO: Track the environment color
                }
                // 0xfc
                Instruction::SetCombine {
                    color_0,
                    alpha_0,
                    color_1,
                    alpha_1,
                } => {
                    state.combiner = Some(CombinerState {
                        color_0,
                        alpha_0,
                        color_1,
                        alpha_1,
                    });
                }
                // 0xfd
                Instruction::SetTimg {
                    format,
                    depth,
                    width: _,
                    ptr,
                } => {
                    // NOTE: That width value does appear to exist, but is not used by LoadBlock
                    // instructions and I'm not sure Ocarina of Time ends up using it at all.
                    if let Ok(vrom_range) = ctx.resolve_vrom(ptr) {
                        state.texture_src = Some(TextureSource {
                            format,
                            depth,
                            ptr: vrom_range.start,
                        });
                    } else {
                        state.texture_src = None;
                        *self.unmapped_textures.entry(ptr).or_default() += 1;
                    }
                }
            }
        });
    }

    fn transform_texcoord(texcoord: i16, scale: Qu0_16) -> i16 {
        (((texcoord as i32) * (scale.0 as i32)) >> 16) as i16
    }

    fn transform_and_encode_unlit_vertex(
        vertex: &UnlitVertex,
        scale_s: Qu0_16,
        scale_t: Qu0_16,
    ) -> [u8; 20] {
        let mut buf = [0; 20];

        let mut w = &mut buf[..];
        // [0..=5] Position
        let pos = vertex.position();
        w.write_i16::<NativeEndian>(pos[0]).unwrap();
        w.write_i16::<NativeEndian>(pos[1]).unwrap();
        w.write_i16::<NativeEndian>(pos[2]).unwrap();
        // [6..=7] Padding
        w.write_u16::<NativeEndian>(0).unwrap();
        // [8..=10] Normal (unused for unlit geometry)
        w.write_i8(0).unwrap();
        w.write_i8(0).unwrap();
        w.write_i8(0).unwrap();
        // [11] Flags
        w.write_u8(FLAGS_UNLIT).unwrap();
        // [12..=15] Texture coordinates
        let texcoord = vertex.texcoord();
        w.write_i16::<NativeEndian>(DisplayListInterpreter::transform_texcoord(
            texcoord[0],
            scale_s,
        ))
        .unwrap();
        w.write_i16::<NativeEndian>(DisplayListInterpreter::transform_texcoord(
            texcoord[1],
            scale_t,
        ))
        .unwrap(); // [16..=19] Color
        let color = vertex.color();
        w.write_all(&color[..]).unwrap();
        assert_eq!(w.len(), 0);

        buf
    }

    fn transform_and_encode_lit_vertex(
        vertex: &LitVertex,
        scale_s: Qu0_16,
        scale_t: Qu0_16,
    ) -> [u8; 20] {
        let mut buf = [0; 20];

        let mut w = &mut buf[..];
        // [0..=5] Position
        let pos = vertex.position();
        w.write_i16::<NativeEndian>(pos[0]).unwrap();
        w.write_i16::<NativeEndian>(pos[1]).unwrap();
        w.write_i16::<NativeEndian>(pos[2]).unwrap();
        // [6..=7] Padding
        w.write_u16::<NativeEndian>(0).unwrap();
        // [8..=10] Normal
        let normal = vertex.normal();
        w.write_i8(normal[0]).unwrap();
        w.write_i8(normal[1]).unwrap();
        w.write_i8(normal[2]).unwrap();
        // [11] Flags
        w.write_u8(FLAGS_LIT).unwrap();
        // [12..=15] Texture coordinates
        let texcoord = vertex.texcoord();
        w.write_i16::<NativeEndian>(DisplayListInterpreter::transform_texcoord(
            texcoord[0],
            scale_s,
        ))
        .unwrap();
        w.write_i16::<NativeEndian>(DisplayListInterpreter::transform_texcoord(
            texcoord[1],
            scale_t,
        ))
        .unwrap();
        // [16..=19] Color (RGB are unused for lit geometry)
        w.write_u8(0).unwrap();
        w.write_u8(0).unwrap();
        w.write_u8(0).unwrap();
        w.write_u8(vertex.alpha()).unwrap();
        assert_eq!(w.len(), 0);

        buf
    }

    fn process_triangle(&mut self, state: &mut RcpState, index: [u8; 3]) {
        // Print each unique shader state as it is encountered.
        let shader_state = state.shader_state();
        let batch = self
            .batches_by_shader_state
            .entry(shader_state.clone())
            .or_insert_with(|| Batch::for_shader_state(&shader_state));

        // Track unique textures used.
        for texture in shader_state
            .texture_a
            .iter()
            .chain(shader_state.texture_b.iter())
        {
            match texture.source {
                TmemSource::Undefined => (),
                _ => {
                    self.unique_textures.insert(texture.clone());
                }
            }
        }

        for slot in index.iter().copied() {
            if let Some(vertex) = state.vertex_slots[slot as usize].as_ref() {
                batch.vertex_data.extend_from_slice(&vertex[..]);
            } else {
                panic!("display list referenced uninitialized vertex slot {}", slot);
            }
        }
    }

    pub fn total_dlists(&self) -> usize {
        self.total_dlists
    }

    pub fn total_instructions(&self) -> usize {
        self.total_instructions
    }

    pub fn unmapped_calls(&self) -> impl Debug + '_ {
        &self.unmapped_calls
    }

    pub fn unmapped_textures(&self) -> impl Debug + '_ {
        &self.unmapped_textures
    }

    pub fn max_depth(&self) -> usize {
        self.max_depth
    }

    pub fn total_lit_verts(&self) -> usize {
        self.total_lit_verts
    }

    pub fn total_unlit_verts(&self) -> usize {
        self.total_unlit_verts
    }

    pub fn unique_textures(&self) -> usize {
        self.unique_textures.len()
    }

    pub fn iter_textures(&self) -> std::collections::hash_set::Iter<'_, TextureDescriptor> {
        self.unique_textures.iter()
    }

    pub fn for_each_batch<F>(&mut self, mut f: F)
    where
        F: FnMut(&Batch),
    {
        for batch in self.batches_by_shader_state.values() {
            f(batch)
        }
    }
}
