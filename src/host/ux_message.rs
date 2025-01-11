use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct RustMessage {
    message_type: String,
    data: String,
}

pub fn handle_ux_message(msg: &str, webview: &wry::webview::WebView, devmode: bool) {
    if let Ok(rust_msg) = serde_json::from_str::<RustMessage>(msg) {
        match rust_msg.message_type.as_str() {
            "command" => {
                if devmode {
                    println!("Received command message: {}", rust_msg.data);
                }

                // send a dummy response back to the UI
                let response = RustMessage {
                    message_type: "response".to_string(),
                    data: format!("Processed Command: {}", rust_msg.data),
                };

                let js = format!(
                    "window.dispatchEvent(new CustomEvent('rust-message', {{ detail: {} }}))",
                    serde_json::to_string(&response).unwrap()
                );

                webview.evaluate_script(&js).unwrap();
            }
            _ => println!("Received unknown message type: {}", rust_msg.message_type),
        }
    }
}
