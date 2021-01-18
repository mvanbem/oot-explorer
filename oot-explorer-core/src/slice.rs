use std::fmt::{self, Debug, Formatter};

/// Trait that enables [Slice].
pub trait StructReader<'a> {
    const SIZE: usize;
    fn new(data: &'a [u8]) -> Self;
}

/// A contiguous slice of structs of known size.
#[derive(Clone, Copy)]
pub struct Slice<'a, T>
where
    T: StructReader<'a>,
{
    data: &'a [u8],
    len: usize,
    phantom_t_: std::marker::PhantomData<&'a [T]>,
}

impl<'a, T> Debug for Slice<'a, T>
where
    T: StructReader<'a>,
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
    T: StructReader<'a>,
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
        <T as StructReader>::new(
            &self.data[index * <T as StructReader>::SIZE..(index + 1) * <T as StructReader>::SIZE],
        )
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
    T: StructReader<'a>,
{
    type Item = T;
    type IntoIter = Iter<'a, T>;
    fn into_iter(self) -> Iter<'a, T> {
        self.iter()
    }
}

pub struct Iter<'a, T>
where
    T: StructReader<'a>,
{
    data: &'a [u8],
    len: usize,
    phantom_t_: std::marker::PhantomData<T>,
}

impl<'a, T> Iterator for Iter<'a, T>
where
    T: StructReader<'a>,
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        if self.len > 0 {
            let item = <T as StructReader>::new(self.data);
            self.data = &self.data[<T as StructReader>::SIZE..];
            self.len -= 1;
            Some(item)
        } else {
            None
        }
    }
}
