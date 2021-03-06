pub(crate) mod api;
pub(crate) mod connector;

pub use crate::transport::api::ApiTransport;

crate::cfg_feature! {
    #![feature = "tokio-tungstenite"]
    pub use crate::transport::connector::TungsteniteConnector;

    use crate::codec::TungsteniteCodec;
    use tokio_tungstenite::{WebSocketStream, MaybeTlsStream};
    use tokio::net::TcpStream;

    /// Type alias for a default [`tokio_tungstenite`] sink/stream.
    pub type TungsteniteTransport = WebSocketStream<MaybeTlsStream<TcpStream>>;

    /// Type alias for an [`ApiTransport`] that handles [`tokio_tungstenite`] messages.
    pub type TungsteniteApiTransport = ApiTransport<TungsteniteTransport, TungsteniteCodec>;
}
