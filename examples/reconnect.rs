use tower::reconnect::Reconnect;
use vtubestudio::data::*;
use vtubestudio::{
    ApiService, ApiTransport, Client, MakeApiService, TungsteniteCodec, TungsteniteConnector,
};

use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let url = "ws://localhost:8001";

    // TODO: Retry request if `ConnectionDropped`
    let mut client = Client::new(Reconnect::new::<
        ApiService<ApiTransport<WebSocketStream<MaybeTlsStream<TcpStream>>, TungsteniteCodec>>,
        &str,
    >(MakeApiService::new(TungsteniteConnector), url));

    let mut line = String::new();
    loop {
        println!("Press Enter to send a request");
        std::io::stdin().read_line(&mut line)?;

        let resp = client.send(ApiStateRequest {}).await;

        println!("Received response: {:#?}\n", resp);
    }
}
