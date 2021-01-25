//! Extension traits for mapping [`Deref`](std::ops::Deref) and [`DerefMut`](std::ops::DerefMut)
//! values.
//!
//! To concisely borrow a value from a container with interior mutability, this crate provides
//! extension traits to adapt a `Deref` value to a new one with a different target type, and the
//! same for `DerefMut`.
//!
//! # Example
//!
//! ```
//! # use std::ops::{Deref, DerefMut};
//! # use std::sync::Mutex;
//! #
//! use deref_map::{DerefMapExt, DerefMutMapExt};
//!
//! struct Foo {
//!     bar: u32,
//! }
//!
//! pub fn borrow_bar_from_foo(foo: &Mutex<Foo>) -> impl Deref<Target = u32> + '_ {
//!     foo.lock().unwrap().map_ref(|foo| &foo.bar)
//! }
//!
//! pub fn borrow_bar_from_foo_mut(foo: &mut Mutex<Foo>) -> impl DerefMut<Target = u32> + '_ {
//!     foo.lock().unwrap().map_mut(|foo_ref| &foo_ref.bar, |foo_mut| &mut foo_mut.bar)
//! }
//! ```
//!
//! The [`map_ref`](crate::DerefMapExt::map_ref) method takes a closure `Fn(&T) -> &R` and wraps a
//! `Deref<Target = T>` into a `Deref<Target = R>`.
//!
//! The `DerefMut` trait requires the `Deref` trait, so [`map_mut`](crate::DerefMutMapExt::map_mut)
//! takes two closures, one `Fn(&T) -> &R` for `Deref`, and one `FnMut(&mut T) -> &mut R` for
//! `DerefMut`.
//!
//! Note the `T + '_` lifetime specifier on both return types. That tells the compiler that the
//! container must remain borrowed while the returned value is live, and in turn permits the
//! returned value to capture the borrowed container.
//!
//! # Motivation
//!
//! It's easy to borrow values from smart pointers like [`Box`], [`Rc`](std::rc::Rc), and
//! [`Arc`](std::sync::Arc) that implement `Deref`.
//!
//! ```
//! # struct Foo {
//! #     bar: u32,
//! # }
//! #
//! fn borrow_bar_from_foo(foo: &Box<Foo>) -> &u32 {
//!     &foo.bar
//! }
//! ```
//!
//! But containers with interior mutability like [`RefCell`](std::cell::RefCell),
//! [`Mutex`](std::sync::Mutex), and [`RwLock`](std::sync::RwLock) cannot implement `Deref`. They
//! have accessors that need to be able to fail in order to enforce the borrowing rules, and they
//! need to be informed when the borrow ends. Instead, their accessors return a wrapper that
//! implements `Deref` (and usually [`Drop`] as well).
//!
//! ## Problem
//!
//! Simply borrowing from the wrapper leaves no way for the wrapper to live as long as the borrow.
//!
//! ```compile_fail
//! # use std::cell::{RefCell};
//! #
//! # struct Foo {
//! #     bar: u32,
//! # }
//! #
//! fn borrow_bar_from_foo(foo: &RefCell<Foo>) -> &u32 {
//!     let wrapper = foo.borrow();
//!     &wrapper.bar
//!     // wrapper is dropped here, so our borrow is invalid
//! }
//! ```
//!
//! ## One-off wrapper type
//!
//! One option is to package the wrapper into the return value to ensure it lives as long as the
//! initial borrow. There's no type in the standard library for this, so create one.
//!
//! ```
//! # use std::cell::{Ref, RefCell};
//! # use std::ops::Deref;
//! #
//! # struct Foo {
//! #     bar: u32,
//! # }
//! #
//! fn borrow_bar_from_foo(foo: &RefCell<Foo>) -> MapFooBar<'_> {
//!     MapFooBar(foo.borrow())
//! }
//!
//! struct MapFooBar<'a>(Ref<'a, Foo>);
//!
//! impl<'a> Deref for MapFooBar<'a> {
//!     type Target = u32;
//!
//!     fn deref(&self) -> &u32 {
//!         &self.0.bar
//!     }
//! }
//! ```
//!
//! But that's more purpose-built code than would be ideal to back a single accessor.
//!
//! ## Generic wrapper type
//!
//! This crate's [`MapRef`] is `MapFooBar` generalized to work for any `Deref` wrapper and an
//! arbitrary transformation.
//!
//! ```
//! # use std::cell::{Ref, RefCell};
//! # use std::ops::Deref;
//! #
//! use deref_map::MapRef;
//!
//! # struct Foo {
//! #     bar: u32,
//! # }
//! #
//! fn borrow_bar_from_foo(foo: &RefCell<Foo>) -> impl Deref<Target = u32> + '_ {
//!     MapRef::<'_, Ref<Foo>, _, u32>::new(foo.borrow(), |foo| &foo.bar)
//!     //     ^^^^^^^^^^^^^^^^^^^^^^^^
//!     // These explicit type parameters demonstrate the generics, but are unnecessary.
//! }
//! ```
//!
//! Note that we are now required to use `impl Trait` syntax in the return type because closure
//! types (like the third parameter to `MapRef` in this example) cannot be named and cannot be
//! inferred with `_` in a return type.
//!
//! ```compile_fail
//! # use std::cell::{Ref, RefCell};
//! # use std::ops::Deref;
//! #
//! use deref_map::MapRef;
//!
//! # struct Foo {
//! #     bar: u32,
//! # }
//! #
//! fn borrow_bar_from_foo(foo: &RefCell<Foo>) -> MapRef::<'_, Ref<Foo>, _, u32> {
//!     //                                                               ^
//!     //                                                     This is not permitted.
//!     MapRef::new(foo.borrow(), |foo| &foo.bar)
//! }
//! ```
//!
//! ## Extension trait
//!
//! This crate's [`DerefMapExt::map_ref`] calls `MapRef::new` without breaking up a method chain.
//!
//! ```
//! # use std::cell::{Ref, RefCell};
//! # use std::ops::Deref;
//! #
//! use deref_map::DerefMapExt;
//!
//! # struct Foo {
//! #     bar: u32,
//! # }
//! #
//! fn borrow_bar_from_foo(foo: &RefCell<Foo>) -> impl Deref<Target = u32> + '_ {
//!     foo.borrow().map_ref(|foo| &foo.bar)
//! }
//! ```
//!
//! ## All of the above for `DerefMut` as well
//!
//! A similar story applies for `DerefMut`, but the one-off wrapper has an additional impl.
//!
//! ```
//! # use std::cell::{RefCell, RefMut};
//! # use std::ops::{Deref, DerefMut};
//! #
//! # struct Foo {
//! #     bar: u32,
//! # }
//! #
//! fn borrow_bar_from_foo_mut(foo: &RefCell<Foo>) -> MapFooBarMut<'_> {
//!     MapFooBarMut(foo.borrow_mut())
//! }
//!
//! struct MapFooBarMut<'a>(RefMut<'a, Foo>);
//!
//! impl<'a> Deref for MapFooBarMut<'a> {
//!     type Target = u32;
//!
//!     fn deref(&self) -> &u32 {
//!         &self.0.bar
//!     }
//! }
//!
//! impl<'a> DerefMut for MapFooBarMut<'a> {
//!     fn deref_mut(&mut self) -> &mut u32 {
//!         &mut self.0.bar
//!     }
//! }
//! ```
//!
//! [`MapMut`] is generic and works with any `DerefMut` wrapper and arbitrary transformations.
//!
//! ```
//! # use std::cell::{RefCell, RefMut};
//! # use std::ops::{Deref, DerefMut};
//! #
//! use deref_map::MapMut;
//!
//! # struct Foo {
//! #     bar: u32,
//! # }
//! #
//! fn borrow_bar_from_foo_mut(foo: &RefCell<Foo>) -> impl DerefMut<Target = u32> + '_ {
//!     // These explicit type parameters demonstrate the generics, but are unnecessary.
//!     //     vvvvvvvvvvvvvvvvvvvvvvvvvvv
//!     MapMut::<'_, RefMut<Foo>, _, _, u32>::new(
//!         foo.borrow_mut(),
//!         |foo_ref| &foo_ref.bar,
//!         |foo_mut| &mut foo_mut.bar,
//!     )
//! }
//! ```
//!
//! And [`DerefMutMapExt`] allows method chaining.
//!
//! ```
//! # use std::cell::{Ref, RefCell};
//! # use std::ops::{Deref, DerefMut};
//! #
//! use deref_map::DerefMutMapExt;
//!
//! # struct Foo {
//! #     bar: u32,
//! # }
//! #
//! fn borrow_bar_from_foo_mut(foo: &RefCell<Foo>) -> impl DerefMut<Target = u32> + '_ {
//!     foo.borrow_mut().map_mut(|foo_ref| &foo_ref.bar, |foo_mut| &mut foo_mut.bar)
//! }
//! ```

mod deref_map_ext;
mod deref_mut_map_ext;
mod map_mut;
mod map_ref;

pub use crate::{
    deref_map_ext::DerefMapExt, deref_mut_map_ext::DerefMutMapExt, map_mut::MapMut, map_ref::MapRef,
};
