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

mod deref_map_ext;
mod deref_mut_map_ext;
mod map_mut;
mod map_ref;

pub use crate::{
    deref_map_ext::DerefMapExt, deref_mut_map_ext::DerefMutMapExt, map_mut::MapMut, map_ref::MapRef,
};
