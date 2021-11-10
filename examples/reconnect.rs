use vtubestudio::data::*;
use vtubestudio::Client;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let url = std::env::var("VTS_URL").unwrap_or_else(|_| "ws://localhost:8001".to_string());

    let stored_token = std::env::var("VTS_AUTH_TOKEN").ok();
    if stored_token.is_some() {
        println!("Attempting to use stored auth token");
    }

    let (mut client, mut new_tokens) = Client::builder()
        .auth_token(stored_token)
        .authentication("vtubestudio-rs example", "Walfie", None)
        .build_tungstenite(url);

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
