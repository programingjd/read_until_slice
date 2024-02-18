use pin_project_lite::pin_project;
use std::future::Future;
use std::io;
use std::marker::PhantomPinned;
use std::mem;
use std::pin::Pin;
use std::task::{ready, Context, Poll};
use tokio::io::AsyncBufRead;

pin_project! {
    /// Future for the [`read_until_slice`](crate::io::AsyncBufReadUntilSliceExt::read_until_slice) method.
    /// The delimiter is included in the resulting vector.
    #[derive(Debug)]
    #[must_use = "futures do nothing unless you `.await` or poll them"]
    pub struct ReadUntilSlice<'a, 'b, R: ?Sized> {
        reader: &'a mut R,
        delimiter: &'b [u8],
        buf: &'a mut Vec<u8>,
        // The number of bytes appended to buf. This can be less than buf.len() if
        // the buffer was not empty when the operation was started.
        read: usize,
        // Make this future `!Unpin` for compatibility with async trait methods.
        #[pin]
        _pin: PhantomPinned,
    }
}

pub(crate) fn read_until_slice<'a, 'b, R>(
    reader: &'a mut R,
    delimiter: &'b [u8],
    buf: &'a mut Vec<u8>,
) -> ReadUntilSlice<'a, 'b, R>
where
    R: AsyncBufRead + ?Sized + Unpin,
{
    ReadUntilSlice {
        reader,
        delimiter,
        buf,
        read: 0,
        _pin: PhantomPinned,
    }
}

pub(crate) fn read_until_slice_internal<R: AsyncBufRead + ?Sized>(
    mut reader: Pin<&mut R>,
    cx: &mut Context<'_>,
    delimiter: &'_ [u8],
    buf: &mut Vec<u8>,
    read: &mut usize,
) -> Poll<io::Result<usize>> {
    let delimiter_len = delimiter.len();
    let mut partial = vec![];
    loop {
        let (done, used) = {
            let available = ready!(reader.as_mut().poll_fill_buf(cx))?;
            if let Some(i) = find_delimiter(
                delimiter,
                delimiter_len,
                partial.iter().copied().chain(available.iter().copied()),
            ) {
                let j = i - partial.len();
                buf.extend_from_slice(&available[..j]);
                (true, j)
            } else {
                buf.extend_from_slice(available);
                let available_len = available.len();
                if available_len >= delimiter_len - 1 {
                    let start = 1 + available_len - delimiter_len;
                    partial = available[start..].to_vec();
                } else {
                    let partial_len = partial.len();
                    if available_len + partial_len < delimiter_len {
                        partial = partial[..]
                            .iter()
                            .chain(available.iter())
                            .copied()
                            .collect();
                    } else {
                        let start = 1 + available_len + partial_len - delimiter_len;
                        partial = partial[start..]
                            .iter()
                            .chain(available.iter())
                            .copied()
                            .collect();
                    }
                }
                (false, available.len())
            }
        };
        reader.as_mut().consume(used);
        *read += used;
        if done || used == 0 {
            return Poll::Ready(Ok(mem::replace(read, 0)));
        }
    }
}

fn find_delimiter(
    delimiter: &[u8],
    n: usize,
    available: impl Iterator<Item = u8>,
) -> Option<usize> {
    let mut match_size = 0usize;
    for (i, it) in available.enumerate() {
        if delimiter[match_size] == it {
            if match_size == n - 1 {
                return Some(i + 1);
            }
            match_size += 1;
        } else if match_size > 0 {
            match_size = 0;
        }
    }
    None
}

impl<R: AsyncBufRead + ?Sized + Unpin> Future for ReadUntilSlice<'_, '_, R> {
    type Output = io::Result<usize>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let me = self.project();
        read_until_slice_internal(Pin::new(*me.reader), cx, me.delimiter, me.buf, me.read)
    }
}
