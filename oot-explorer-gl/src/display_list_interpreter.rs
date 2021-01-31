use byteorder::{NativeEndian, WriteBytesExt};
use oot_explorer_game_data::gbi::{
    DisplayList, GeometryMode, Instruction, LitVertex, MtxFlags, OtherModeH, OtherModeL, Qu0_16,
    Qu10_2, UnlitVertex,
};
use oot_explorer_read::{FromVrom, ReadError, Slice};
use oot_explorer_segment::{SegmentAddr, SegmentTable};
use oot_explorer_vrom::Vrom;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Debug;
use std::io::Write;

use crate::batch::Batch;
use crate::rcp::{
    CombinerState, Matrix, Point, RcpState, RdpOtherMode, RspTextureState, TextureSource,
    TileAttributes, TileDimensions, Tmem, TmemRegion, TmemSource,
};
use crate::shader_state::{ShaderState, TextureDescriptor};
use crate::{FLAGS_LIT, FLAGS_UNLIT};

#[derive(Clone)]
pub struct DisplayListInterpreter {
    total_dlists: usize,
    total_instructions: usize,
    unmapped_calls: BTreeMap<SegmentAddr, usize>,
    unmapped_matrices: BTreeMap<SegmentAddr, usize>,
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
            unmapped_matrices: BTreeMap::new(),
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

