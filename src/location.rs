use crate::error::AppError;
use crate::login::create_user;
use crate::sync_service::Broadcaster;
use actix_session::UserSession;
use actix_web::dev::Payload;
use actix_web::error::ErrorBadRequest;
use actix_web::{
    get, post,
    web::{self, Data, ServiceConfig},
};
use actix_web::{Error, FromRequest, HttpRequest, HttpResponse, Result};
use deadpool_postgres::Pool;
use futures::future::{self, Ready};
use rand::{thread_rng, Rng};
use serde::Deserialize;
use serde_json::json;
use std::sync::Mutex;

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(create_location);
    cfg.service(get_location_events);
}

#[derive(Deserialize)]
pub struct LocationForm {
    name: String,
}

#[post("/api/v2/location")]
pub async fn create_location(
    location_form: web::Json<LocationForm>,
    db_pool: web::Data<Pool>,
) -> Result<HttpResponse, AppError> {
    let location_form = location_form.into_inner();
    let db_conn = db_pool.get().await?;

    let pincode = generate_location_code();
    let user_id = create_user(&db_conn, &pincode).await?;

    let sql = "INSERT into locations (id,name) values ($1, $2)  RETURNING id";
    let row = db_conn
        .query_one(sql, &[&user_id, &location_form.name])
        .await?;
    let id: i32 = row.get(0);
    let resp = json!({"status":"ok", "id":id, "name": location_form.name,"pincode":pincode });
    Ok(HttpResponse::Ok().json(resp))
    // notify display add client to queue
}

#[derive(Deserialize)]
struct Token {
    token: String,
}

#[get("/api/v1/location/events")]
async fn get_location_events(
    broadcaster: Data<Mutex<Broadcaster>>,
    info: web::Query<Token>,
) -> Result<HttpResponse, AppError> {
    let location_id = info.into_inner().token;
    // fixme: retrieve location id from token
    let id: i32 = location_id.parse().map_err(|_| AppError::NotFound)?;
    let rx = broadcaster.lock().unwrap().new_client(id);

    Ok(HttpResponse::Ok()
        .header("content-type", "text/event-stream")
        // work around webpack proxy bug with sse : https://github.com/facebook/create-react-app/issues/1633
        .header("Cache-Control", "no-transform")
        .no_chunking()
        .streaming(rx))
}

fn generate_location_code() -> String {
    let mut rng = thread_rng();
    format!(
        "{}{}{}",
        rng.gen_range(0, 10),
        rng.gen_range(0, 10),
        rng.gen_range(0, 10),
    )
}

pub struct Location {
    pub id: i32,
}

impl FromRequest for Location {
    type Error = Error;
    type Future = Ready<Result<Location, Error>>;
    type Config = ();
    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let session = req.get_session();
        if let Ok(Some(id)) = session.get("azap_location_id") {
            future::ok(Location { id })
        } else {
            future::err(ErrorBadRequest(
                json!({"status":"error","error":"invalid session"}),
            ))
        }
    }
}
