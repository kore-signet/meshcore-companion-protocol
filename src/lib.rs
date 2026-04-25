#![no_std]
extern crate alloc;

use alloc::borrow::Cow;

use memchr::memchr;
use meshcore::DecodeError;
use meshcore::io::SliceWriter;
use meshcore::io::TinyReadExt;

pub mod commands;
pub mod responses;

pub trait CompanionSer {
    type Decoded<'data>;

    fn ser_size(&self) -> usize;
    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8];
    fn companion_deserialize<'d>(input: &'d [u8]) -> Result<Self::Decoded<'d>, DecodeError>;
}

fn trim_slice_nils(data: &[u8]) -> &[u8] {
    if let Some(end) = memchr(0, data) {
        &data[..end]
    } else {
        data
    }
}

pub struct NullPaddedString<'a, const SIZE: usize>(pub Cow<'a, str>);

impl<'a, const SIZE: usize> NullPaddedString<'a, SIZE> {
    pub fn encode_to(&self, out: &mut SliceWriter<'_>) {
        NullPaddedSlice::<SIZE>(self.0.as_bytes()).encode_to(out);
    }

    pub fn read<'d>(
        rdr: &mut impl TinyReadExt<'d>,
    ) -> Result<NullPaddedString<'d, SIZE>, DecodeError> {
        let data = rdr.read_chunk::<32>()?;
        core::str::from_utf8(trim_slice_nils(data))
            .map(|v| NullPaddedString(Cow::Borrowed(v)))
            .map_err(DecodeError::from)
    }
}

pub struct NullPaddedSlice<'a, const SIZE: usize>(pub &'a [u8]);

impl<'a, const SIZE: usize> From<&'a [u8]> for NullPaddedSlice<'a, SIZE> {
    fn from(value: &'a [u8]) -> Self {
        debug_assert!(value.len() <= SIZE);
        NullPaddedSlice(value)
    }
}

impl<'a, const SIZE: usize> From<&'a str> for NullPaddedSlice<'a, SIZE> {
    fn from(value: &'a str) -> Self {
        debug_assert!(value.len() <= SIZE);
        NullPaddedSlice(value.as_bytes())
    }
}

impl<'a, const SIZE: usize> NullPaddedSlice<'a, SIZE> {
    pub fn encode_to(&self, out: &mut SliceWriter<'_>) {
        let to_pad = SIZE - self.0.len();
        out.write_slice(self.0);
        out.write_repeated(0, to_pad);
    }

    pub fn read<'d>(
        rdr: &mut impl TinyReadExt<'d>,
    ) -> Result<NullPaddedSlice<'d, SIZE>, DecodeError> {
        let data = rdr.read_chunk::<32>()?;
        Ok(NullPaddedSlice(trim_slice_nils(data)))
    }
}
