/// A trait describing how to encode/decode a websocket message. This is provided to allow users to
/// use their own websocket library instead of the default [`tokio_tungstenite`] one.
///
/// # Example
///
/// ```
/// use vtubestudio::codec::MessageCodec;
///
/// // Custom websocket message type
/// pub enum Message {
///     Text(String),
///     Binary(Vec<u8>),
///     Ping(Vec<u8>),
///     Pong(Vec<u8>),
///     Close,
/// }
///
/// #[derive(Debug, Clone)]
/// pub struct MyCustomMessageCodec;
///
/// impl MessageCodec for MyCustomMessageCodec {
///     type Message = Message;
///
///     fn decode(msg: Self::Message) -> Option<String> {
///         match msg {
///             Message::Text(s) => Some(s),
///             _ => None,
///         }
///     }
///
///     fn encode(text: String) -> Self::Message {
///         Message::Text(text)
///     }
/// }
/// ```
pub trait MessageCodec {
    /// The underlying read message type. E.g., [`tungstenite::Message`].
    type ReadMessage;

    /// The underlying write message type. E.g., [`tungstenite::Message`].
    type WriteMessage;

    /// Error type returned on decode failure.
    type Error;

    /// Decodes a websocket text message. `None` values are ignored (E.g., for disregarding ping
    /// messages).
    fn decode(msg: Self::ReadMessage) -> Result<Option<String>, Self::Error>;

    /// Converts a string into a websocket text message.
    fn encode(text: String) -> Self::WriteMessage;
}

crate::cfg_feature! {
    #![feature = "tokio-tungstenite"]

    use tokio_tungstenite::tungstenite;
    use std::convert::Infallible;

    /// A codec describing how to encode/decode [`tungstenite::Message`]s.
    #[derive(Debug, Clone)]
    pub struct TungsteniteCodec;

    impl MessageCodec for TungsteniteCodec {
        type ReadMessage = tungstenite::Message;
        type WriteMessage = tungstenite::Message;
        type Error = Infallible;

        fn decode(msg: Self::ReadMessage) -> Result<Option<String>, Self::Error> {
            Ok(match msg {
                Self::ReadMessage::Text(s) => Some(s),
                _ => None,
            })
        }

        fn encode(text: String) -> Self::WriteMessage {
            Self::WriteMessage::Text(text)
        }
    }
}

crate::cfg_feature! {
    #![feature = "awc"]

    use ::awc::ws::{Frame, Message};

    /// A codec describing how to encode/decode [`awc::ws::Message`]s.
    #[derive(Debug, Clone)]
    pub struct AwcCodec;

    impl MessageCodec for AwcCodec {
        type ReadMessage = Frame;
        type WriteMessage = Message;
        type Error = std::str::Utf8Error;

        // TODO: format
        fn decode(msg: Self::ReadMessage) -> Result<Option<String>, Self::Error> {
            Ok(match msg {
                Self::ReadMessage::Text(s) => Some(std::str::from_utf8(&s)?.to_string()),
                _ => None,
            })
        }

        fn encode(text: String) -> Self::WriteMessage {
            Self::WriteMessage::Text(text)
        }
    }
}
