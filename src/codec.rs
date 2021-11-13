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
    /// The underlying message type. E.g., [`tungstenite::Message`].
    type Message;

    /// Decodes a websocket text message. `None` values are ignored (E.g., for disregarding ping
    /// messages).
    fn decode(msg: Self::Message) -> Option<String>;

    /// Converts a string into a websocket text message.
    fn encode(text: String) -> Self::Message;
}

crate::cfg_feature! {
    #![feature = "tokio-tungstenite"]

    use tokio_tungstenite::tungstenite;

    /// A codec describing how to encode/decode [`tungstenite::Message`]s.
    #[derive(Debug, Clone)]
    pub struct TungsteniteCodec;

    impl MessageCodec for TungsteniteCodec {
        type Message = tungstenite::Message;

        fn decode(msg: Self::Message) -> Option<String> {
            match msg {
                Self::Message::Text(s) => Some(s),
                _ => None,
            }
        }

        fn encode(text: String) -> Self::Message {
            Self::Message::Text(text)
        }
    }
}
