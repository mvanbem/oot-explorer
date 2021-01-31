use oot_explorer_rom::{Rom, RomAddr, RomError};
use std::borrow::{Borrow, Cow};
use thiserror::Error;

use crate::file_system_table_entry::FileSystemTableEntry;

mod addr;
mod borrowed;
mod error;
mod file_system_table_entry;
mod file_table;
mod owned;
pub mod yaz;

pub use addr::VromAddr;
pub use borrowed::Vrom;
pub use error::VromError;
pub use file_table::{FileIndex, FileTable, GetFileError};
pub use owned::OwnedVrom;

pub fn decompress(
    rom: Rom<'_>,
    file_table_addr: RomAddr,
) -> Result<(FileTable, OwnedVrom), DecompressError> {
    let mut entry_addr = file_table_addr;
    let mut file_ranges = vec![];
    let mut vrom = vec![];
    loop {
        // Locate the table entry.
        let entry = FileSystemTableEntry::from_rom(rom, entry_addr)?;
        if entry.is_end() {
            break;
        }

        // Record the file's VROM range.
        file_ranges.push(entry.virtual_range());

        if entry.is_present() {
            // Grow the VROM buffer if needed.
            let start = entry.virtual_start.0 as usize;
            let end = entry.virtual_end.0 as usize;
            if vrom.len() < end {
                vrom.resize(end, 0x00);
            }

            // Retrieve the file's data.
            let file_data = if entry.is_compressed() {
                Cow::Owned(yaz::decompress(rom.slice(entry.physical_range())?)?)
            } else {
                Cow::Borrowed(rom.slice(
                    entry.physical_start
                        ..entry.physical_start + (entry.virtual_end - entry.virtual_start),
                )?)
            };

            // Copy the file into the VROM buffer.
            (&mut vrom[start..end]).copy_from_slice(file_data.borrow());
        }

        entry_addr += FileSystemTableEntry::SIZE;
    }
    let vrom = vrom.into_boxed_slice();

    Ok((FileTable { file_ranges }, OwnedVrom::new(vrom)))
}

#[derive(Debug, Error)]
pub enum DecompressError {
    #[error("{0}")]
    RomError(#[from] RomError),

    #[error("{0}")]
    YazError(#[from] yaz::DecompressError),
}
