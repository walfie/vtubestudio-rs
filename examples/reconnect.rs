use tower::reconnect::Reconnect;
use tower::util::ServiceExt;
use tower::Service;
use vtubestudio::data::*;
use vtubestudio::{ApiTransport, Client, ClientMaker, TungsteniteCodec, TungsteniteConnector};

use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let url = "ws://localhost:8001";

    // TODO: Retry request if `ConnectionDropped`
    let mut client = Reconnect::new::<
        Client<ApiTransport<WebSocketStream<MaybeTlsStream<TcpStream>>, TungsteniteCodec>>,
        &str,
    >(ClientMaker::new(TungsteniteConnector), url);

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
