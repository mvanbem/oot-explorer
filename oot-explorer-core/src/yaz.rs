use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::ops::Range;

#[derive(Clone, Copy, Debug)]
enum Code {
    Literal(u8),
    Match(Match),
}

#[derive(Clone, Copy, Debug)]
struct Match {
    pub distance: u16,
    pub length: u16,
}
impl Match {
    pub const MIN_DISTANCE: u16 = 1;
    pub const MAX_DISTANCE: u16 = 0x1000;
    pub const MIN_LENGTH: u16 = 3;
    pub const MAX_LENGTH: u16 = 255 + 18;

    pub fn read<R>(mut r: R) -> io::Result<Match>
    where
        R: Read,
    {
        let word = r.read_u16::<BigEndian>()?;
        let distance = (word & 0xfff) + 1;
        let n = (word >> 12) & 0xf;
        let length = if n > 0 {
            // Short length.
            n + 2
        } else {
            // Long length.
            (r.read_u8()? as u16) + 18
        };
        Ok(Match { distance, length })
    }

    pub fn write<W>(&self, mut w: W) -> io::Result<()>
    where
        W: Write,
    {
        if !(Match::MIN_DISTANCE..=Match::MAX_DISTANCE).contains(&self.distance) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "match distance out of range",
            ));
        }
        if !(Match::MIN_LENGTH..=Match::MAX_LENGTH).contains(&self.length) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "match length out of range",
            ));
        }
        let (length1, length2) = if self.length <= 17 {
            (self.length - 2, None)
        } else {
            (0, Some((self.length - 18) as u8))
        };
        let word = (length1 << 12) | (self.distance - 1);
        w.write_u16::<BigEndian>(word)?;
        match length2 {
            Some(x) => w.write_u8(x)?,
            None => (),
        }
        Ok(())
    }
}

pub fn decompressed_size(mut data: &[u8]) -> io::Result<u32> {
    // Skip magic word.
    data = &data[4..];

    data.read_u32::<BigEndian>()
}

pub fn decompress<R>(mut r: R) -> io::Result<Vec<u8>>
where
    R: Read,
{
    // Verify header.
    {
        let mut magic = [0; 4];
        r.read_exact(&mut magic)?;
        // Don't verify the fourth byte, which varies but doesn't seem to affect this
        // algorithm's ability to decompress the stream.
        if &magic[0..3] != "Yaz".as_bytes() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "bad Yaz magic word",
            ));
        }
    }

    let decompressed_size = r.read_u32::<BigEndian>()? as usize;

    // Skip padding.
    r.read_u32::<BigEndian>()?;
    r.read_u32::<BigEndian>()?;

    let mut result = Vec::with_capacity(decompressed_size);
    while result.len() < decompressed_size {
        let mut literal_flags = r.read_u8()?;
        for _bit in 0..8 {
            if (literal_flags & 0x80) == 0x80 {
                // Literal.
                result.push(r.read_u8()?);
            } else {
                // Match.
                let Match { distance, length } = Match::read(&mut r)?;
                let distance = distance as usize;
                for _ in 0..length {
                    result.push(result[result.len() - distance]);
                }
            }
            if result.len() >= decompressed_size {
                break;
            }
            literal_flags <<= 1;
        }
    }
    if result.len() != decompressed_size {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "bad actual decompressed size",
        ));
    }

    Ok(result)
}

struct Buffer {
    // Invariant: Length is always at least 1. `buf[0]` contains the literal flags byte.
    buf: Vec<u8>,
    flag_bits_remaining: u8,
}
impl Buffer {
    fn new() -> Buffer {
        // Longest possible encoded length for a single byte of literal flags. The longest single
        // code is three bytes (a match with u16 code and long length in an additional u8). Eight
        // three-byte codes plus a flags byte is 25.
        let mut buf = Buffer {
            buf: Vec::with_capacity(25),
            flag_bits_remaining: 0,
        };
        buf.clear();
        buf
    }
    fn is_full(&self) -> bool {
        self.flag_bits_remaining == 0
    }
    fn is_empty(&self) -> bool {
        self.flag_bits_remaining == 8
    }
    fn clear(&mut self) {
        self.buf.clear();
        self.buf.push(0);
        self.flag_bits_remaining = 8;
    }
    fn push(&mut self, code: Code) -> io::Result<usize> {
        if self.is_full() {
            return Err(io::Error::new(io::ErrorKind::Other, "push while full"));
        }
        self.flag_bits_remaining -= 1;
        match code {
            Code::Literal(x) => {
                self.buf[0] |= 1 << self.flag_bits_remaining;
                self.buf.push(x);
                Ok(1)
            }
            Code::Match(m) => {
                let len_before = self.buf.len();
                m.write(&mut self.buf)?;
                Ok(self.buf.len() - len_before)
            }
        }
    }
}

