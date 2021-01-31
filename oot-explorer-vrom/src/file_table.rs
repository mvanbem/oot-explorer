use std::ops::Range;
use thiserror::Error;

use crate::VromAddr;

#[derive(Debug)]
pub struct FileIndex(pub u32);

/// A processed file table for use after VROM is decompressed.
#[derive(Clone)]
pub struct FileTable {
    pub(crate) file_ranges: Vec<Range<VromAddr>>,
}

impl FileTable {
    pub fn file_vrom_range(&self, index: FileIndex) -> Result<Range<VromAddr>, GetFileError> {
        Ok(self
            .file_ranges
            .get(index.0 as usize)
            .ok_or_else(|| GetFileError::InvalidFileIndex {
                index,
                file_count: self.file_ranges.len() as u32,
            })?
            .clone())
    }
}

#[derive(Debug, Error)]
pub enum GetFileError {
    #[error("invalid file index: {index:?}, file count {file_count}")]
    InvalidFileIndex { index: FileIndex, file_count: u32 },
}
