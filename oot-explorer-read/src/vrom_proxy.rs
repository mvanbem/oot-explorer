use oot_explorer_vrom::VromAddr;

use crate::FromVrom;

/// Proxy types that wrap a VROM address.
pub trait VromProxy: FromVrom {
    fn addr(&self) -> VromAddr;
}
