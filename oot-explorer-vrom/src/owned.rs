use crate::Vrom;

/// A boxed slice representing all of VROM.
///
/// Function parameters should generally prefer the borrowed [`Vrom`].
pub struct OwnedVrom {
    vrom: Box<[u8]>,
}

impl OwnedVrom {
    pub fn new(vrom: Box<[u8]>) -> Self {
        Self { vrom }
    }

    pub fn borrow(&self) -> Vrom<'_> {
        Vrom(&self.vrom)
    }
}
