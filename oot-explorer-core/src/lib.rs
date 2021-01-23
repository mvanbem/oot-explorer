#![recursion_limit = "256"]
#![cfg_attr(feature = "trace_macros", feature(trace_macros))]

#[cfg(feature = "trace_macros")]
trace_macros!(true);

#[macro_use]
mod macros;

pub mod collision;
pub mod delimited;
pub mod fs;
pub mod gbi;
pub mod header;
pub mod mesh;
pub mod object;
pub mod reflect;
pub mod rom;
pub mod room;
pub mod scene;
pub mod segment;
pub mod slice;
pub mod versions;
pub mod yaz;
