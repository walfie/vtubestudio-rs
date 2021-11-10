use vtubestudio::data::*;
use vtubestudio::Client;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let url = "ws://localhost:8001";

    // TODO: function for handling new token
    // Handle the new token (save it somewhere, etc)
    let mut client = Client::builder()
        .authentication("vtubestudio-rs example", "Walfie", None)
        .build_tungstenite(url);

    let mut line = String::new();
    loop {
        println!("Press Enter to send a request");
        std::io::stdin().read_line(&mut line)?;

        let resp = client.send(&StatisticsRequest {}).await;

        match resp {
            Ok(resp) => println!("Received response:\n{:#?}\n", resp),
            Err(e) => println!("Received error:\n{}\n{:#?}", e, e),
        }
    }
}
