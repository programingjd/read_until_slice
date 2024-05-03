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
    let mut match_len = 0_usize;
    loop {
        let (done, used) = {
            let available = ready!(reader.as_mut().poll_fill_buf(cx))?;
            if let Some(i) = memchr(delimiter, available, &mut match_len) {
                buf.extend_from_slice(&available[..i]);
                (true, i)
            } else {
                buf.extend_from_slice(available);
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

/// Returns the first index matching the `needle` in the `haystack`. `match_len` specifies how
/// many bytes from the needle were already matched during the previous lookup.
/// If we reach the end of the `haystack` with a partial match, then this is a partial match,
/// and we update the `match_len` value accordingly, even though we still return `None`.
pub fn memchr(needle: &[u8], haystack: &[u8], match_len: &mut usize) -> Option<usize> {
    let haystack_len = haystack.len();
    let needle_len = needle.len();
    #[allow(clippy::needless_range_loop)]
    for i in 0..haystack_len {
        if haystack[i] == needle[*match_len] {
            *match_len += 1;
            if *match_len == needle_len {
                return Some(i + 1);
            }
        } else if *match_len > 0 {
            *match_len = 0;
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
