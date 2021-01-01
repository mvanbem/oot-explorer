use oot_explorer_core::gbi::{
    self, DisplayList, GeometryMode, Instruction, OtherModeH, Qu10_2, TextureDepth,
};
use oot_explorer_core::segment::{SegmentAddr, SegmentCtx};
use oot_explorer_core::slice::Slice;
use std::collections::{BTreeMap, HashMap};

use crate::batch::Batch;
use crate::lit_vertex::LitVertex;
use crate::rcp::{
    CombinerState, RcpState, RdpOtherMode, TextureSource, Tile, TileAttributes, TileDimensions,
    TileMipScale, Tmem,
};
use crate::shader_state::ShaderState;
use crate::unlit_vertex::UnlitVertex;

#[derive(Clone)]
pub struct DisplayListInterpreter {
    total_dlists: usize,
    total_instructions: usize,
    unmapped_calls: BTreeMap<SegmentAddr, usize>,
    unmapped_textures: BTreeMap<SegmentAddr, usize>,
    max_depth: usize,
    total_lit_verts: usize,
    total_unlit_verts: usize,

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
                    hi: OtherModeH::CYC_2CYCLE,
                },
                combiner: None,
                texture_src: None,
                tiles: [Tile::default(); 8],
                tmem: Tmem::Undefined,
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
                        let vertices = Slice::<'a, gbi::LitVertex<'a>>::new(
                            ctx.resolve(ptr).unwrap(),
                            count as usize,
                        );
                        for vertex in vertices {
                            state.vertex_slots[index] = Some(self.encode_lit_vertex(&vertex));
                            index = (index + 1) & 0x1f;
                        }
                    } else {
                        // Unlit vertices
                        self.total_unlit_verts += 1;
                        let vertices = Slice::<'a, gbi::UnlitVertex<'a>>::new(
                            ctx.resolve(ptr).unwrap(),
                            count as usize,
                        );
                        for vertex in vertices {
                            state.vertex_slots[index] = Some(self.encode_unlit_vertex(&vertex));
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
                    // Print each unique shader state as it is encountered.
                    let shader_state = state.shader_state();
                    let batch = self
                        .batches_by_shader_state
                        .entry(shader_state.clone())
                        .or_insert_with(|| Batch::for_shader_state(&shader_state));

                    for slot in index.iter().copied() {
                        if let Some(vertex) = state.vertex_slots[slot as usize].as_ref() {
                            batch.vertex_data.extend_from_slice(&vertex[..]);
                        } else {
                            panic!("display list referenced uninitialized vertex slot {}", slot);
                        }
                    }
                }
                // 0x06
                Instruction::Tri2 { index_a, index_b } => {
                    // Print each unique shader state as it is encountered.
                    let shader_state = state.shader_state();
                    let batch = self
                        .batches_by_shader_state
                        .entry(shader_state.clone())
                        .or_insert_with(|| Batch::for_shader_state(&shader_state));

                    for slot in index_a.iter().copied().chain(index_b.iter().copied()) {
                        if let Some(vertex) = state.vertex_slots[slot as usize].as_ref() {
                            batch.vertex_data.extend_from_slice(&vertex[..]);
                        } else {
                            panic!("display list referenced uninitialized vertex slot {}", slot);
                        }
                    }
                }
                // 0xd7
                Instruction::Texture {
                    level,
                    tile,
                    enable,
                    scale_s,
                    scale_t,
                } => {
                    state.tiles[(tile & 0x7) as usize].mip_scale = Some(TileMipScale {
                        level,
                        enable,
                        scale_s,
                        scale_t,
                    });
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
                Instruction::LoadTlut { .. } => {
                    eprintln!("WARNING: LoadTlut instruction is unimplemented")
                }
                // 0xf2
                Instruction::SetTileSize {
                    start_s,
                    start_t,
                    tile,
                    end_s,
                    end_t,
                } => {
                    // Haven't figured out how to handle these yet.
                    assert_eq!(start_s, Qu10_2(0));
                    assert_eq!(start_t, Qu10_2(0));
                    state.tiles[tile as usize].dimensions = Some(TileDimensions {
                        width: end_s.as_f32() as usize + 1,
                        height: end_t.as_f32() as usize + 1,
                    });
                }
                // 0xf3
                Instruction::LoadBlock {
                    start_s,
                    start_t,
                    tile: _,
                    texels,
                    dxt,
                } => {
                    // Haven't figured out how to handle these yet.
                    assert_eq!(start_s, Qu10_2(0));
                    assert_eq!(start_t, Qu10_2(0));

                    // TODO: Use the tile input somehow? Does it
                    // matter if the start coordinates are always
                    // zero?

                    if let Some(texture_src) = state.texture_src.as_ref() {
                        state.tmem = Tmem::LoadBlock {
                            dxt,
                            ptr: texture_src.ptr,
                            len: match texture_src.depth {
                                TextureDepth::Bits4 => (texels as u32) / 2,
                                TextureDepth::Bits8 => texels as u32,
                                TextureDepth::Bits16 => (texels as u32) * 2,
                                TextureDepth::Bits32 => (texels as u32) * 4,
                            },
                        };
                    } else {
                        eprintln!("WARNING: texture load from unmapped address");
                        state.tmem = Tmem::Undefined;
                    }
                    println!("{:?}", state.tmem);
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
                    if let Ok(vrom_range) = ctx.resolve_vrom(ptr) {
                        state.texture_src = Some(TextureSource {
                            format,
                            depth,
                            ptr: vrom_range.start,
                        });
                        println!("{:?}", state.texture_src.as_ref().unwrap());
                    } else {
                        state.texture_src = None;
                        *self.unmapped_textures.entry(ptr).or_default() += 1;
                    }
                }
            }
        });
    }
    fn encode_unlit_vertex<T>(&mut self, vertex: &T) -> [u8; 20]
    where
        T: UnlitVertex,
    {
        let mut buf = [0; 20];
        crate::unlit_vertex::write_unlit_vertex(&mut buf, vertex);
        buf
    }
    fn encode_lit_vertex<T>(&mut self, vertex: &T) -> [u8; 20]
    where
        T: LitVertex,
    {
        let mut buf = [0; 20];
        crate::lit_vertex::write_lit_vertex(&mut buf, vertex);
        buf
    }

    pub fn total_dlists(&self) -> usize {
        self.total_dlists
    }
    pub fn total_instructions(&self) -> usize {
        self.total_instructions
    }
    pub fn unmapped_calls(&self) -> impl std::fmt::Debug + '_ {
        &self.unmapped_calls
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
    pub fn for_each_batch<F>(&mut self, mut f: F)
    where
        F: FnMut(&Batch),
    {
        for batch in self.batches_by_shader_state.values() {
            f(batch)
        }
    }
}
