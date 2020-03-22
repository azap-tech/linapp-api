use crate::error::AppError;
use crate::location::Location;
use crate::login::create_user;
use crate::sync_service::Broadcaster;
use actix_web::{
    post,
    web::{self, Path, ServiceConfig},
};
use actix_web::{HttpResponse, Result};
use deadpool_postgres::Pool;
use serde::Deserialize;
use serde_json::json;
use std::sync::Mutex;

mod models;

pub use models::{Doctor, DoctorStatus};

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(create_doctor);
    cfg.service(set_doctor_status);
}

#[derive(Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct DoctorForm {
    name: String,
    location_id: i32,
    phone: String,
}

#[post("/api/v2/doctor")]
pub async fn create_doctor(
    doctor_form: web::Json<DoctorForm>,
    db_pool: web::Data<Pool>,
    location_broadcaster: web::Data<Mutex<Broadcaster>>,
) -> Result<HttpResponse, AppError> {
    use rand::Rng;
    let doctor_form = doctor_form.into_inner();
    let db_conn = db_pool.get().await?;
    let mut rng = rand::thread_rng();
    let pincode = format!(
        "{}{}{}{}{}{}",
        rng.gen_range(0, 10),
        rng.gen_range(0, 10),
        rng.gen_range(0, 10),
        rng.gen_range(0, 10),
        rng.gen_range(0, 10),
        rng.gen_range(0, 10)
    );
    let user_id = create_user(&db_conn, &pincode).await?;
    let doctor_row = db_conn
        .query_one(
            "INSERT into doctors (id, name, location_id, phone) values ($1, $2, $3, $4)  RETURNING *",
            &[
                &user_id,
                &doctor_form.name,
                &doctor_form.location_id,
                &doctor_form.phone,
            ],
        )
        .await?;
    let doctor: Doctor = Doctor::from(&doctor_row);
    // fixme define message
    location_broadcaster.lock().unwrap().send(
        doctor_form.location_id,
        &json!({"type":"newdoctor", "payload":doctor}).to_string(),
    );
    Ok(HttpResponse::Ok().json(json!({"status":"ok", "id":doctor.id,"pincode":pincode })))
}

#[derive(Deserialize)]
pub struct DoctorStatusForm {
    status: DoctorStatus,
}

#[post("/api/v2/doctors/{id}/status")]
pub async fn set_doctor_status(
    path: Path<i32>,
    doctor_status_form: web::Json<DoctorStatusForm>,
    location: Location,
    db_pool: web::Data<Pool>,
    location_broacaster: web::Data<Mutex<Broadcaster>>,
) -> Result<HttpResponse, AppError> {
    let doctor_status_form = doctor_status_form.into_inner();
    let status = doctor_status_form.status;
    let doctor_id = path.into_inner();

    let db_conn = db_pool.get().await?;
    let doctor_row = db_conn
        .query_one(
            "UPDATE doctors SET status=$1 WHERE id=$2 RETURNING *",
            &[&status, &doctor_id],
        )
        .await?;

    let doctor: Doctor = Doctor::from(&doctor_row);
    location_broacaster.lock().unwrap().send(
        location.id,
        &json!({"type":"updatedoctorstatus", "payload":{"doctorId": doctor.id, "status": doctor.status}}).to_string(),
    );

    Ok(HttpResponse::Ok().json(json!({"status":"ok", "id":doctor.id })))
}
