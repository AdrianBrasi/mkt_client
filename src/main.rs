use dotenv::dotenv;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use tokio::time::{Duration, Timeout, timeout};
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[derive(Serialize)]
struct WebSocketMessage {
    action: String,
    params: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let api_key = dotenv::var("API_KEY").expect("API_KEY must be set");

    let mut url = "wss://delayed.polygon.io/stocks";
    let (mut ws, resp) = connect_async(url).await?;
    println!("HTTP Status: {}", resp.status());

    let auth_msg = WebSocketMessage {
        action: "auth".to_string(),
        params: api_key,
    };
    let auth_json = serde_json::to_string(&auth_msg)?;
    ws.send(Message::Text(auth_json.into())).await?;
    let subscribe_msg = WebSocketMessage {
        action: "subscribe".to_string(),
        params: "AM.SPY".to_string(),
    };
    let subscribe_json = serde_json::to_string(&subscribe_msg)?;
    ws.send(Message::Text(subscribe_json.into())).await?;

    let result = timeout(Duration::from_secs(5), async {
        while let Some(msg) = ws.next().await {
            let msg = msg?;
            match msg {
                Message::Text(text) => println!("Recieved: {}", text),
                Message::Close(close_frame) => {
                    println!("Connection closed: {:?}", close_frame);
                    break;
                }
                _ => continue,
            }
        }
        Ok::<(), Box<dyn std::error::Error>>(())
    })
    .await;
    match result {
        Ok(_) => println!("Loop completed"),
        Err(_) => println!("Loop closed with error"),
    }

    ws.close(None).await?;
    println!("Socket connection closed with {}", url);
    Ok(())
}
