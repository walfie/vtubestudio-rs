// This example demonstrates activating hotkeys using the API.

use vtubestudio::data::{HotkeyTriggerRequest, HotkeysInCurrentModelRequest};
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

    loop {
        let resp = client
            .send(&HotkeysInCurrentModelRequest { model_id: None })
            .await?;

        println!("Choose which hotkey to activate:");

        for (index, hotkey) in resp.available_hotkeys.iter().enumerate() {
            println!(
                "{}:\t{}\t({})",
                index + 1,
                hotkey.name,
                hotkey.type_.as_str()
            );
        }

        line.clear();
        std::io::stdin().read_line(&mut line)?;

        match line.trim().parse::<usize>() {
            Ok(index) => match resp.available_hotkeys.get(index - 1) {
                Some(hotkey) => {
                    println!("Activating hotkey {}", hotkey.name);
                    client
                        .send(&HotkeyTriggerRequest {
                            hotkey_id: hotkey.hotkey_id.clone(),
                        })
                        .await?;
                }
                None => eprintln!(
                    "Could not find hotkey. Value should be between 1 and {}",
                    resp.available_hotkeys.len()
                ),
            },
            Err(e) => eprintln!("Failed to parse input {} as number: {}", line, e),
        }
    }
}
