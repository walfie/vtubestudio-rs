use tower::reconnect::Reconnect;
use tower::retry::Policy;
use tower::util::ServiceExt;
use tower::Service;
use vtubestudio::data::*;
use vtubestudio::Error;
use vtubestudio::{
    ApiService, ApiTransport, MakeApiService, TungsteniteCodec, TungsteniteConnector,
};

use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let url = "ws://localhost:8001";

    let mut client = Reconnect::new::<
        ApiService<ApiTransport<WebSocketStream<MaybeTlsStream<TcpStream>>, TungsteniteCodec>>,
        &str,
    >(MakeApiService::new(TungsteniteConnector), url);

    // TODO: Retry request if `ConnectionDropped`
    // let mut client = Retry::new(RetryOnDisconnect, client);

    let mut line = String::new();

    loop {
        println!("Press Enter to send a request");
        std::io::stdin().read_line(&mut line)?;

        let resp = client
            .ready()
            .await?
            .call(RequestEnvelope::new(ApiStateRequest {}.into()))
            .await;

        println!("Received response: {:#?}\n", resp);
    }
}

// TODO: This currently doesn't work
#[derive(Clone)]
struct RetryOnDisconnect;
type E = vtubestudio::Error<
    Error<tungstenite::Error, tungstenite::Error>,
    Error<tungstenite::Error, tungstenite::Error>,
>;
impl Policy<RequestEnvelope, ResponseEnvelope, E> for RetryOnDisconnect {
    type Future = futures_util::future::Ready<Self>;

    fn retry(
        &self,
        _req: &RequestEnvelope,
        result: Result<&ResponseEnvelope, &E>,
    ) -> Option<Self::Future> {
        match result {
            Err(Error::ConnectionDropped) => Some(futures_util::future::ready(Self)),
            Err(_) => None,
            Ok(_) => None,
        }
    }

    fn clone_request(&self, req: &RequestEnvelope) -> Option<RequestEnvelope> {
        Some(req.clone())
    }
}
