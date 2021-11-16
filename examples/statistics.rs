// This example authenticates and sends a request every time you press the Enter key.

use vtubestudio::data::StatisticsRequest;
use vtubestudio::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stored_token = std::env::var("VTS_AUTH_TOKEN").ok();
    if stored_token.is_some() {
        println!("Attempting to use stored auth token");
    }

    let (mut client, mut new_tokens) = Client::builder()
        .auth_token(stored_token)
        .authentication("vtubestudio-rs example", "Walfie", None)
        .build_tungstenite();

    tokio::spawn(async move {
        // This returns whenever the authentication middleware receives a new auth token.
        // We can handle it by saving it somewhere, etc.
        while let Some(token) = new_tokens.next().await {
            println!("Received new token: {}", token);
        }
    });

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
