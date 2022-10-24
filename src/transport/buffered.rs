use crate::data::{RequestEnvelope, ResponseEnvelope};

use futures_core::{Stream, TryStream};
use futures_sink::Sink;
use futures_util::stream::{IntoStream, SplitSink};
use futures_util::{StreamExt, TryStreamExt};
use pin_project_lite::pin_project;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::mpsc;

pin_project! {
    /// API transport that buffers elements of the stream.
    ///
    /// This is used to ensure that the underlying transport continues to be polled even if we're
    /// not awaiting paired API responses (e.g., receiving events).
    #[derive(Debug)]
    pub(crate) struct BufferedApiTransport<T> where T: TryStream {
        #[pin]
        sink: SplitSink<IntoStream<T>, RequestEnvelope>,
        #[pin]
        stream: mpsc::Receiver<Result<ResponseEnvelope, T::Error>>,
    }
}

impl<T> BufferedApiTransport<T>
where
    T: Sink<RequestEnvelope> + TryStream<Ok = ResponseEnvelope> + Send + 'static,
    <T as TryStream>::Error: Send + 'static,
{
    /// Creates a new [`BufferedTransport`].
    pub fn new(transport: T, buffer_size: usize) -> Self {
        let (resp_sink, mut resp_stream) = transport.into_stream().split();

        let (buffered_sender, buffered_receiver) = mpsc::channel(buffer_size);

        tokio::spawn(async move {
            while let Some(item) = resp_stream.next().await {
                if buffered_sender.send(item).await.is_err() {
                    tracing::warn!("Dropping message due to buffer being full");
                }
            }

            drop(buffered_sender);
        });

        Self {
            sink: resp_sink,
            stream: buffered_receiver,
        }
    }
}

impl<T> Sink<RequestEnvelope> for BufferedApiTransport<T>
where
    T: Sink<RequestEnvelope> + TryStream,
{
    type Error = <T as Sink<RequestEnvelope>>::Error;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.as_mut().project().sink.poll_ready(cx)
    }

    fn start_send(mut self: Pin<&mut Self>, item: RequestEnvelope) -> Result<(), Self::Error> {
        self.as_mut().project().sink.start_send(item)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.as_mut().project().sink.poll_flush(cx)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.as_mut().project().sink.poll_close(cx)
    }
}

impl<T> Stream for BufferedApiTransport<T>
where
    T: TryStream<Ok = ResponseEnvelope>,
{
    type Item = Result<ResponseEnvelope, T::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().stream.poll_recv(cx)
    }
}
