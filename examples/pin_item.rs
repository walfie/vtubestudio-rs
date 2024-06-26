// This example demonstrates pinning items.

use base64::Engine;
use vtubestudio::data::{
    AngleRelativeTo, ArtMeshPosition, Event, EventSubscriptionRequest, ItemEventConfig,
    ItemEventType, ItemLoadRequest, ItemPinRequest, ItemUnloadRequest, ModelClickedEventConfig,
    Permission, PermissionRequest, SizeRelativeTo, VertexPinType,
};
use vtubestudio::{Client, ClientEvent};

const PNG_IMAGE_DATA: &[u8] = include_bytes!("walfie-point.png");
const VTS_IMAGE_NAME: &'static str = "example.png";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base64_image = base64::prelude::BASE64_STANDARD.encode(PNG_IMAGE_DATA);

    // Check the `statistics` example to see how to use an
    // existing auth token and how to handle new tokens.
    let (mut client, mut events) = Client::builder()
        .authentication(
            "vtubestudio-rs example",
            "Walfie",
            Some(base64_image.clone().into()),
        )
        .build_tungstenite();

    let subscribe_reqs = [
        EventSubscriptionRequest::subscribe(&ModelClickedEventConfig {
            only_clicks_on_model: true,
        })?,
        EventSubscriptionRequest::subscribe(&ItemEventConfig {
            item_instance_ids: Vec::new(),
            item_file_names: vec![VTS_IMAGE_NAME.to_owned()],
        })?,
    ];

    let mut permission_granted = false;
    while !permission_granted {
        println!("Please accept the permission pop-up in VTube Studio");

        let permission_resp = client
            .send(&PermissionRequest {
                requested_permission: Some(Permission::LoadCustomImagesAsItems.into()),
            })
            .await?;

        permission_granted = permission_resp
            .permissions
            .iter()
            .any(|perm| perm.name == Permission::LoadCustomImagesAsItems && perm.granted);
    }

    println!(
        "\n\nRight-click in VTube Studio to pin an item. Click a pinned item to unload it.\n\n"
    );

    while let Some(client_event) = events.next().await {
        match client_event {
            // We receive a `Disconnected` client event whenever we are disconnected, including on
            // startup. This can be used as a cue to refresh any event subscriptions.
            ClientEvent::Disconnected => {
                println!("Connecting...");

                for req in &subscribe_reqs {
                    // Try to subscribe to events, retrying on failure. Note that the client
                    // attempts to reconnect automatically when sending a request.
                    while let Err(e) = client.send(req).await {
                        eprintln!("Failed to subscribe to events: {e}");
                        eprintln!("Retrying in 2s...");
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    }
                }
            }

            ClientEvent::Api(Event::ModelClicked(event)) => {
                println!("Model click event: {event:?}");

                // Ignore if not right-click
                if !event.mouse_button_id.is_right() {
                    continue;
                }

                let item = client
                    .send(&ItemLoadRequest {
                        file_name: VTS_IMAGE_NAME.to_owned(),
                        position_x: 0.0,
                        position_y: 0.0,
                        size: 0.32,
                        rotation: 0.0,
                        fade_time: 0.1,
                        unload_when_plugin_disconnects: true,
                        custom_data_base64: Some(base64_image.clone()),
                        ..Default::default()
                    })
                    .await?;

                eprintln!("Loaded item: {item:?}");

                if let Some(hit) = event.art_mesh_hits.first() {
                    client
                        .send(&ItemPinRequest {
                            pin: true,
                            item_instance_id: item.instance_id.clone(),
                            angle_relative_to: AngleRelativeTo::RelativeToModel.into(),
                            size_relative_to: SizeRelativeTo::RelativeToCurrentItemSize.into(),
                            vertex_pin_type: VertexPinType::Provided.into(),
                            pin_info: ArtMeshPosition {
                                angle: 0.0,
                                size: 0.0,
                                ..hit.hit_info.clone()
                            },
                        })
                        .await?;
                }
            }

            ClientEvent::Api(Event::Item(event)) => {
                println!("Item click event: {event:?}");
                if event.item_event_type == ItemEventType::Clicked {
                    client
                        .send(&ItemUnloadRequest {
                            instance_ids: vec![event.item_instance_id],
                            ..Default::default()
                        })
                        .await?;
                }
            }

            other => {
                println!("Received event: {other:?}");
            }
        }
    }

    Ok(())
}
