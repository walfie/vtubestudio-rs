use vtubestudio::data::{MoveModelRequest, StatisticsRequest};
use vtubestudio::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check the `statistics` example to see how to use an
    // existing auth token and how to handle new tokens.
    let (mut client, _) = Client::builder()
        .authentication("vtubestudio-rs example", "Walfie", None)
        .build_tungstenite();

    let mut line = String::new();

    println!("Please accept the permission pop-up in VTube Studio");
    client.send(&StatisticsRequest {}).await?;

    println!("Press Enter to reset model position");
    std::io::stdin().read_line(&mut line)?;

    client
        .send(&MoveModelRequest {
            time_in_seconds: 0.0,
            values_are_relative_to_model: false,
            rotation: Some(0.0),
            ..MoveModelRequest::default()
        })
        .await?;

    println!("Press Enter to start spinning");
    std::io::stdin().read_line(&mut line)?;

    println!("Spinning now... press Ctrl+C to exit");
    loop {
        client
            .send(&MoveModelRequest {
                time_in_seconds: 0.0,
                values_are_relative_to_model: true,
                rotation: Some(6.0),
                ..MoveModelRequest::default()
            })
            .await?;
    }
}
