/// A trait describing how to encode/decode a websocket message. This is provided to allow users to
/// use their own websocket library instead of the default one.
///
/// Typically the `Input` and `Output` message types will be the same, but they're defined
/// separately to allow for flexibility (e.g., if the underlying websocket client uses distinct
/// types for sending vs receiving, like validating UTF-8 only for outgoing messages).
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
///     type Input = Message;
///     type Ouput = Message;
///     type Error = std::convert::Infallible;
///
///     fn decode(msg: Self::Message) -> Result<Option<String>, Self::Error> {
///         Ok(match msg {
///             Message::Text(s) => Some(s),
///             _ => None,
///         })
///     }
///
///     fn encode(text: String) -> Self::Message {
///         Message::Text(text)
///     }
/// }
/// ```
pub trait MessageCodec {
    /// The underlying incoming message type.
    type Input;

    /// The underlying outgoing message type.
    type Output;

    /// Error type returned on decode failure.
    type Error;

    /// Decodes a websocket text message. `None` values are ignored (E.g., for disregarding ping
    /// messages).
    fn decode(msg: Self::Input) -> Result<Option<String>, Self::Error>;

    /// Converts a string into a websocket text message.
    fn encode(text: String) -> Self::Output;
}

crate::cfg_feature! {
    #![feature = "tokio-tungstenite"]
    pub use self::tungstenite::TungsteniteCodec;
}

#[cfg(feature = "tokio-tungstenite")]
mod tungstenite {
    use super::*;

    use std::convert::Infallible;
    use tokio_tungstenite::tungstenite;

    /// A codec describing how to encode/decode [`tungstenite::Message`]s.
    #[derive(Debug, Clone)]
    pub struct TungsteniteCodec;

    impl MessageCodec for TungsteniteCodec {
        type Input = tungstenite::Message;
        type Output = tungstenite::Message;
        type Error = Infallible;

        fn decode(msg: Self::Input) -> Result<Option<String>, Self::Error> {
            Ok(match msg {
                Self::Input::Text(s) => Some(s),
                _ => None,
            })
        }

        fn encode(text: String) -> Self::Output {
            Self::Output::Text(text)
        }
    }
}
