pub mod batch;
pub mod display_list_interpreter;
mod glsl_float_constant;
mod glsl_vec3_constant;
mod lit_vertex;
mod rcp;
mod shader_state;
mod su8;
mod to_expr;
mod unlit_vertex;

const FLAGS_UNLIT: u8 = 0;
const FLAGS_LIT: u8 = 1;
