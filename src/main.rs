mod downloader;
mod settings;
mod updater;

use std::env;
use futures::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::process::Command;
use anyhow::{Context, Error};
use serde::{Deserialize, Serialize};
use system_uri::{install, App};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use crate::downloader::{handle_download, handle_remove};
use crate::settings::LocalData;
use crate::updater::update;

/// Handle an individual WebSocket connection with broadcasting
async fn handle_connection(stream: TcpStream, _addr: SocketAddr) -> Result<(), Error> {
    let ws_stream = accept_async(stream).await?;

    tokio::task::spawn_blocking(update).await??;

    let (mut write, mut read) = ws_stream.split();
    let data = LocalData::new();
    let message = read.next().await.context("No data sent!")??;
    let handler = match serde_json::from_str(message.to_text()?)? {
        OneclickAction::Download(map) => handle_download(data, map).await,
        OneclickAction::Remove(map) => handle_remove(data, map),
        OneclickAction::Sync(_) => Err(Error::msg("Not implemented!"))
    };
    let result = match handler {
        Ok(_) => OneclickResponse::Ok(),
        Err(err) => OneclickResponse::Err(format!("{}", err))
    };
    write.send(Message::Text(serde_json::to_string(&result)?)).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    match run().await {
        Ok(()) => (),
        Err(err) => {
            println!("Error: {err:?}");
            loop {}
        }
    }
}

async fn run() -> Result<(), Error> {
    tokio::task::spawn_blocking(update).await??;
    let exec = env::current_exe()?.to_str().unwrap().to_string();
    match env::args().len() {
        1 => {
            let app = App::new(
                "beatmapbrowser".to_string(),
                "BigBadE".to_string(),
                "OneClick Client".to_string(),
                exec,
                None,
            );
            install(&app, &["beatblockbrowser".to_string()])?;
        }
        2 => {
            Command::new("cmd")
                .env("RUST_BACKTRACE", "1")
                .args(&["/C", "start", "", &*exec, &env::args().nth(1).unwrap(), "dummy"])
                .spawn()
                .expect("Failed to launch new command window");
        }
        _ => {
            // Define the address to listen on
            let addr = "127.0.0.1:61523";
            let listener = TcpListener::bind(&addr).await?;
            println!("WebSocket server with broadcasting listening on ws://{}", addr);

            // Accept incoming TCP connections
            let (stream, addr) = listener.accept().await?;

            if let Err(e) = handle_connection(stream, addr).await {
                eprintln!("Error handling connection {}: {}", addr, e);
                loop {}
            }
        }
    }
    Ok(())
}

#[derive(Serialize, Deserialize)]
enum OneclickAction {
    Sync(Vec<String>),
    Download(String),
    Remove(String)
}

#[derive(Serialize, Deserialize)]
enum OneclickResponse {
    Ok(),
    Err(String),
}