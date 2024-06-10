// This example authenticates and sends a request every time you press the Enter key.

use base64::Engine;
use vtubestudio::data::StatisticsRequest;
use vtubestudio::Client;

const PNG_IMAGE_DATA: &[u8] = include_bytes!("walfie-point.png");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base64_image = base64::prelude::BASE64_STANDARD.encode(PNG_IMAGE_DATA);

    let stored_token = std::env::var("VTS_AUTH_TOKEN").ok();
    if stored_token.is_some() {
        println!("Attempting to use stored auth token");
    }

    let (mut client, mut events) = Client::builder()
        .auth_token(stored_token)
        .authentication(
            "vtubestudio-rs example",
            "Walfie",
            Some(base64_image.into()),
        )
        .build_tungstenite();

    tokio::spawn(async move {
        while let Some(event) = events.next().await {
            println!("Received new event: {:?}", event);
        }
    });

    let mut line = String::new();
    loop {
        println!("Press Enter to send a request");
        line.clear();
        std::io::stdin().read_line(&mut line)?;

        let resp = client.send(&StatisticsRequest {}).await;

        match resp {
            Ok(resp) => println!("Received response:\n{:#?}\n", resp),
            Err(e) => println!("Received error:\n{}\n{:#?}", e, e),
        }
    }
}
