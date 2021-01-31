use oot_explorer_vrom::{Vrom, VromAddr};
use std::iter::FusedIterator;
use std::marker::PhantomData;

use crate::{FromVrom, Layout, ReadError};

/// Types with sentinel values that may end a list.
pub trait Sentinel {
    const ITER_YIELDS_SENTINEL_VALUE: bool;

    fn is_end(&self, vrom: Vrom<'_>) -> bool;
}

#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct SentinelIter<'a, T> {
    vrom: Vrom<'a>,
    addr: Option<VromAddr>,
    _phantom_t: PhantomData<fn() -> T>,
}

impl<'a, T> SentinelIter<'a, T>
where
    T: FromVrom + Layout,
{
    pub fn new(vrom: Vrom<'a>, addr: VromAddr) -> Self {
        Self {
            vrom,
            addr: Some(addr),
            _phantom_t: PhantomData,
        }
    }
}

impl<'a, T> Iterator for SentinelIter<'a, T>
where
    T: FromVrom + Layout + Sentinel,
{
    type Item = Result<T, ReadError>;

    fn next(&mut self) -> Option<Result<T, ReadError>> {
        // Early out if the iterator has already ended.
        let addr = self.addr?;

        // Attempt to construct the value. If inaccessible, treat it as a sentinel value.
        let (is_end, result) = match T::from_vrom(self.vrom, addr) {
            Ok(value) => (value.is_end(self.vrom), Ok(value)),
            Err(e) => (true, Err(e)),
        };

        // Advance the address unless this was a sentinel value.
        self.addr = if is_end { None } else { Some(addr + T::SIZE) };

        if is_end && !T::ITER_YIELDS_SENTINEL_VALUE {
            None
        } else {
            Some(result)
        }
    }
}

impl<'a, T> FusedIterator for SentinelIter<'a, T> where T: FromVrom + Layout + Sentinel {}

pub fn is_end<T>(vrom: Vrom<'_>, addr: VromAddr) -> bool
where
    T: FromVrom + Layout + Sentinel,
{
    match T::from_vrom(vrom, addr) {
        Ok(value) => value.is_end(vrom),
        Err(_) => true,
    }
}
