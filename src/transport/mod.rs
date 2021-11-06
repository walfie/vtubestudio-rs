pub(crate) mod api;
pub(crate) mod connector;

use crate::codec::TungsteniteCodec;
pub use crate::transport::api::ApiTransport;

use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

pub type TungsteniteTransport = WebSocketStream<MaybeTlsStream<TcpStream>>;
pub type TungsteniteApiTransport = ApiTransport<TungsteniteTransport, TungsteniteCodec>;
