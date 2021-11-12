pub(crate) mod api;
pub(crate) mod connector;

use crate::codec::TungsteniteCodec;

use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

pub use crate::transport::api::ApiTransport;
pub use crate::transport::connector::TungsteniteConnector;

/// Type alias for a default [`tokio_tungstenite`] sink/stream.
pub type TungsteniteTransport = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// Type alias for an [`ApiTransport`] that handles [`tokio_tungstenite`] messages.
pub type TungsteniteApiTransport = ApiTransport<TungsteniteTransport, TungsteniteCodec>;
