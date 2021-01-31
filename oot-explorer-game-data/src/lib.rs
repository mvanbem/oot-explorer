#![recursion_limit = "256"]
#![cfg_attr(feature = "trace_macros", feature(trace_macros))]

#[cfg(feature = "trace_macros")]
trace_macros!(true);

#[macro_use]
mod macros;

pub mod collision;
pub mod gbi;
pub mod header_common;
pub mod header_room;
pub mod header_scene;
pub mod mesh;
pub mod object;
pub mod room;
pub mod scene;
pub mod versions;
