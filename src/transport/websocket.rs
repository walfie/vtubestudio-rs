use futures_core::Stream;
use futures_sink::Sink;
use std::error::Error as StdError;
use std::fmt::Debug;
use std::marker::PhantomData;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_tungstenite::tungstenite;

pub trait WebSocketTransport: 'static {
    type Message: Debug + Send + 'static;
    type SinkError: StdError + Send + Sync + 'static;
    type StreamError: StdError + Send + Sync + 'static;
    type Underlying: Sink<Self::Message, Error = Self::SinkError>
        + Stream<Item = Result<Self::Message, Self::StreamError>>
        + Send
        + Debug
        + 'static;

    fn extract_text(msg: Self::Message) -> Result<Option<String>, Self::Message>;
    fn create_message(text: String) -> Self::Message;
}

// TODO: Put this behind feature flag
#[derive(Debug)]
pub struct Tungstenite<S>(PhantomData<S>);

impl<S> WebSocketTransport for Tungstenite<S>
where
    S: AsyncRead + AsyncWrite + Unpin + Send + Debug + 'static,
{
    type Message = tungstenite::Message;
    type SinkError = tungstenite::Error;
    type StreamError = tungstenite::Error;
    type Underlying = tokio_tungstenite::WebSocketStream<S>;

    fn extract_text(msg: Self::Message) -> Result<Option<String>, Self::Message> {
        match msg {
            Self::Message::Text(s) => Ok(Some(s)),
            Self::Message::Ping(..) => Ok(None),
            other => Err(other),
        }
    }

    fn create_message(text: String) -> Self::Message {
        Self::Message::Text(text)
    }
}
