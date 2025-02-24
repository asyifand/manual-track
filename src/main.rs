use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::transport::Server;

mod adapter;
mod controllers;

use controllers::manual_track_controller::ManualTrackService;
use crate::adapter::add_track_pubs::AddManualTrackPublisher;
// use adapter::add_track_pubs::AddManualTrackPublisher;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port = env::var("PORT").unwrap_or_else(|_| "50059".to_string());
    let addr = format!("0.0.0.0:{}", port).parse()?;
    let service = ManualTrackService::default();

    println!("ManualTrackServer listening on {}", addr);

    let manual_track = <ManualTrackService as std::default::Default>::default();

    Server::builder()
        .add_service(controllers::grpc::ManualTrackServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
