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

    let mut input = String::new();
    println!("Please accept the permission pop-up in VTube Studio");

    loop {
        let resp = client
            .send(&HotkeysInCurrentModelRequest {
                model_id: None,
                live2d_item_file_name: None,
            })
            .await?;

        if resp.available_hotkeys.len() == 0 {
            println!(
                "No hotkeys detected! Please add some in the VTube Studio app, then press Enter."
            );
            input.clear();
            std::io::stdin().read_line(&mut input)?;
            continue;
        }

        println!("Choose which hotkey to activate:");

        for (index, hotkey) in resp.available_hotkeys.iter().enumerate() {
            println!(
                "{}:\t{}\t({})",
                index + 1,
                hotkey.name,
                hotkey.type_.as_str()
            );
        }

        input.clear();
        std::io::stdin().read_line(&mut input)?;

        match input.trim().parse::<usize>() {
            Ok(index) => match resp.available_hotkeys.get(index - 1) {
                Some(hotkey) => {
                    println!("Activating hotkey {}", hotkey.name);
                    client
                        .send(&HotkeyTriggerRequest {
                            hotkey_id: hotkey.hotkey_id.clone(),
                            item_instance_id: None,
                        })
                        .await?;
                }
                None => eprintln!(
                    "Could not find hotkey. Value should be between 1 and {}",
                    resp.available_hotkeys.len()
                ),
            },
            Err(e) => eprintln!("Failed to parse input `{}` as number: {}", input, e),
        }
    }
}
