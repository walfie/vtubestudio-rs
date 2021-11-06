pub mod api;
pub mod connector;

use crate::codec::TungsteniteCodec;
use crate::transport::api::ApiTransport;

use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

pub type TungsteniteTransport = WebSocketStream<MaybeTlsStream<TcpStream>>;
pub type TungsteniteApiTransport = ApiTransport<TungsteniteTransport, TungsteniteCodec>;
