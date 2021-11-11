pub(crate) mod api;
pub(crate) mod connector;

use crate::codec::TungsteniteCodec;
pub use crate::transport::api::ApiTransport;

use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

/// Type alias for a default [`tokio_tungstenite`] sink/stream.
pub type TungsteniteTransport = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// Type alias for an [`ApiTransport`] that handles [`tokio_tungstenite`] messages.
pub type TungsteniteApiTransport = ApiTransport<TungsteniteTransport, TungsteniteCodec>;
