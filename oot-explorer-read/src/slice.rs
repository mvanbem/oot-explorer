use oot_explorer_vrom::{Vrom, VromAddr};
use std::iter::FusedIterator;
use std::marker::PhantomData;

use crate::{FromVrom, Layout, ReadError};

/// A contiguous slice of sized values in VROM.
#[derive(Clone, Copy)]
pub struct Slice<T>
where
    T: FromVrom + Layout,
{
    addr: VromAddr,
    len: u32,
    phantom_t_: PhantomData<fn() -> T>,
}

impl<T> Slice<T>
where
    T: FromVrom + Layout,
{
    pub fn new(addr: VromAddr, len: u32) -> Self {
        Self {
            addr,
            len,
            phantom_t_: PhantomData,
        }
    }

    pub fn len(self) -> u32 {
        self.len
    }

    pub fn get(self, vrom: Vrom<'_>, index: u32) -> Result<T, ReadError> {
        T::from_vrom(vrom, self.addr + index * T::SIZE)
    }

    pub fn iter(self, vrom: Vrom<'_>) -> SliceIter<'_, T> {
        SliceIter {
            vrom,
            addr: self.addr,
            len: self.len,
            phantom_t_: PhantomData,
        }
    }
}

#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct SliceIter<'a, T>
where
    T: FromVrom + Layout,
{
    vrom: Vrom<'a>,
    addr: VromAddr,
    len: u32,
    phantom_t_: PhantomData<T>,
}

impl<'a, T> Iterator for SliceIter<'a, T>
where
    T: FromVrom + Layout,
{
    type Item = Result<T, ReadError>;

    fn next(&mut self) -> Option<Result<T, ReadError>> {
        if self.len > 0 {
            let result = T::from_vrom(self.vrom, self.addr);
            self.addr += T::SIZE;
            self.len -= 1;
            Some(result)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl<'a, T> ExactSizeIterator for SliceIter<'a, T>
where
    T: FromVrom + Layout,
{
    fn len(&self) -> usize {
        self.len as usize
    }
}

impl<'a, T> FusedIterator for SliceIter<'a, T> where T: FromVrom + Layout {}
