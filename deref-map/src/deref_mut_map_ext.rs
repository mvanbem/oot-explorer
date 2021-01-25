use std::ops::DerefMut;

use crate::MapMut;

/// An extension trait for [`DerefMut`] for transforming the dereferenced value.
///
/// The mapped value may be borrowed from the dereferenced value, allowing for convenient projection
/// from a borrowed value to one of its fields.
///
/// # Example
///
/// ```
/// # use std::cell::RefCell;
/// # use std::ops::{Deref, DerefMut};
/// # use std::sync::{Mutex, RwLock};
/// #
/// use deref_map::{DerefMapExt, DerefMutMapExt};
///
/// struct Foo {
///     bar: u32,
/// }
///
/// fn borrow_bar_from_ref_cell_foo_mut(foo: &RefCell<Foo>) -> impl DerefMut<Target = u32> + '_ {
///     foo.borrow_mut().map_mut(|foo_ref| &foo_ref.bar, |foo_mut| &mut foo_mut.bar)
/// }
///
/// fn borrow_bar_from_mutex_foo_mut(foo: &Mutex<Foo>) -> impl DerefMut<Target = u32> + '_ {
///     foo.lock().unwrap().map_mut(|foo_ref| &foo_ref.bar, |foo_mut| &mut foo_mut.bar)
/// }
///
/// fn borrow_bar_from_rwlock_foo_mut(foo: &RwLock<Foo>) -> impl DerefMut<Target = u32> + '_ {
///     foo.write().unwrap().map_mut(|foo_ref| &foo_ref.bar, |foo_mut| &mut foo_mut.bar)
/// }
/// ```
pub trait DerefMutMapExt<'a>: DerefMut + Sized + 'a {
    fn map_mut<FRef, FMut, R>(self, f_ref: FRef, f_mut: FMut) -> MapMut<'a, Self, FRef, FMut, R>
    where
        FRef: for<'b> Fn(&'b Self::Target) -> &'b R,
        FMut: for<'b> FnMut(&'b mut Self::Target) -> &'b mut R,
        R: ?Sized,
    {
        MapMut::new(self, f_ref, f_mut)
    }
}

impl<'a, T> DerefMutMapExt<'a> for T where T: DerefMut + 'a {}
