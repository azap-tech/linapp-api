use actix_web::{
    get,
    web::{Data, Path, ServiceConfig},
    HttpResponse, Responder,
};
use std::sync::Mutex;

mod broadcaster;
pub use broadcaster::{Broadcaster, Client};

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(new_client);
}

#[get("/api/v2/store/{id}/events")]
async fn new_client(broadcaster: Data<Mutex<Broadcaster>>, info: Path<i32>) -> impl Responder {
    let store_id = info.into_inner();
    let rx = broadcaster.lock().unwrap().new_client(store_id);

    HttpResponse::Ok()
        .header("content-type", "text/event-stream")
        // work arorund webpack proxy bug with sse : https://github.com/facebook/create-react-app/issues/1633
        .header("Cache-Control", "no-transform")
        .no_chunking()
        .streaming(rx)
}
