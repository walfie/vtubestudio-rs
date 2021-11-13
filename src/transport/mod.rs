pub(crate) mod api;
pub(crate) mod connector;

pub use crate::transport::api::ApiTransport;

#[cfg(feature = "tokio-tungstenite")]
pub use crate::transport::connector::TungsteniteConnector;

#[cfg(feature = "tokio-tungstenite")]
/// Type alias for a default [`tokio_tungstenite`] sink/stream.
pub type TungsteniteTransport =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

#[cfg(feature = "tokio-tungstenite")]
/// Type alias for an [`ApiTransport`] that handles [`tokio_tungstenite`] messages.
pub type TungsteniteApiTransport =
    ApiTransport<TungsteniteTransport, crate::codec::TungsteniteCodec>;
