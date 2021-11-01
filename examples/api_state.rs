use vtubestudio::data::*;
use vtubestudio::Result;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let mut client = vtubestudio::client::new("ws://localhost:8001").await?;
    let resp = client.send(ApiStateRequest {}).await?;
    dbg!(resp);
    Ok(())
}
