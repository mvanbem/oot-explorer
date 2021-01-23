use scoped_owner::ScopedOwner;
use std::fmt::{self, Debug};
use std::ops::{Add, AddAssign, Range, Sub, SubAssign};
use thiserror::Error;

use crate::reflect::instantiate::Instantiate;
use crate::reflect::primitive::PrimitiveType;
use crate::reflect::sized::ReflectSized;
use crate::reflect::type_::TypeDescriptor;
use crate::rom::{Rom, RomAddr};
use crate::yaz;

pub const VROM_ADDR_DESC: TypeDescriptor = TypeDescriptor::Primitive(PrimitiveType::VromAddr);

/// An address in VROM.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct VromAddr(pub u32);

impl Debug for VromAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VROM(0x{:08x})", self.0)
    }
}

impl Add<u32> for VromAddr {
    type Output = VromAddr;
    fn add(self, rhs: u32) -> VromAddr {
        VromAddr(self.0 + rhs)
    }
}

impl AddAssign<u32> for VromAddr {
    fn add_assign(&mut self, rhs: u32) {
        self.0 += rhs;
    }
}

impl Sub<VromAddr> for VromAddr {
    type Output = u32;
    fn sub(self, rhs: VromAddr) -> u32 {
        self.0 - rhs.0
    }
}

impl Sub<u32> for VromAddr {
    type Output = VromAddr;
    fn sub(self, rhs: u32) -> VromAddr {
        VromAddr(self.0 - rhs)
    }
}

impl SubAssign<u32> for VromAddr {
    fn sub_assign(&mut self, rhs: u32) {
        self.0 -= rhs;
    }
}

impl<'scope> Instantiate<'scope> for VromAddr {
    fn new(data: &'scope [u8]) -> Self {
        VromAddr(<u32 as Instantiate>::new(data))
    }
}

impl ReflectSized for VromAddr {
    const SIZE: usize = 4;
}

#[derive(Clone)]
pub struct FileSystemTableEntry<'a> {
    rom: Rom<'a>,
    addr: RomAddr,
}

impl<'a> std::fmt::Debug for FileSystemTableEntry<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if f.alternate() {
            f.debug_struct("FileSystemTableEntry")
                .field("addr", &self.addr)
                .field("virtual_start", &self.virtual_start())
                .field("virtual_end", &self.virtual_end())
                .field("physical_start", &self.physical_start())
                .field("physical_end", &self.physical_end())
                .field("is_present", &self.is_present())
                .field("is_compressed", &self.is_compressed())
                .finish()
        } else {
            write!(f, "fs::TableEntry({:?})", self.addr)
        }
    }
}

impl<'a> FileSystemTableEntry<'a> {
    pub const SIZE: u32 = 16;

    pub fn new(rom: Rom<'a>, addr: RomAddr) -> FileSystemTableEntry<'a> {
        FileSystemTableEntry { rom, addr }
    }

    pub fn rom(&self) -> Rom<'a> {
        self.rom
    }

    pub fn addr(&self) -> RomAddr {
        self.addr
    }

    pub fn virtual_start(&self) -> VromAddr {
        VromAddr(self.rom.read_u32_at(self.addr))
    }

    pub fn virtual_end(&self) -> VromAddr {
        VromAddr(self.rom.read_u32_at(self.addr + 4))
    }

    pub fn physical_start(&self) -> RomAddr {
        RomAddr(self.rom.read_u32_at(self.addr + 8))
    }

    pub fn physical_end(&self) -> RomAddr {
        RomAddr(self.rom.read_u32_at(self.addr + 12))
    }

    pub fn virtual_range(&self) -> Range<VromAddr> {
        self.virtual_start()..self.virtual_end()
    }

    pub fn physical_range(&self) -> Range<RomAddr> {
        self.physical_start()..self.physical_end()
    }

    pub fn is_end(&self) -> bool {
        self.virtual_range() == (VromAddr(0)..VromAddr(0))
    }

    pub fn is_present(&self) -> bool {
        self.physical_range() != (RomAddr(0xffffffff)..RomAddr(0xffffffff))
    }

    pub fn is_compressed(&self) -> bool {
        self.physical_end().0 > 0
    }
}

#[derive(Clone)]
pub struct LazyFileSystem<'a> {
    rom: Rom<'a>,
    virtual_starts: Vec<VromAddr>,
    files: Vec<LazyFile<'a>>,
}

impl<'a> LazyFileSystem<'a> {
    pub fn new(rom: Rom<'a>, table_rom_addr: RomAddr) -> LazyFileSystem {
        let mut entry_addr = table_rom_addr;
        let mut virtual_starts = Vec::new();
        let mut files = Vec::new();
        loop {
            let entry = FileSystemTableEntry::new(rom, entry_addr);
            if entry.is_end() {
                break;
            }
            let entry_virtual_range = entry.virtual_range();
            virtual_starts.push(entry_virtual_range.start);
            let entry_physical_range = entry.physical_range();
            files.push(LazyFile {
                metadata: FileMetadata {
                    virtual_range: entry_virtual_range.clone(),
                },
                content: LazyFileContent::Unrealized(if entry.is_compressed() {
                    UnrealizedContent::Compressed {
                        physical_addr: entry_physical_range.start,
                        size: entry_physical_range.end - entry_physical_range.start,
                    }
                } else {
                    UnrealizedContent::Uncompressed {
                        physical_addr: entry_physical_range.start,
                        // Note: In this case, entry_physical_range.end is zero.
                        size: entry_virtual_range.end - entry_virtual_range.start,
                    }
                }),
            });

            entry_addr += FileSystemTableEntry::SIZE;
        }
        LazyFileSystem {
            rom,
            virtual_starts,
            files,
        }
    }

