use std::fmt::{self, Debug, Formatter};

use crate::reflect::instantiate::Instantiate;
use crate::reflect::sized::ReflectSized;

/// A contiguous slice of structs of known size.
#[derive(Clone, Copy)]
pub struct Slice<'a, T>
where
    T: Instantiate<'a> + ReflectSized,
{
    data: &'a [u8],
    len: usize,
    phantom_t_: std::marker::PhantomData<&'a [T]>,
}

impl<'a, T> Debug for Slice<'a, T>
where
    T: Instantiate<'a> + ReflectSized,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("header::Slice")
            .field("data", &self.data)
            .field("len", &self.len)
            .finish()
    }
}

impl<'a, T> Slice<'a, T>
where
    T: Instantiate<'a> + ReflectSized,
{
    pub fn new(data: &'a [u8], len: usize) -> Slice<'a, T> {
        Slice {
            data,
            len,
            phantom_t_: std::marker::PhantomData,
        }
    }

    pub fn len(self) -> usize {
        self.len
    }

    pub fn get(self, index: usize) -> T {
        let base = index * <T as ReflectSized>::SIZE;
        <T as Instantiate>::new(&self.data[base..base + <T as ReflectSized>::SIZE])
    }

    pub fn iter(self) -> Iter<'a, T> {
        Iter {
            data: self.data,
            len: self.len,
            phantom_t_: std::marker::PhantomData,
        }
    }
}

impl<'a, T> IntoIterator for Slice<'a, T>
where
    T: Instantiate<'a> + ReflectSized,
{
    type Item = T;
    type IntoIter = Iter<'a, T>;
    fn into_iter(self) -> Iter<'a, T> {
        self.iter()
    }
}

pub struct Iter<'a, T>
where
    T: Instantiate<'a> + ReflectSized,
{
    data: &'a [u8],
    len: usize,
    phantom_t_: std::marker::PhantomData<T>,
}

impl<'a, T> Iterator for Iter<'a, T>
where
    T: Instantiate<'a> + ReflectSized,
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        if self.len > 0 {
            let item = <T as Instantiate>::new(self.data);
            self.data = &self.data[<T as ReflectSized>::SIZE..];
            self.len -= 1;
            Some(item)
        } else {
            None
        }
    }
}
