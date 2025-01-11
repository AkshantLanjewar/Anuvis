use std::{sync::mpsc, time::Duration};

use tao::event_loop::{ControlFlow, EventLoop};
use tokio::time::sleep;
use wry::webview::WebViewBuilder;
use include_dir::{include_dir, Dir};
use super::ux_message::handle_ux_message;

use super::vite_server::ViteServer;

static DIST_DIR: Dir = include_dir!("ui/dist");

pub async fn launch_ux_loop(devmode: bool) -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new();
    let (tx, rx): (mpsc::Sender<String>, mpsc::Receiver<String>) = mpsc::channel();

    let window = tao::window::WindowBuilder::new()
        .with_title("Anuvis Image Processor")
        .with_inner_size(tao::dpi::LogicalSize::new(1200, 800))
        .build(&event_loop)?;

    // build the vite server if we are in dev mode
    let vite_server = if devmode {
        let server = ViteServer::new()?;
        for _ in 0..50 {
            if reqwest::get(format!("http://localhost:{}", server.port))
                .await
                .is_ok()
            {
                break;
            }
            sleep(Duration::from_millis(100)).await;
        }
        Some(server)
    } else {
        None
    };

    let url = if devmode {
        format!("http://localhost:{}", vite_server.as_ref().unwrap().port)
    } else {
        let index_html = DIST_DIR
            .get_file("index.html")
            .expect("index.html not found")
            .contents_utf8()
            .expect("invalid utf8");
        format!("data:text/html;base64,{}", base64::encode(index_html))
    };

    let webview = WebViewBuilder::new(window)?
        .with_url(&url)?
        .with_ipc_handler(move |_, msg| {
            println!("Received message from UI: {}", msg);
            tx.send(msg.to_string()).unwrap();
        })
        .build()?;

    // Store webview in a way we can send messages back to UI
    event_loop.run(move |_event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        if let Ok(msg) = rx.try_recv() {
            handle_ux_message(&msg, &webview, devmode);
        }
    });
}
