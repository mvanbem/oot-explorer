use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// An adapter from [`DerefMut`]`<Target = T>` to `DerefMut<Target = R>`.
///
/// This `struct` is created by the [`map_mut`](crate::DerefMutMapExt::map_mut) method on
/// [`DerefMutMapExt`](crate::DerefMutMapExt). See its documentation for more.
pub struct MapMut<'a, T, FRef, FMut, R>
where
    R: ?Sized,
{
    inner: T,
    f_ref: FRef,
    f_mut: FMut,
    _phantom_data: PhantomData<fn() -> &'a R>,
}

impl<'a, T, FRef, FMut, R> MapMut<'a, T, FRef, FMut, R>
where
    R: ?Sized,
{
    pub fn new(inner: T, f_ref: FRef, f_mut: FMut) -> Self {
        MapMut {
            inner,
            f_ref,
            f_mut,
            _phantom_data: PhantomData,
        }
    }
}

impl<'a, T, FRef, FMut, R> Deref for MapMut<'a, T, FRef, FMut, R>
where
    T: DerefMut + 'a,
    FRef: for<'b> Fn(&'b T::Target) -> &'b R,
    FMut: for<'b> FnMut(&'b mut T::Target) -> &'b mut R,
    R: ?Sized,
{
    type Target = R;

    fn deref(&self) -> &Self::Target {
        (self.f_ref)(self.inner.deref())
    }
}

impl<'a, T, FRef, FMut, R> DerefMut for MapMut<'a, T, FRef, FMut, R>
where
    T: DerefMut + 'a,
    FRef: for<'b> Fn(&'b T::Target) -> &'b R,
    FMut: for<'b> FnMut(&'b mut T::Target) -> &'b mut R,
    R: ?Sized,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        (self.f_mut)(self.inner.deref_mut())
    }
}
