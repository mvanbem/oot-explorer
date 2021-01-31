use oot_explorer_read::ReadError;
use oot_explorer_vrom::{Vrom, VromAddr};

use crate::PrimitiveType;

pub struct EnumDescriptor {
    pub name: &'static str,
    pub underlying: PrimitiveType,
    pub values: &'static [(u32, &'static str)],
}

impl EnumDescriptor {
    pub fn read_as_u32(&self, vrom: Vrom<'_>, addr: VromAddr) -> Result<u32, ReadError> {
        self.underlying.read_as_u32(vrom, addr)
    }
}
