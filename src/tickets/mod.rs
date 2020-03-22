use crate::error::AppError;
use crate::sync_service::Broadcaster;
use crate::users_notification::{notify_creation, notify_get_closer, notify_your_turn};
use actix_session::Session;
use actix_web::{
    get, patch, post, web,
    web::{Path, ServiceConfig},
    HttpResponse, Result,
};
use chrono::prelude::*;
use deadpool_postgres::Pool;
use postgres_types::ToSql;
use serde::Deserialize;
use serde_json::json;
use std::sync::Mutex;

mod models;
pub use models::Ticket;

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(get_tickets);
    cfg.service(submit_ticket_form);
    cfg.service(update_ticket_status);
    cfg.service(submit_ticket_form_no_login);
}

#[get("/api/v2/ticket")]
async fn get_tickets(db_pool: web::Data<Pool>) -> Result<HttpResponse, AppError> {
    let db_conn = db_pool.get().await?;
    let tickets: Vec<Ticket> = db_conn
        .query("SELECT * from tickets ORDER BY creation_time ASC", &[])
        .await?
        .iter()
        .map(|row| row.into())
        .collect();
    Ok(HttpResponse::Ok().json(tickets))
}

#[derive(Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct TicketForm {
    name: String,
    phone: String,
    sex: String,
    pathology: String,
    age: i32,
    doctor_id: Option<i32>,
    location_id: Option<i32>,
}
#[post("/api/v2/ticket")]
async fn submit_ticket_form(
    //req: HttpRequest,
    ticket_form: web::Json<TicketForm>,
    db_pool: web::Data<Pool>,
    location_broadcaster: web::Data<Mutex<Broadcaster>>,
    session: Session,
) -> Result<HttpResponse, AppError> {
    let ticket_form = ticket_form.into_inner();
    let db_conn = db_pool.get().await?;
    let location_id: i32 = match session.get::<i32>("azap-location")? {
        None => {
            if ticket_form.location_id.is_none() {
                return Err(AppError::NotFound);
            } else {
                ticket_form.location_id.unwrap()
            }
        }
        Some(ok) => ok,
    };
    let doctor_id = match session.get::<i32>("azap-doctor")? {
        Some(ok) => Some(ok),
        None => ticket_form.doctor_id,
    };
    let name = ticket_form.name;
    let phone = ticket_form.phone;
    let creation_time = Local::now();
    let sex = ticket_form.sex;
    let pathology = ticket_form.pathology;
    let age = ticket_form.age;

    // if doctor has no tickets
    let docotor_tickets_rows = db_conn.query("Select * from tickets WHERE location_id=$2 and doctor_id=$1 and done_time is NULL and canceled_time is NULL", &[&doctor_id, &location_id]).await?;
    let started_time = if docotor_tickets_rows.is_empty() {
        Some(creation_time)
    } else {
        None
    };

    let query = r#"INSERT INTO tickets
        (location_id, doctor_id, creation_time, started_time, name, phone, sex, pathology,age)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
        RETURNING *"#;
    let param: &[&(dyn ToSql + Sync)] = &[
        &location_id,
        &doctor_id,
        &creation_time,
        &started_time,
        &name,
        &phone,
        &sex,
        &pathology,
        &age,
    ];
    let ticket_row = db_conn.query_one(query, param).await?;
    let ticket: Ticket = Ticket::from(&ticket_row);

    if notify_creation(&phone, &ticket.name, ticket.id)
        .await
        .is_err()
    {
        return Ok(
            HttpResponse::BadRequest().json(json!({"status":"error","error":"could not send sms"}))
        );
    }

    // fixme define message
    location_broadcaster
        .lock()
        .unwrap()
        .send_new_ticket(location_id, &ticket);
    Ok(HttpResponse::Ok().json(json!({"status":"ok", "id":ticket.id })))
}

