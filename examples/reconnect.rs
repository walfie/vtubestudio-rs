use tower::reconnect::Reconnect;
use vtubestudio::data::*;
use vtubestudio::service::TungsteniteApiService;
use vtubestudio::{Client, MakeApiService};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let url = "ws://localhost:8001";

    // TODO: Retry request if `ConnectionDropped`
    let mut client = Client::new(Reconnect::new::<TungsteniteApiService, &str>(
        MakeApiService::new_tungstenite(),
        url,
    ));

    let mut line = String::new();
    loop {
        println!("Press Enter to send a request");
        std::io::stdin().read_line(&mut line)?;

        let resp = client.send(ApiStateRequest {}).await;

        println!("Received response: {:#?}\n", resp);
    }
}
