use crate::expr;
use byteorder::{NativeEndian, WriteBytesExt};
use num_traits::identities::{One, Zero};
use oot_explorer_core::fs::VromAddr;
use oot_explorer_core::gbi::{
    self, AlphaCombine, AlphaInput, ColorCombine, ColorInput, DisplayList, GeometryMode,
    Instruction, OtherModeH, OtherModeHMask, Qu10_2, Qu1_11, TextureDepth, TextureFormat,
};
use oot_explorer_core::segment::{SegmentAddr, SegmentCtx};
use oot_explorer_core::slice::Slice;
use std::collections::{BTreeMap, HashMap};
use std::io::Write;

#[derive(Debug)]
struct RcpState {
    vertex_slots: [Option<[u8; 20]>; 32],
    geometry_mode: GeometryMode,
    rdp_half_1: Option<u32>,
    rdp_other_mode: RdpOtherMode,
    combiner: Option<CombinerState>,
    texture_src: Option<TextureSource>,
    tiles: [Option<Tile>; 8],
    tmem: Tmem,
}
impl RcpState {
    fn shader_state(&self) -> ShaderState {
        ShaderState {
            two_cycle_mode: match self.rdp_other_mode.hi & OtherModeHMask::CYCLETYPE {
                x if x == OtherModeH::CYC_1CYCLE => false,
                x if x == OtherModeH::CYC_2CYCLE => true,
                _ => panic!(
                    "display list did not choose one- or two-cycle mode: {:#?}",
                    self
                ),
            },
            combiner: self.combiner.as_ref().unwrap().clone(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct RdpOtherMode {
    hi: OtherModeH,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct CombinerState {
    color_0: ColorCombine,
    alpha_0: AlphaCombine,
    color_1: ColorCombine,
    alpha_1: AlphaCombine,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct TextureSource {
    format: TextureFormat,
    depth: TextureDepth,
    ptr: VromAddr,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct Tile {
    width: usize,
    height: usize,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum Tmem {
    Undefined,
    LoadBlock {
        dxt: Qu1_11,
        ptr: VromAddr,
        len: u32,
    },
}

#[derive(Clone, Eq, Hash, PartialEq)]
struct ShaderState {
    two_cycle_mode: bool,
    combiner: CombinerState,
}
impl ShaderState {
    fn color_0_expr(&self, ctx: &mut expr::Context<GlslVec3Constant>) -> expr::Key {
        self.combiner.color_0.to_expr(ctx, Cycle::Cycle1)
    }
    fn alpha_0_expr(&self, ctx: &mut expr::Context<GlslFloatConstant>) -> expr::Key {
        self.combiner.alpha_0.to_expr(ctx, Cycle::Cycle1)
    }
    fn color_1_expr(&self, ctx: &mut expr::Context<GlslVec3Constant>) -> expr::Key {
        self.combiner.color_1.to_expr(ctx, Cycle::Cycle2)
    }
    fn alpha_1_expr(&self, ctx: &mut expr::Context<GlslFloatConstant>) -> expr::Key {
        self.combiner.alpha_1.to_expr(ctx, Cycle::Cycle2)
    }
    fn to_glsl(&self) -> String {
        let mut color_ctx = expr::Context::new();
        let mut alpha_ctx = expr::Context::new();
        let color_0 = self.color_0_expr(&mut color_ctx);
        let alpha_0 = self.alpha_0_expr(&mut alpha_ctx);
        let color_1 = self.color_1_expr(&mut color_ctx);
        let alpha_1 = self.alpha_1_expr(&mut alpha_ctx);
        assert!(self.two_cycle_mode);

        format!(
            r#"#version 300 es

precision highp float;
precision highp int;

uniform vec4 u_env;
/*
uniform vec3 u_center;
uniform vec3 u_scale;
uniform float u_k4;
uniform float u_k5;
*/

in vec4 v_color;
in vec4 v_shade;

layout(location = 0) out vec4 fragColor;

void main() {{
  // TODO: implement texturing
  vec4 texel0 = vec4(1.0, 0.0, 1.0, 0.5);
  vec4 texel1 = vec4(1.0, 0.0, 1.0, 0.5);
  // TODO: implement noise?
  // TODO: implement LOD
  float lod_fraction = 0.5;
  float prim_lod_frac = 0.5;

  vec4 cycle1 = vec4({:?}, {:?});
  fragColor = vec4({:?}, {:?});
}}
"#,
            color_ctx.get_with_ctx(color_0).unwrap(),
            alpha_ctx.get_with_ctx(alpha_0).unwrap(),
            color_ctx.get_with_ctx(color_1).unwrap(),
            alpha_ctx.get_with_ctx(alpha_1).unwrap(),
        )
    }
}

#[derive(Clone)]
pub struct Batch {
    fragment_shader: String,
    vertex_data: Vec<u8>,
}
impl Batch {
    fn for_shader_state(shader_state: &ShaderState) -> Batch {
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
                tiles: [None; 8],
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
                Instruction::Noop { tag } => {
                    if tag != SegmentAddr(0) {
                        // TODO: Safely attempt to retrieve a string at the tag address.
                        panic!("nop with actual tag: {:?}", tag);
                    }
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
                    // experiment
                    return;
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
                Instruction::Texture { .. } => {
                    eprintln!("WARNING: Texture instruction is unimplemented")
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
                Instruction::EndDl => unimplemented!("EndDl instruction"),
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
                    state.tiles[tile as usize] = Some(Tile {
                        width: end_s.as_f32() as usize + 1,
                        height: end_t.as_f32() as usize + 1,
                    });
                }
                // 0xf3
                Instruction::LoadBlock {
                    start_s,
                    start_t,
                    tile: _,
                    end_s,
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
                                TextureDepth::Bits4 => (end_s as u32) / 2,
                                TextureDepth::Bits8 => end_s as u32,
                                TextureDepth::Bits16 => (end_s as u32) * 2,
                                TextureDepth::Bits32 => (end_s as u32) * 4,
                            },
                        };
                    } else {
                        state.tmem = Tmem::Undefined;
                    }
                    println!("{:?}", state.tmem);
                }
                // 0xf5
                Instruction::SetTile { .. } => {
                    eprintln!("WARNING: SetTile instruction is unimplemented")
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
        write_unlit_vertex(&mut buf, vertex);
        buf
    }
    fn encode_lit_vertex<T>(&mut self, vertex: &T) -> [u8; 20]
    where
        T: LitVertex,
    {
        let mut buf = [0; 20];
        write_lit_vertex(&mut buf, vertex);
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

const FLAGS_UNLIT: u8 = 0;
const FLAGS_LIT: u8 = 1;

fn write_unlit_vertex<T>(dst: &mut [u8; 20], vertex: &T)
where
    T: UnlitVertex,
{
    let mut w = &mut dst[..];
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
    w.write_i16::<NativeEndian>(texcoord[0]).unwrap();
    w.write_i16::<NativeEndian>(texcoord[1]).unwrap();
    // [16..=19] Color
    let color = vertex.color();
    w.write_all(&color[..]).unwrap();
    assert_eq!(w.len(), 0);
}

fn write_lit_vertex<T>(dst: &mut [u8; 20], vertex: &T)
where
    T: LitVertex,
{
    let mut w = &mut dst[..];
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
    w.write_i16::<NativeEndian>(texcoord[0]).unwrap();
    w.write_i16::<NativeEndian>(texcoord[1]).unwrap();
    // [16..=19] Color (RGB are unused for lit geometry)
    w.write_u8(0).unwrap();
    w.write_u8(0).unwrap();
    w.write_u8(0).unwrap();
    w.write_u8(vertex.alpha()).unwrap();
    assert_eq!(w.len(), 0);
}

trait UnlitVertex {
    fn position(&self) -> [i16; 3];
    fn texcoord(&self) -> [i16; 2];
    fn color(&self) -> [u8; 4];
}
impl<'a> UnlitVertex for gbi::UnlitVertex<'a> {
    fn position(&self) -> [i16; 3] {
        gbi::UnlitVertex::position(*self)
    }
    fn texcoord(&self) -> [i16; 2] {
        gbi::UnlitVertex::texcoord(*self)
    }
    fn color(&self) -> [u8; 4] {
        gbi::UnlitVertex::color(*self)
    }
}

trait LitVertex {
    fn position(&self) -> [i16; 3];
    fn texcoord(&self) -> [i16; 2];
    fn normal(&self) -> [i8; 3];
    fn alpha(&self) -> u8;
}
impl<'a> LitVertex for gbi::LitVertex<'a> {
    fn position(&self) -> [i16; 3] {
        gbi::LitVertex::position(*self)
    }
    fn texcoord(&self) -> [i16; 2] {
        gbi::LitVertex::texcoord(*self)
    }
    fn normal(&self) -> [i8; 3] {
        gbi::LitVertex::normal(*self)
    }
    fn alpha(&self) -> u8 {
        gbi::LitVertex::alpha(*self)
    }
}

/// A scaled unsigned 8-bit number, where 0 -> 0.0 and 255 -> 1.0.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Su8(pub u8);
impl std::fmt::Display for Su8 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.as_f32())
    }
}
impl std::fmt::Debug for Su8 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Su8({})", self.as_f32())
    }
}
impl Su8 {
    pub fn as_f32(self) -> f32 {
        self.0 as f32 / 255.0
    }
}
impl Zero for Su8 {
    fn zero() -> Su8 {
        Su8(0)
    }
    fn is_zero(&self) -> bool {
        self.0 == 0
    }
}
impl One for Su8 {
    fn one() -> Su8 {
        Su8(255)
    }
    fn is_one(&self) -> bool {
        self.0 == 255
    }
}
impl std::ops::Add<Su8> for Su8 {
    type Output = Su8;
    fn add(self, rhs: Su8) -> Su8 {
        Su8(self.0.checked_add(rhs.0).unwrap())
    }
}
impl std::ops::AddAssign<Su8> for Su8 {
    fn add_assign(&mut self, rhs: Su8) {
        self.0 = self.0.checked_add(rhs.0).unwrap();
    }
}
impl std::ops::Mul<Su8> for Su8 {
    type Output = Su8;
    fn mul(self, rhs: Su8) -> Su8 {
        let long = (self.0 as u16).checked_mul(rhs.0 as u16).unwrap();
        let units = long / 255;
        let remainder = long - 255 * units;
        Su8((units + remainder / 128) as u8)
    }
}
impl std::ops::MulAssign<Su8> for Su8 {
    fn mul_assign(&mut self, rhs: Su8) {
        *self = *self * rhs;
    }
}
impl std::ops::Neg for Su8 {
    type Output = Su8;
    fn neg(self) -> Su8 {
        match self.0 {
            0 => Su8(0),
            _ => panic!("result would be out of range: !{:?}", self),
        }
    }
}
#[cfg(test)]
mod su8_tests {
    use super::Su8;

    #[test]
    fn add() {
        assert_eq!(Su8(3) + Su8(5), Su8(8));
    }

    #[test]
    #[should_panic]
    fn add_panics_on_overflow() {
        let _ = Su8(128) + Su8(128);
    }

    #[test]
    fn mul() {
        assert_eq!(Su8(0) * Su8(0), Su8(0));
        assert_eq!(Su8(0) * Su8(255), Su8(0));
        assert_eq!(Su8(255) * Su8(0), Su8(0));
        assert_eq!(Su8(255) * Su8(255), Su8(255));
        assert_eq!(Su8(85) * Su8(170), Su8(57));
    }
}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
enum GlslVec3Constant {
    Zero,
    One,
}
impl std::fmt::Display for GlslVec3Constant {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            GlslVec3Constant::Zero => write!(f, "vec3(0.0, 0.0, 0.0)"),
            GlslVec3Constant::One => write!(f, "vec3(1.0, 1.0, 1.0)"),
        }
    }
}
impl Zero for GlslVec3Constant {
    fn zero() -> GlslVec3Constant {
        GlslVec3Constant::Zero
    }
    fn is_zero(&self) -> bool {
        *self == GlslVec3Constant::Zero
    }
}
impl One for GlslVec3Constant {
    fn one() -> GlslVec3Constant {
        GlslVec3Constant::One
    }
    fn is_one(&self) -> bool {
        *self == GlslVec3Constant::One
    }
}
impl std::ops::Add for GlslVec3Constant {
    type Output = GlslVec3Constant;
    fn add(self, rhs: GlslVec3Constant) -> GlslVec3Constant {
        use GlslVec3Constant::{One, Zero};
        match (self, rhs) {
            (Zero, Zero) => Zero,
            (Zero, One) | (One, Zero) => One,
            (One, One) => panic!("overflow"),
        }
    }
}
impl std::ops::AddAssign for GlslVec3Constant {
    fn add_assign(&mut self, rhs: GlslVec3Constant) {
        *self = *self + rhs;
    }
}
impl std::ops::Mul for GlslVec3Constant {
    type Output = GlslVec3Constant;
    fn mul(self, rhs: GlslVec3Constant) -> GlslVec3Constant {
        use GlslVec3Constant::{One, Zero};
        match (self, rhs) {
            (One, One) => One,
            _ => Zero,
        }
    }
}
impl std::ops::MulAssign for GlslVec3Constant {
    fn mul_assign(&mut self, rhs: GlslVec3Constant) {
        *self = *self * rhs;
    }
}
impl std::ops::Neg for GlslVec3Constant {
    type Output = GlslVec3Constant;
    fn neg(self) -> GlslVec3Constant {
        use GlslVec3Constant::{One, Zero};
        match self {
            Zero => Zero,
            One => panic!("overflow"),
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
enum GlslFloatConstant {
    Zero,
    One,
}
impl std::fmt::Display for GlslFloatConstant {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            GlslFloatConstant::Zero => write!(f, "0.0"),
            GlslFloatConstant::One => write!(f, "1.0"),
        }
    }
}
impl Zero for GlslFloatConstant {
    fn zero() -> GlslFloatConstant {
        GlslFloatConstant::Zero
    }
    fn is_zero(&self) -> bool {
        *self == GlslFloatConstant::Zero
    }
}
impl One for GlslFloatConstant {
    fn one() -> GlslFloatConstant {
        GlslFloatConstant::One
    }
    fn is_one(&self) -> bool {
        *self == GlslFloatConstant::One
    }
}
impl std::ops::Add for GlslFloatConstant {
    type Output = GlslFloatConstant;
    fn add(self, rhs: GlslFloatConstant) -> GlslFloatConstant {
        use GlslFloatConstant::{One, Zero};
        match (self, rhs) {
            (Zero, Zero) => Zero,
            (Zero, One) | (One, Zero) => One,
            (One, One) => panic!("overflow"),
        }
    }
}
impl std::ops::AddAssign for GlslFloatConstant {
    fn add_assign(&mut self, rhs: GlslFloatConstant) {
        *self = *self + rhs;
    }
}
impl std::ops::Mul for GlslFloatConstant {
    type Output = GlslFloatConstant;
    fn mul(self, rhs: GlslFloatConstant) -> GlslFloatConstant {
        use GlslFloatConstant::{One, Zero};
        match (self, rhs) {
            (One, One) => One,
            _ => Zero,
        }
    }
}
impl std::ops::MulAssign for GlslFloatConstant {
    fn mul_assign(&mut self, rhs: GlslFloatConstant) {
        *self = *self * rhs;
    }
}
impl std::ops::Neg for GlslFloatConstant {
    type Output = GlslFloatConstant;
    fn neg(self) -> GlslFloatConstant {
        use GlslFloatConstant::{One, Zero};
        match self {
            Zero => Zero,
            One => panic!("overflow"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Cycle {
    Cycle1,
    Cycle2,
}

trait ToExpr<T: expr::ValueType> {
    fn to_expr(&self, ctx: &mut expr::Context<T>, cycle: Cycle) -> expr::Key;
}
impl ToExpr<GlslVec3Constant> for ColorCombine {
    fn to_expr(&self, ctx: &mut expr::Context<GlslVec3Constant>, cycle: Cycle) -> expr::Key {
        let a = self.a().to_expr(ctx, cycle);
        let b = self.b().to_expr(ctx, cycle);
        let neg_b = ctx.neg(b);
        let sum = ctx.add(vec![a, neg_b]);
        let c = self.c().to_expr(ctx, cycle);
        let product = ctx.mul(vec![sum, c]);
        let d = self.d().to_expr(ctx, cycle);
        ctx.add(vec![product, d])
    }
}
impl ToExpr<GlslFloatConstant> for AlphaCombine {
    fn to_expr(&self, ctx: &mut expr::Context<GlslFloatConstant>, cycle: Cycle) -> expr::Key {
        let a = self.a().to_expr(ctx, cycle);
        let b = self.b().to_expr(ctx, cycle);
        let neg_b = ctx.neg(b);
        let sum = ctx.add(vec![a, neg_b]);
        let c = self.c().to_expr(ctx, cycle);
        let product = ctx.mul(vec![sum, c]);
        let d = self.d().to_expr(ctx, cycle);
        ctx.add(vec![product, d])
    }
}
impl ToExpr<GlslVec3Constant> for ColorInput {
    fn to_expr(&self, ctx: &mut expr::Context<GlslVec3Constant>, cycle: Cycle) -> expr::Key {
        match self {
            ColorInput::Combined => match cycle {
                Cycle::Cycle1 => panic!("combined input is invalid on cycle 1"),
                Cycle::Cycle2 => ctx.symbol("cycle1.rgb".to_string()),
            },
            ColorInput::Texel0 => ctx.symbol("texel0.rgb".to_string()),
            ColorInput::Texel1 => ctx.symbol("texel1.rgb".to_string()),
            ColorInput::Primitive => ctx.symbol("v_color.rgb".to_string()),
            ColorInput::Shade => ctx.symbol("v_shade.rgb".to_string()),
            ColorInput::Environment => ctx.symbol("u_env.rgb".to_string()),
            ColorInput::One => ctx.literal(One::one()),
            ColorInput::Noise => ctx.symbol("noise.rgb".to_string()),
            ColorInput::Zero => ctx.literal(Zero::zero()),
            ColorInput::Center => ctx.symbol("u_center".to_string()),
            ColorInput::K4 => ctx.symbol("vec3(u_k4)".to_string()),
            ColorInput::Scale => ctx.symbol("u_scale".to_string()),
            ColorInput::CombinedAlpha => match cycle {
                Cycle::Cycle1 => panic!("combined input is invalid on cycle 1"),
                Cycle::Cycle2 => ctx.symbol("cycle1.aaa".to_string()),
            },
            ColorInput::Texel0Alpha => ctx.symbol("texel0.aaa".to_string()),
            ColorInput::Texel1Alpha => ctx.symbol("texel1.aaa".to_string()),
            ColorInput::PrimitiveAlpha => ctx.symbol("v_color.aaa".to_string()),
            ColorInput::ShadeAlpha => ctx.symbol("shade.aaa".to_string()),
            ColorInput::EnvAlpha => ctx.symbol("u_env.aaa".to_string()),
            ColorInput::LodFraction => ctx.symbol("vec3(lod_fraction)".to_string()),
            ColorInput::PrimLodFrac => ctx.symbol("vec3(prim_lod_frac)".to_string()),
            ColorInput::K5 => ctx.symbol("vec3(u_k5)".to_string()),
        }
    }
}
impl ToExpr<GlslFloatConstant> for AlphaInput {
    fn to_expr(&self, ctx: &mut expr::Context<GlslFloatConstant>, cycle: Cycle) -> expr::Key {
        match self {
            AlphaInput::Combined => match cycle {
                Cycle::Cycle1 => panic!("combined input is invalid on cycle 1"),
                Cycle::Cycle2 => ctx.symbol("cycle1.a".to_string()),
            },
            AlphaInput::Texel0 => ctx.symbol("texel0.a".to_string()),
            AlphaInput::Texel1 => ctx.symbol("texel1.a".to_string()),
            AlphaInput::Primitive => ctx.symbol("v_color.a".to_string()),
            AlphaInput::Shade => ctx.symbol("v_shade.a".to_string()),
            AlphaInput::Environment => ctx.symbol("u_env.a".to_string()),
            AlphaInput::One => ctx.literal(One::one()),
            AlphaInput::Zero => ctx.literal(Zero::zero()),
            AlphaInput::LodFraction => ctx.symbol("lod_fraction".to_string()),
            AlphaInput::PrimLodFrac => ctx.symbol("prim_lod_frac".to_string()),
        }
    }
}
#[cfg(test)]
mod to_expr_tests {
    use super::*;

    #[test]
    fn test() {
        let mut ctx = expr::Context::new();
        let expr = ColorCombine::new(
            ColorInput::Combined,
            ColorInput::Texel0,
            ColorInput::Texel1,
            ColorInput::Primitive,
        )
        .to_expr(&mut ctx, Cycle::Cycle2);
        assert_eq!(
            format!("{:?}", ctx.get_with_ctx(expr).unwrap()),
            "(cycle1.rgb - texel0.rgb) * texel1.rgb + v_color.rgb"
        );
    }

    #[test]
    fn a_zero() {
        let mut ctx = expr::Context::new();
        let expr = ColorCombine::new(
            ColorInput::Zero,
            ColorInput::Texel0,
            ColorInput::Texel1,
            ColorInput::Primitive,
        )
        .to_expr(&mut ctx, Cycle::Cycle2);
        assert_eq!(
            format!("{:?}", ctx.get_with_ctx(expr).unwrap()),
            "-texel0.rgb * texel1.rgb + v_color.rgb"
        );
    }

    #[test]
    fn b_zero() {
        let mut ctx = expr::Context::new();
        let expr = ColorCombine::new(
            ColorInput::Combined,
            ColorInput::Zero,
            ColorInput::Texel1,
            ColorInput::Primitive,
        )
        .to_expr(&mut ctx, Cycle::Cycle2);
        assert_eq!(
            format!("{:?}", ctx.get_with_ctx(expr).unwrap()),
            "cycle1.rgb * texel1.rgb + v_color.rgb"
        );
    }

    #[test]
    fn c_zero() {
        let mut ctx = expr::Context::new();
        let expr = ColorCombine::new(
            ColorInput::Combined,
            ColorInput::Texel0,
            ColorInput::Zero,
            ColorInput::Primitive,
        )
        .to_expr(&mut ctx, Cycle::Cycle2);
        assert_eq!(
            format!("{:?}", ctx.get_with_ctx(expr).unwrap()),
            "v_color.rgb"
        );
    }

    #[test]
    fn c_one() {
        let mut ctx = expr::Context::new();
        let expr = ColorCombine::new(
            ColorInput::Combined,
            ColorInput::Texel0,
            ColorInput::One,
            ColorInput::Primitive,
        )
        .to_expr(&mut ctx, Cycle::Cycle2);
        assert_eq!(
            format!("{:?}", ctx.get_with_ctx(expr).unwrap()),
            "cycle1.rgb - texel0.rgb + v_color.rgb"
        );
    }

    #[test]
    fn d_zero() {
        let mut ctx = expr::Context::new();
        let expr = ColorCombine::new(
            ColorInput::Combined,
            ColorInput::Texel0,
            ColorInput::Texel1,
            ColorInput::Zero,
        )
        .to_expr(&mut ctx, Cycle::Cycle2);
        assert_eq!(
            format!("{:?}", ctx.get_with_ctx(expr).unwrap()),
            "(cycle1.rgb - texel0.rgb) * texel1.rgb"
        );
    }
}
