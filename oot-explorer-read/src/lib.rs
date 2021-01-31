mod error;
mod from_vrom;
mod layout;
mod sentinel;
mod slice;
mod vrom_proxy;

pub use error::ReadError;
pub use from_vrom::FromVrom;
pub use layout::{aligned_data, check_alignment, Layout};
pub use sentinel::{is_end, Sentinel, SentinelIter};
pub use slice::{Slice, SliceIter};
pub use vrom_proxy::VromProxy;
