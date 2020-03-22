use actix_web::{
    web,
    web::{post, resource, ServiceConfig},
    HttpResponse, Result,
};

use reqwest::{self, Error, Response};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Deserialize, Serialize)]
pub struct UserNotificationForm {
    token: String,
    from: String,
    to: String,
    message: String,
}

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(resource("/api/v2/notification").route(post().to(send_notification)));
}

pub async fn notify_creation(to: &str, name: &str, ticket_id: i32) -> Result<Response, Error> {
    let host = "https://mon-ticket.azap.io";
    let msg = format!(
        r#"Félicitations {},

⌛ Tu es bien inscrit dans la file, tu peux consulter ton temps d'attente ici :

{}/mobile-view/{}

📩 On te prévient par message avant ton passage

👀 Ne t'éloigne pas trop et surveille ton portable pour ne pas perdre ta place

A tout de suite"#,
        name, host, ticket_id
    );
    notify_client(to, &msg).await
}

pub async fn notify_your_turn(to: &str, name: &str) -> Result<Response, Error> {
    notify_client(
        to,
        &format!("{}, c'est à toi !\n\nMerci d'avoir patienté 😉", name),
    )
    .await
}

pub async fn notify_get_closer(to: &str, name: &str, id: i32) -> Result<Response, Error> {
    let host = "https://mon-ticket.azap.io";
    let msg = format!(
        r#" {},
La personne juste avant toi viens de passer sur le siège 🔜💺, rapproche toi du salon c'est bientôt à toi :

{}/mobile-view/{}
"#,
        name, host, id
    );
    notify_client(to, &msg).await
}

async fn notify_client(to: &str, message: &str) -> Result<Response, Error> {
    //fixme: retrive from DB / use store ID?
    //fixme: get url from env and build test server with echo
    let token = "123".into();
    let from = "+33766322917".into();
    let param = UserNotificationForm {
        token,
        from,
        to: to.into(),
        message: message.into(),
    };
    let client = reqwest::Client::new();
    client
        .post("https://azap-sms-gateway.herokuapp.com/send-sms/")
        .json(&param)
        .send()
        .await
}

#[derive(Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct NotificationForm {
    message: String,
    phone_number: String,
}
pub async fn send_notification(
    notification_form: web::Json<NotificationForm>,
) -> Result<HttpResponse> {
    let notification_form = notification_form.into_inner();
    let res = notify_client(&notification_form.phone_number, &notification_form.message).await;
    match res {
        Ok(_) => Ok(HttpResponse::Ok().json(json!({"status":"ok"}))),
        Err(_) => Ok(HttpResponse::BadRequest().json(json!({}))),
    }
}