    pub fn interpret(
        &mut self,
        vrom: Vrom<'_>,
        segment_table: &SegmentTable,
        opacity: DisplayListOpacity,
        dlist: DisplayList,
    ) -> Result<(), ReadError> {
        self.interpret_internal(
            vrom,
            segment_table,
            dlist,
            opacity,
            &mut RcpState {
                matrix_stack: vec![Matrix::identity()],
                vertex_slots: [None; 32],
                geometry_mode: GeometryMode::default(),
                rdp_half_1: None,
                rdp_other_mode: RdpOtherMode {
                    lo: OtherModeL(0),
                    hi: OtherModeH::CYC_2CYCLE | OtherModeH::TT_RGBA16,
                },
                primitive_color: None,
                env_color: None,
                prim_lod_frac: None,
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
        )
    }

    fn interpret_internal(
        &mut self,
        vrom: Vrom<'_>,
        segment_table: &SegmentTable,
        dlist: DisplayList,
        opacity: DisplayListOpacity,
        state: &mut RcpState,
        depth: usize,
    ) -> Result<(), ReadError> {
        self.total_dlists += 1;
        self.max_depth = self.max_depth.max(depth);

        for result in dlist.instructions(vrom) {
            let instruction = result?;
            self.total_instructions += 1;
            match instruction {
                // 0x00
                Instruction::Noop { .. } => {
                    panic!("Wasn't expecting a Noop instruction. May be interpreting zeros.");
                }
                // 0x01
                Instruction::Vtx {
                    count,
                    index,
                    segment_addr,
                } => {
                    // TODO: Dedupe vertices (at least by address, but maybe by value?).

                    let mut index = index as usize;
                    if state.geometry_mode.test(GeometryMode::LIGHTING) {
                        // Lit vertices
                        self.total_lit_verts += 1;
                        let vertices = Slice::<LitVertex>::new(
                            segment_table.resolve(segment_addr).unwrap(),
                            count as u32,
                        );
                        for result in vertices.iter(vrom) {
                            let vertex = result?;
                            state.vertex_slots[index] =
                                Some(DisplayListInterpreter::transform_and_encode_lit_vertex(
                                    vrom,
                                    vertex,
                                    state.matrix_stack.last().as_ref().unwrap(),
                                    state.rsp_texture_state.scale_s,
                                    state.rsp_texture_state.scale_t,
                                ));
                            index = (index + 1) & 0x1f;
                        }
                    } else {
                        // Unlit vertices
                        self.total_unlit_verts += 1;
                        let vertices = Slice::<UnlitVertex>::new(
                            segment_table.resolve(segment_addr)?,
                            count as u32,
                        );
                        for result in vertices.iter(vrom) {
                            let vertex = result?;
                            state.vertex_slots[index] =
                                Some(DisplayListInterpreter::transform_and_encode_unlit_vertex(
                                    vrom,
                                    vertex,
                                    state.matrix_stack.last().as_ref().unwrap(),
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
                    let segment_addr = SegmentAddr(state.rdp_half_1.unwrap());
                    match segment_table.resolve(segment_addr) {
                        Ok(vrom_addr) => self.interpret_internal(
                            vrom,
                            segment_table,
                            DisplayList::from_vrom(vrom, vrom_addr)?,
                            opacity,
                            // NOTE: Clone the RCP state because normally only one path would be
                            // taken. Since we take both paths, each must be unaffected by the
                            // other.
                            &mut state.clone(),
                            depth + 1,
                        )?,
                        Err(_) => *self.unmapped_calls.entry(segment_addr).or_default() += 1,
                    }
                }
                // 0x05
                Instruction::Tri1 { index } => {
                    self.process_triangle(opacity, state, index);
                }
                // 0x06
                Instruction::Tri2 { index_a, index_b } => {
                    self.process_triangle(opacity, state, index_a);
                    self.process_triangle(opacity, state, index_b);
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
                Instruction::Mtx {
                    flags,
                    segment_addr,
                } => {
                    // The projection matrix is not modeled here.
                    if flags & MtxFlags::PROJECTION == MtxFlags::MODELVIEW {
                        let new_matrix = match segment_table.resolve(segment_addr) {
                            Ok(vrom_addr) => Matrix::from_rsp_format(
                                vrom.slice(vrom_addr..vrom_addr + Matrix::SIZE)?,
                            )
                            .unwrap(),
                            Err(_) => {
                                // NOTE: This doesn't seem right at all, but I can't get Jabu's main
                                // room to look right unless matrix loads from unmapped regions are
                                // totally skipped. Working with an identity matrix messes it up badly.
                                // This might just be an artifact of not modeling the scene render
                                // function.
                                continue;
                            }
                        };
                        let product = match flags & MtxFlags::LOAD {
                            MtxFlags::MUL => state.matrix_stack.last().unwrap() * &new_matrix,
                            MtxFlags::LOAD => new_matrix,
                            _ => unreachable!(),
                        };
                        match (flags & MtxFlags::PUSH, state.matrix_stack.len()) {
                            (MtxFlags::NOPUSH, _) | (MtxFlags::PUSH, 10) => {
                                *state.matrix_stack.last_mut().unwrap() = product
                            }
                            (MtxFlags::PUSH, _) => state.matrix_stack.push(product),
                            _ => unreachable!(),
                        }
                    }
                }
                // 0xde
                Instruction::Dl {
                    jump: _,
                    segment_addr,
                } => {
                    // NOTE: The jump field is handled by DisplayList::parse().
                    match segment_table.resolve(segment_addr) {
                        Ok(vrom_addr) => self.interpret_internal(
                            vrom,
                            segment_table,
                            DisplayList::from_vrom(vrom, vrom_addr)?,
                            opacity,
                            // NOTE: It really seems like calling another display list and returning
                            // to continue should not fork the RCP state. But it seems like I have
                            // to do this to get Jabu's main room to look right, maybe because of
                            // other bugs.
                            &mut state.clone(),
                            depth + 1,
                        )?,
                        Err(_) => *self.unmapped_calls.entry(segment_addr).or_default() += 1,
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
                Instruction::SetOtherModeL {
                    clear_bits,
                    set_bits,
                } => {
                    state.rdp_other_mode.lo &= !clear_bits;
                    state.rdp_other_mode.lo |= set_bits;
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

                    match state.texture_src.as_ref() {
                        Some(texture_src) => {
                            let range = 256..256 + count;
                            let source = TmemSource::LoadTlut {
                                ptr: texture_src.ptr,
                                count,
                            };
                            state.tmem.overlay(TmemRegion { range, source });
                        }
                        None => {
                            // Source attributes are not sufficiently defined to determine the
                            // destination range. Invalidate the whole TMEM.
                            state.tmem = Tmem::default();
                        }
                    };
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
                    match (state.texture_src.as_ref(), tile_attributes) {
                        (Some(texture_src), Some(tile_attributes)) => {
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
                        }
                        _ => {
                            // Tile parameters and/or source attributes are not sufficiently defined
                            // to determine the destination range. Invalidate the whole TMEM.
                            state.tmem = Tmem::default();
                        }
                    };
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
                        clamp_s,
                        mirror_s,
                        mask_s,
                        shift_s,
                        clamp_t,
                        mirror_t,
                        mask_t,
                        shift_t,
                    });
                }
                // 0xfa
                Instruction::SetPrimColor {
                    min_lod: _,
                    lod_fraction,
                    r,
                    g,
                    b,
                    a,
                } => {
                    state.prim_lod_frac = Some(lod_fraction);
                    state.primitive_color = Some([r, g, b, a]);
                }
                // 0xfb
                Instruction::SetEnvColor { r, g, b, a } => {
                    state.env_color = Some([r, g, b, a]);
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
                    segment_addr,
                } => {
                    // NOTE: That width value does appear to exist, but is not used by LoadBlock
                    // instructions and I'm not sure Ocarina of Time ends up using it at all.
                    match segment_table.resolve(segment_addr) {
                        Ok(vrom_addr) => {
                            state.texture_src = Some(TextureSource {
                                format,
                                depth,
                                ptr: vrom_addr,
                            })
                        }
                        Err(_) => {
                            state.texture_src = None;
                            *self.unmapped_textures.entry(segment_addr).or_default() += 1;
                        }
                    };
                }
            }
        }

        Ok(())
    }

    fn transform_texcoord(texcoord: i16, scale: Qu0_16) -> i16 {
        (((texcoord as i32) * (scale.0 as i32)) >> 16) as i16
    }

    fn transform_and_encode_unlit_vertex(
        vrom: Vrom<'_>,
        vertex: UnlitVertex,
        matrix: &Matrix,
        scale_s: Qu0_16,
        scale_t: Qu0_16,
    ) -> [u8; 20] {
        let mut buf = [0; 20];

        let mut w = &mut buf[..];
        // [0..=5] Position
        let pos = vertex.position(vrom);
        let pos = matrix * Point([pos[0], pos[1], pos[2], 1]);
        w.write_i16::<NativeEndian>(pos.0[0]).unwrap();
        w.write_i16::<NativeEndian>(pos.0[1]).unwrap();
        w.write_i16::<NativeEndian>(pos.0[2]).unwrap();
        // [6..=7] Padding
        w.write_u16::<NativeEndian>(0).unwrap();
        // [8..=10] Normal (unused for unlit geometry)
        w.write_i8(0).unwrap();
        w.write_i8(0).unwrap();
        w.write_i8(0).unwrap();
        // [11] Flags
        w.write_u8(FLAGS_UNLIT).unwrap();
        // [12..=15] Texture coordinates
        let texcoord = vertex.texcoord(vrom);
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
        let color = vertex.color(vrom);
        w.write_all(&color[..]).unwrap();
        assert_eq!(w.len(), 0);

        buf
    }

    fn transform_and_encode_lit_vertex(
        vrom: Vrom<'_>,
        vertex: LitVertex,
        matrix: &Matrix,
        scale_s: Qu0_16,
        scale_t: Qu0_16,
    ) -> [u8; 20] {
        let mut buf = [0; 20];

        let mut w = &mut buf[..];
        // [0..=5] Position
        let pos = vertex.position(vrom);
        let pos = matrix * Point([pos[0], pos[1], pos[2], 1]);
        w.write_i16::<NativeEndian>(pos.0[0]).unwrap();
        w.write_i16::<NativeEndian>(pos.0[1]).unwrap();
        w.write_i16::<NativeEndian>(pos.0[2]).unwrap();
        // [6..=7] Padding
        w.write_u16::<NativeEndian>(0).unwrap();
        // [8..=10] Normal
        let normal = vertex.normal(vrom);
        w.write_i8(normal[0]).unwrap();
        w.write_i8(normal[1]).unwrap();
        w.write_i8(normal[2]).unwrap();
        // [11] Flags
        w.write_u8(FLAGS_LIT).unwrap();
        // [12..=15] Texture coordinates
        let texcoord = vertex.texcoord(vrom);
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
        w.write_u8(vertex.alpha(vrom)).unwrap();
        assert_eq!(w.len(), 0);

        buf
    }

    fn process_triangle(
        &mut self,
        opacity: DisplayListOpacity,
        state: &mut RcpState,
        index: [u8; 3],
    ) {
        // Print each unique shader state as it is encountered.
        let shader_state = state.to_shader_state();
        let batch = self
            .batches_by_shader_state
            .entry(shader_state.clone())
            .or_insert_with(|| Batch::for_shader_state(&shader_state, opacity));

        // Track unique textures used.
        for texture in shader_state
            .texture_0
            .iter()
            .chain(shader_state.texture_1.iter())
        {
            match texture.descriptor.source {
                TmemSource::Undefined => (),
                _ => {
                    self.unique_textures.insert(texture.descriptor.clone());
                }
            }
        }

        for slot in index.iter().copied() {
            match state.vertex_slots[slot as usize].as_ref() {
                Some(vertex) => batch.vertex_data.extend_from_slice(&vertex[..]),
                None => panic!("display list referenced uninitialized vertex slot {}", slot),
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

    pub fn unmapped_matrices(&self) -> impl Debug + '_ {
        &self.unmapped_matrices
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

    pub fn iter_textures(&self) -> impl Iterator<Item = &TextureDescriptor> + '_ {
        self.unique_textures.iter()
    }

    pub fn iter_batches(&self) -> impl Iterator<Item = &Batch> + '_ {
        self.batches_by_shader_state.values()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DisplayListOpacity {
    Opaque,
    Translucent,
}
