use std::borrow::Cow;
use std::fmt::{self, Debug, Formatter};
use std::ops::Deref;

use crate::fs::VromAddr;

#[derive(Clone)]
pub struct Sourced<T> {
    addr: VromAddr,
    value: T,
}

impl<T> Sourced<T> {
    pub fn new(addr: VromAddr, value: T) -> Sourced<T> {
        Sourced { addr, value }
    }

    pub fn addr(&self) -> VromAddr {
        self.addr
    }
}

impl<T> Deref for Sourced<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}

pub enum Value<'a> {
    Struct(Box<dyn Reflect + 'a>),
    U8(u8),
    U32(u32),
}

impl<'a> Debug for Value<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Value::Struct(reflect) => DebugReflect(reflect.as_ref()).fmt(f),
            Value::U8(value) => <u8 as Debug>::fmt(value, f),
            Value::U32(value) => <u32 as Debug>::fmt(value, f),
        }
    }
}

pub trait Reflect {
    fn name(&self) -> Cow<'static, str>;

    /// Note that for dynamically-sized types, this can require a full parse on every call.
    fn size(&self) -> u32;

    fn addr(&self) -> VromAddr;

    fn iter_fields(&self) -> Box<dyn Iterator<Item = Box<dyn Field + '_>> + '_>;
}

pub struct DebugReflect<'a>(pub &'a dyn Reflect);

impl<'a> Debug for DebugReflect<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut debug_struct = f.debug_struct(self.0.name().as_ref());
        for field in self.0.iter_fields() {
            debug_struct.field(
                field.name().as_ref(),
                match field.try_get() {
                    Some(ref value) => value,
                    None => &Inaccessible,
                },
            );
        }
        debug_struct.finish()
    }
}

pub trait Field {
    fn size(&self) -> u32;
    fn addr(&self) -> VromAddr;

    fn name(&self) -> Cow<'static, str>;
    fn try_get(&self) -> Option<Value>;
}

struct Inaccessible;

impl Debug for Inaccessible {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "(inaccessible)")
    }
}
