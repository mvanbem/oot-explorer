use scoped_owner::ScopedOwner;
use std::marker::PhantomData;

use crate::fs::{LazyFileSystem, VirtualSliceError, VromAddr};
use crate::reflect::instantiate::Instantiate;
use crate::reflect::sized::ReflectSized;

pub trait Delimited {
    fn is_end(&self) -> bool;
}

pub struct Iter<'scope, T> {
    data: Option<&'scope [u8]>,
    _phantom_t: PhantomData<*const T>,
}

impl<'scope, T> Iter<'scope, T>
where
    T: Instantiate<'scope> + ReflectSized,
{
    pub fn new(data: &'scope [u8]) -> Self {
        Self {
            data: Some(data),
            _phantom_t: PhantomData,
        }
    }
}

impl<'scope, T> Iterator for Iter<'scope, T>
where
    T: Instantiate<'scope> + ReflectSized + Delimited,
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        let data = self.data?;
        let value = T::new(data);
        self.data = if value.is_end() {
            None
        } else {
            Some(&data[T::SIZE..])
        };
        Some(value)
    }
}

pub fn is_end<'scope, T>(
    scope: &'scope ScopedOwner,
    fs: &mut LazyFileSystem<'scope>,
    addr: VromAddr,
) -> bool
where
    T: Instantiate<'scope> + ReflectSized + Delimited,
{
    let data = match fs.get_virtual_slice(scope, addr..addr + T::SIZE as u32) {
        Ok(data) => data,
        Err(VirtualSliceError::OutOfRange { .. }) => return true,
    };
    T::new(data).is_end()
}
