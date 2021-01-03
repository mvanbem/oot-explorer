pub mod batch;
pub mod display_list_interpreter;
mod glsl_float_constant;
mod glsl_vec3_constant;
pub mod rcp;
pub mod shader_state;
mod su8;
pub mod texture;
mod to_expr;

const FLAGS_UNLIT: u8 = 0;
const FLAGS_LIT: u8 = 1;
