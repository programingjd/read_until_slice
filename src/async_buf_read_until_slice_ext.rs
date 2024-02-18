use crate::read_until_slice::{read_until_slice, ReadUntilSlice};
use tokio::io::AsyncBufRead;

pub trait AsyncBufReadUntilSliceExt: AsyncBufRead {
    fn read_until_slice<'a, 'b>(
        &'a mut self,
        delimiter: &'b [u8],
        buf: &'a mut Vec<u8>,
    ) -> ReadUntilSlice<'a, 'b, Self>
    where
        Self: Unpin,
    {
        read_until_slice(self, delimiter, buf)
    }
}

impl<R: AsyncBufRead + ?Sized> AsyncBufReadUntilSliceExt for R {}