    pub fn len(&self) -> usize {
        self.files.len()
    }

    pub fn metadata(&self, index: usize) -> &FileMetadata {
        &self.files[index].metadata
    }

    pub fn get_file(&mut self, scope: &'a ScopedOwner, index: usize) -> &'a [u8] {
        (&mut self.files[index]).content.get(scope, self.rom)
    }

    pub fn get_to_end_of_file(
        &mut self,
        scope: &'a ScopedOwner,
        addr: VromAddr,
    ) -> Result<&'a [u8], VirtualSliceError> {
        let (file_index, offset) = match self.virtual_starts.binary_search(&addr) {
            Ok(file_index) => {
                // Exact match. The given range starts at the beginning of this file.
                (file_index, 0)
            }
            Err(next_index) => {
                // The range start is between the beginnings of files. The given range starts
                // somewhere within the previous file.
                let file_index = next_index - 1;
                let file_virtual_start = self.virtual_starts[file_index];

                let start_within_file = (addr - file_virtual_start) as usize;
                (file_index, start_within_file)
            }
        };

        let file = self.get_file(scope, file_index);
        file.get(offset..)
            .ok_or_else(move || VirtualSliceError::OutOfRange {
                file_index,
                len: file.len(),
                range: offset..file.len(),
            })
    }

    pub fn get_virtual_slice(
        &mut self,
        scope: &'a ScopedOwner,
        range: Range<VromAddr>,
    ) -> Result<&'a [u8], VirtualSliceError> {
        let (file_index, range) = match self.virtual_starts.binary_search(&range.start) {
            Ok(file_index) => {
                // Exact match. The given range starts at the beginning of this file.
                let end_within_file = (range.end - range.start) as usize;
                let range = 0..end_within_file;
                (file_index, range)
            }
            Err(next_index) => {
                // The range start is between the beginnings of files. The given range starts
                // somewhere within the previous file.
                let file_index = next_index - 1;
                let file_virtual_start = self.virtual_starts[file_index];

                let start_within_file = (range.start - file_virtual_start) as usize;
                let end_within_file = (range.end - file_virtual_start) as usize;
                let range = start_within_file..end_within_file;
                (file_index, range)
            }
        };

        let file = self.get_file(scope, file_index);
        file.get(range.clone())
            .ok_or_else(|| VirtualSliceError::OutOfRange {
                file_index,
                len: file.len(),
                range,
            })
    }

    pub fn get_virtual_slice_or_die(
        &mut self,
        scope: &'a ScopedOwner,
        range: Range<VromAddr>,
    ) -> &'a [u8] {
        self.get_virtual_slice(scope, range).unwrap()
    }
}

#[derive(Clone, Debug, Error)]
pub enum VirtualSliceError {
    #[error("file access out of range: file_index={file_index}, len={len}, range={range:?}")]
    OutOfRange {
        file_index: usize,
        len: usize,
        range: Range<usize>,
    },
}

#[derive(Clone)]
struct LazyFile<'a> {
    metadata: FileMetadata,
    content: LazyFileContent<'a>,
}

#[derive(Clone)]
pub struct FileMetadata {
    virtual_range: Range<VromAddr>,
}

impl FileMetadata {
    pub fn virtual_range(&self) -> Range<VromAddr> {
        self.virtual_range.clone()
    }
}

#[derive(Clone)]
enum LazyFileContent<'a> {
    Unrealized(UnrealizedContent),
    Realized(&'a [u8]),
}

#[derive(Clone, Copy)]
enum UnrealizedContent {
    Uncompressed { physical_addr: RomAddr, size: u32 },
    Compressed { physical_addr: RomAddr, size: u32 },
}

impl<'a> LazyFileContent<'a> {
    fn get(&mut self, scope: &'a ScopedOwner, rom: Rom<'a>) -> &'a [u8] {
        if let LazyFileContent::Unrealized(unrealized) = self {
            *self = LazyFileContent::Realized(
                scope
                    .add(match unrealized {
                        UnrealizedContent::Uncompressed {
                            physical_addr,
                            size,
                        } => rom
                            .slice(*physical_addr, *size)
                            .iter()
                            .copied()
                            .collect::<Vec<u8>>(),
                        UnrealizedContent::Compressed {
                            physical_addr,
                            size,
                        } => yaz::decompress(rom.slice(*physical_addr, *size)).unwrap(),
                    })
                    .as_slice(),
            );
        }
        match self {
            LazyFileContent::Unrealized { .. } => unreachable!(),
            LazyFileContent::Realized(data) => data,
        }
    }
}
