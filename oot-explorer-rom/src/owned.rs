use crate::borrowed::Rom;

/// A boxed slice representing all of ROM.
///
/// Function parameters should generally prefer the borrowed [`Rom`].
pub struct OwnedRom {
    data: Box<[u8]>,
}

impl OwnedRom {
    pub fn new(data: Box<[u8]>) -> Self {
        Self { data }
    }

    pub fn borrow(&self) -> Rom<'_> {
        Rom(&self.data)
    }
}
