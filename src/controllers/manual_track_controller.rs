use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};

use super::grpc::{
    AddManualTrackReq, AddManualTrackRes, ManualTrack, UpdateManualTrackReq, UpdateManualTrackRes,
};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct ManualTrackService {
    map: Arc<Mutex<HashMap<String, String>>>,
}

#[tonic::async_trait]
impl ManualTrack for ManualTrackService {
    async fn add_manual_track(
        &self,
        request: Request<AddManualTrackReq>,
    ) -> Result<Response<AddManualTrackRes>, Status> {
        let req = request.into_inner();

        if req.identity.is_empty() || req.r#type.is_empty() {
            return Err(Status::invalid_argument(
                "Identity and Type must not be empty",
            ));
        }
        let mut map = self.map.lock().await;
        // save 
        map.insert(format!("MA{}", req.environment), req.environment);
        // cek total
        println!("total: {}", map.len());

        let response = AddManualTrackRes {
            id: "generated-id-123".to_string(),
            message: "Manual track successfully added".to_string(),
            source: req.r#type.clone(),
            id_source: req.identity.clone(),
            quality: 100,
            total: 1,
            status: "SUCCESS".to_string(),
        };

        Ok(Response::new(response))
    }

    async fn update_manual_track(
        &self,
        request: Request<UpdateManualTrackReq>,
    ) -> Result<Response<UpdateManualTrackRes>, Status> {
        println!("data: {:?}", request);
        let req = request.into_inner();
        let mut map = self.map.lock().await;

        // cek di hashmap
        if !map.contains_key(&format!("MA{}", req.environment)) {
            return Err(Status::invalid_argument("Not Valid Environment"));
        } else {

            // hapus dari hashmap
            map.remove(&format!("MA{}", req.environment));

            let response = UpdateManualTrackRes {
                id: req.id.clone(),
                message: "update success".to_string(),
                longitude: req.longitude.clone(),
                latitude: req.latitude.clone(),
                source: "AA".to_string(),
                id_source: req.id.clone(),
                quality: 10,
            };
            Ok(Response::new(response))
        }
    }
}
