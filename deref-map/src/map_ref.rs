use std::marker::PhantomData;
use std::ops::Deref;

/// An adapter from [`Deref`]`<Target = T>` to `Deref<Target = R>`.
///
/// This `struct` is created by the [`map_ref`](crate::DerefMapExt::map_ref) method on
/// [`DerefMapExt`](crate::DerefMapExt). See its documentation for more.
pub struct MapRef<'a, T, F, R>
where
    R: ?Sized,
{
    inner: T,
    f: F,
    _phantom_data: PhantomData<fn() -> &'a R>,
}

impl<'a, T, F, R> MapRef<'a, T, F, R>
where
    R: ?Sized,
{
    pub fn new(inner: T, f: F) -> Self {
        MapRef {
            inner,
            f,
            _phantom_data: PhantomData,
        }
    }
}

impl<'a, T, F, R> Deref for MapRef<'a, T, F, R>
where
    T: Deref + 'a,
    F: for<'b> Fn(&'b T::Target) -> &'b R,
    R: ?Sized,
{
    type Target = R;

    fn deref(&self) -> &Self::Target {
        (self.f)(self.inner.deref())
    }
}
