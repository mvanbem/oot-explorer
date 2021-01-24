use std::ops::Deref;

use crate::MapRef;

/// An extension trait for [`Deref`] for transforming the dereferenced value.
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
/// fn borrow_bar_from_ref_cell_foo(foo: &RefCell<Foo>) -> impl Deref<Target = u32> + '_ {
///     foo.borrow().map_ref(|foo| &foo.bar)
/// }
///
/// fn borrow_bar_from_mutex_foo(foo: &Mutex<Foo>) -> impl Deref<Target = u32> + '_ {
///     foo.lock().unwrap().map_ref(|foo| &foo.bar)
/// }
///
/// fn borrow_bar_from_rwlock_foo(foo: &RwLock<Foo>) -> impl Deref<Target = u32> + '_ {
///     foo.read().unwrap().map_ref(|foo_ref| &foo_ref.bar)
/// }
/// ```
pub trait DerefMapExt<'a>: Deref + Sized + 'a {
    fn map_ref<F, R>(self, f: F) -> MapRef<'a, Self, F, R>
    where
        F: for<'b> Fn(&'b Self::Target) -> &'b R,
        R: ?Sized,
    {
        MapRef::new(self, f)
    }
}

impl<'a, T> DerefMapExt<'a> for T where T: Deref + 'a {}
