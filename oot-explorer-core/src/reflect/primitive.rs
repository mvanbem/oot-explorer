use byteorder::{BigEndian, ReadBytesExt};
use scoped_owner::ScopedOwner;

use crate::fs::{LazyFileSystem, VirtualSliceError, VromAddr};
use crate::segment::SegmentAddr;

use super::type_::TypeDescriptor;

pub const BOOL_DESC: TypeDescriptor = TypeDescriptor::Primitive(PrimitiveType::Bool);
pub const U8_DESC: TypeDescriptor = TypeDescriptor::Primitive(PrimitiveType::U8);
pub const I8_DESC: TypeDescriptor = TypeDescriptor::Primitive(PrimitiveType::I8);
pub const U16_DESC: TypeDescriptor = TypeDescriptor::Primitive(PrimitiveType::U16);
pub const I16_DESC: TypeDescriptor = TypeDescriptor::Primitive(PrimitiveType::I16);
pub const U32_DESC: TypeDescriptor = TypeDescriptor::Primitive(PrimitiveType::U32);
pub const I32_DESC: TypeDescriptor = TypeDescriptor::Primitive(PrimitiveType::I32);

#[derive(Clone, Copy)]
pub enum PrimitiveType {
    Bool,
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    VromAddr,
    SegmentAddr,
}

impl PrimitiveType {
    pub fn name(self) -> &'static str {
        match self {
            PrimitiveType::Bool => "bool",
            PrimitiveType::U8 => "u8",
            PrimitiveType::I8 => "i8",
            PrimitiveType::U16 => "u16",
            PrimitiveType::I16 => "i16",
            PrimitiveType::U32 => "u32",
            PrimitiveType::I32 => "i32",
            PrimitiveType::VromAddr => "VromAddr",
            PrimitiveType::SegmentAddr => "SegmentAddr",
        }
    }

    pub fn size(self) -> Option<u32> {
        match self {
            PrimitiveType::Bool | PrimitiveType::U8 | PrimitiveType::I8 => Some(1),
            PrimitiveType::U16 | PrimitiveType::I16 => Some(2),
            PrimitiveType::U32
            | PrimitiveType::I32
            | PrimitiveType::VromAddr
            | PrimitiveType::SegmentAddr => Some(4),
        }
    }

    pub fn read_as_u32<'scope>(
        self,
        scope: &'scope ScopedOwner,
        fs: &mut crate::fs::LazyFileSystem<'scope>,
        addr: VromAddr,
    ) -> Result<u32, VirtualSliceError> {
        let mut fetch = |size| fs.get_virtual_slice(scope, addr..addr + size);

        Ok(match self {
            PrimitiveType::Bool => (fetch(1)?[0] != 0) as u32,
            PrimitiveType::U8 => fetch(1)?[0] as u32,
            PrimitiveType::I8 => fetch(1)?[0] as i8 as u32,
            PrimitiveType::U16 => fetch(2)?.read_u16::<BigEndian>().unwrap() as u32,
            PrimitiveType::I16 => fetch(2)?.read_i16::<BigEndian>().unwrap() as u32,
            PrimitiveType::U32 | PrimitiveType::VromAddr | PrimitiveType::SegmentAddr => {
                fetch(4)?.read_u32::<BigEndian>().unwrap()
            }
            PrimitiveType::I32 => fetch(4)?.read_i32::<BigEndian>().unwrap() as u32,
        })
    }
}

pub(super) fn dump_primitive<'scope>(
    scope: &'scope ScopedOwner,
    fs: &mut LazyFileSystem<'scope>,
    desc: PrimitiveType,
    addr: VromAddr,
) -> () {
    let mut try_print = || {
        let mut fetch = |size| fs.get_virtual_slice(scope, addr..addr + size);

        match desc {
            PrimitiveType::Bool => print!("{}", fetch(1)?[0] != 0),
            PrimitiveType::U8 => print!("{}", fetch(1)?[0]),
            PrimitiveType::I8 => print!("{}", fetch(1)?[0] as i8),
            PrimitiveType::U16 => {
                print!("{}", fetch(2)?.read_u16::<BigEndian>().unwrap())
            }
            PrimitiveType::I16 => {
                print!("{}", fetch(2)?.read_i16::<BigEndian>().unwrap())
            }
            PrimitiveType::U32 => {
                print!("{}", fetch(4)?.read_u32::<BigEndian>().unwrap())
            }
            PrimitiveType::I32 => {
                print!("{}", fetch(4)?.read_i32::<BigEndian>().unwrap())
            }
            PrimitiveType::VromAddr => {
                print!("{:?}", VromAddr(fetch(4)?.read_u32::<BigEndian>().unwrap()))
            }
            PrimitiveType::SegmentAddr => {
                print!(
                    "{:?}",
                    SegmentAddr(fetch(4)?.read_u32::<BigEndian>().unwrap())
                )
            }
        }
        Ok(())
    };

    if let Err(VirtualSliceError::OutOfRange { .. }) = try_print() {
        print!("(inaccessible)");
    }
}