#[post("/api/v2/ticket/new")]
async fn submit_ticket_form_no_login(
    //req: HttpRequest,
    ticket_form: web::Json<TicketForm>,
    db_pool: web::Data<Pool>,
    location_broadcaster: web::Data<Mutex<Broadcaster>>,
) -> Result<HttpResponse, AppError> {
    let ticket_form = ticket_form.into_inner();
    let db_conn = db_pool.get().await?;
    let location_id = if ticket_form.location_id.is_none() {
        return Err(AppError::NotFound);
    } else {
        ticket_form.location_id.unwrap()
    };

    let doctor_id = ticket_form.doctor_id;
    let name = ticket_form.name;
    let phone = ticket_form.phone;
    let creation_time = Local::now();
    let sex = ticket_form.sex;
    let pathology = ticket_form.pathology;
    let age = ticket_form.age;

    // if doctor has no tickets
    let docotor_tickets_rows = db_conn.query("Select * from tickets WHERE location_id=$2 and doctor_id=$1 and done_time is NULL and canceled_time is NULL", &[&doctor_id, &location_id]).await?;
    let started_time = if docotor_tickets_rows.is_empty() {
        Some(creation_time)
    } else {
        None
    };

    let query = r#"INSERT INTO tickets
        (location_id, doctor_id, creation_time, started_time, name, phone, sex, pathology,age)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
        RETURNING *"#;
    let param: &[&(dyn ToSql + Sync)] = &[
        &location_id,
        &doctor_id,
        &creation_time,
        &started_time,
        &name,
        &phone,
        &sex,
        &pathology,
        &age,
    ];
    let ticket_row = db_conn.query_one(query, param).await?;
    let ticket: Ticket = Ticket::from(&ticket_row);

    if notify_creation(&phone, &ticket.name, ticket.id)
        .await
        .is_err()
    {
        return Ok(
            HttpResponse::BadRequest().json(json!({"status":"error","error":"could not send sms"}))
        );
    }

    // fixme define message
    location_broadcaster
        .lock()
        .unwrap()
        .send_new_ticket(location_id, &ticket);
    Ok(HttpResponse::Ok().json(json!({"status":"ok", "id":ticket.id })))
}

#[derive(Deserialize, Debug, PartialEq)]
enum TicketStatus {
    DONE,
    STARTED,
    WAITING,
    CANCELED,
}

// FIXME: ERROR handling !
// some error should not stop process
// example : if the endpoint can not send sms but it has already updated the bdd the brodacast should be sended and the id should be returned.
#[patch("/api/v2/ticket/{id}/status")]
async fn update_ticket_status(
    path: Path<i32>,
    ticket_status: web::Json<TicketStatus>,
    db_pool: web::Data<Pool>,
    location_broadcaster: web::Data<Mutex<Broadcaster>>,
) -> Result<HttpResponse, AppError> {
    let ticket_id = path.into_inner();
    let ticket_status = ticket_status.into_inner();
    let db_conn = db_pool.get().await?;

    let row = db_conn
        .query_one("SELECT * from tickets WHERE id=$1", &[&ticket_id])
        .await?;
    let ticket = Ticket::from(&row);

    let first_3: Vec<Ticket> = db_conn.query(
        "SELECT * from tickets WHERE doctor_id=$1 and location_id=$2 and done_time IS NULL and canceled_time IS NULL ORDER BY creation_time ASC LIMIT 3",
        &[&ticket.doctor_id, &ticket.location_id],
    ).await?
    .iter()
    .map(|row| row.into()).collect();

    let now = Local::now();

    let query =
        match ticket_status {
            TicketStatus::WAITING | TicketStatus::STARTED => return Ok(HttpResponse::Ok().json(
                json!({"status":"error", "err":format!("ca not set status {:?}",ticket_status) }),
            )),
            TicketStatus::DONE => "UPDATE tickets SET done_time=$1 WHERE id=$2 returning *",
            TicketStatus::CANCELED => "UPDATE tickets SET canceled_time=$1 WHERE id=$2 returning *",
        };

    let row = db_conn.query_one(query, &[&now, &ticket_id]).await?;
    let ticket = Ticket::from(&row);
    location_broadcaster.lock().unwrap().send(
        ticket.location_id,
        &json!({"type":"updateticket", "payload":ticket}).to_string(),
    );

    if let Some(t0) = first_3.get(0) {
        if ticket.id == t0.id {
            if let Some(t1) = first_3.get(1) {
                let t1_row = db_conn
                    .query_one(
                        "UPDATE tickets SET started_time=$1 WHERE id=$2 returning *",
                        &[&now, &t1.id],
                    )
                    .await?;
                let t1 = Ticket::from(&t1_row);
                if let Some(p1) = &t1.phone {
                    notify_your_turn(&p1, &t1.name).await?;
                }
                location_broadcaster.lock().unwrap().send(
                    t1.location_id,
                    &json!({"type":"updateticket", "payload":t1}).to_string(),
                );

                if let Some(t2) = first_3.get(2) {
                    if let Some(p2) = &t2.phone {
                        notify_get_closer(&p2, &t2.name, t2.id).await?;
                    }
                }
            }
        }
    }
    Ok(HttpResponse::Ok().json(json!({"status":"ok", "id":ticket_id })))
}
