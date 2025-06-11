use std::{
    future::Future,
    io,
    marker::PhantomPinned,
    pin::Pin,
    task::{Context, Poll},
};

use pin_project_lite::pin_project;

use crate::AsyncBufRead;

pub(crate) fn peek<'a, R>(reader: &'a mut R, amt: usize) -> Peek<'a, R>
where
    R: AsyncBufRead + Unpin + ?Sized,
{
    Peek {
        reader: Some(reader),
        amt,
        _pin: PhantomPinned,
    }
}

pin_project! {
    /// Future for the [`peek`](crate::AsyncBufReadExt::peek) method.
    #[derive(Debug)]
    #[must_use = "futures do nothing unless you `.await` or poll them"]
    pub struct Peek<'a, R: ?Sized> {
        reader: Option<&'a mut R>,
        amt: usize,
        #[pin]
        _pin: PhantomPinned,
    }

}

impl<'a, R> Future for Peek<'a, R>
where
    R: AsyncBufRead + Unpin + ?Sized,
{
    type Output = io::Result<&'a [u8]>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let me = self.project();

        let reader = me.reader.take().expect("Polled after completion.");
        match Pin::new(&mut *reader).poll_fill_buf(cx, *me.amt) {
            Poll::Ready(Ok(slice)) => unsafe {
                // SAFETY: This is necessary only due to a limitation in the
                // borrow checker. Once Rust starts using the polonius borrow
                // checker, this can be simplified.
                //
                // The safety of this transmute relies on the fact that the
                // value of `reader` is `None` when we return in this branch.
                // Otherwise the caller could poll us again after
                // completion, and access the mutable reference while the
                // returned immutable reference still exists.
                let slice = std::mem::transmute::<&[u8], &'a [u8]>(slice);
                Poll::Ready(Ok(slice))
            },
            Poll::Ready(Err(err)) => Poll::Ready(Err(err)),
            Poll::Pending => {
                *me.reader = Some(reader);
                Poll::Pending
            }
        }
    }
}