struct CodeVector {
    data: Vec<u8>,
    buf: Buffer,
}
impl CodeVector {
    fn new(decompressed_size: u32) -> CodeVector {
        let mut data = Vec::new();
        data.extend_from_slice("Yaz0".as_bytes());
        data.write_u32::<BigEndian>(decompressed_size).unwrap();
        data.write_u32::<BigEndian>(0).unwrap();
        data.write_u32::<BigEndian>(0).unwrap();
        CodeVector {
            data,
            buf: Buffer::new(),
        }
    }
    fn push(&mut self, code: Code) -> usize {
        if self.buf.is_full() {
            self.data.extend_from_slice(&self.buf.buf);
            self.buf.clear();
        }
        self.buf.push(code).unwrap()
    }
    fn into_vec(mut self) -> Vec<u8> {
        if !self.buf.is_empty() {
            self.data.extend_from_slice(&self.buf.buf);
        }
        let padding_len = ((self.data.len() + 15) & !0xf) - self.data.len();
        for _i in 0..padding_len {
            self.data.push(0);
        }
        self.data
    }
}

pub struct MaxEffort(pub usize);
impl MaxEffort {
    pub const DEFAULT: MaxEffort = MaxEffort(100);
}

pub fn compress(data: &[u8], max_effort: MaxEffort) -> Vec<u8> {
    let mut dict: HashMap<&[u8], Vec<Range<usize>>> = HashMap::new();
    let mut result = CodeVector::new(data.len() as u32);
    let mut last_pos = 0;
    let mut pos = 0;
    while pos < data.len() {
        // Remove potential matches that have left the sliding window.
        for remove_pos in last_pos..pos {
            let begin = remove_pos.wrapping_sub(Match::MAX_DISTANCE as usize);
            let end = begin.wrapping_add(3);
            if begin < end && end <= data.len() {
                if let Some(bucket_list) = dict.get_mut(&data[begin..end]) {
                    // The oldest entry will always be the first one in its bucket's list.
                    if bucket_list.len() > 0 {
                        bucket_list.remove(0);
                    }
                }
            }
        }

        // Add potential matches that have entered the sliding window.
        for add_pos in last_pos..pos {
            let begin = add_pos.wrapping_sub(1);
            let end = begin.wrapping_add(3);
            if begin < end && end <= data.len() {
                let bucket_list = dict
                    .entry(&data[begin..end])
                    .or_insert_with(|| Vec::with_capacity(max_effort.0));
                bucket_list.push(begin..end);
                while bucket_list.len() > max_effort.0 {
                    bucket_list.remove(0);
                }
            }
        }

        last_pos = pos;

        if pos + 3 <= data.len() {
            if let Some(bucket_list) = dict.get(&data[pos..pos + 3]) {
                let mut best_match: Option<Match> = None;
                for range in bucket_list.iter().rev() {
                    let match_length = data[range.start..]
                        .iter()
                        .take(Match::MAX_LENGTH as usize)
                        .zip(data[pos..].iter())
                        .take_while(|(a, b)| a == b)
                        .count();
                    if match_length >= (Match::MIN_LENGTH as usize) {
                        if match best_match {
                            Some(m) => match_length > (m.length as usize),
                            None => true,
                        } {
                            best_match = Some(Match {
                                distance: (pos - range.start) as u16,
                                length: match_length as u16,
                            });
                        }
                    }
                }
                if let Some(m) = best_match {
                    result.push(Code::Match(m));
                    pos += m.length as usize;
                    continue;
                }
            }
        }
        result.push(Code::Literal(data[pos]));
        pos += 1
    }
    result.into_vec()
}
