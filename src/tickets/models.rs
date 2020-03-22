use chrono::prelude::*;
use postgres_types::{FromSql, ToSql};
use serde::Serialize;
use tokio_postgres::row::Row;

#[derive(Debug, Serialize, ToSql, FromSql)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct Ticket {
    pub id: i32,
    pub store_id: i32,
    pub name: String,
    pub phone: Option<String>,
    pub docotor_id: Option<i32>,
    pub creation_time: chrono::DateTime<Local>,
    pub started_time: Option<chrono::DateTime<Local>>,
    pub done_time: Option<chrono::DateTime<Local>>,
    pub canceled_time: Option<chrono::DateTime<Local>>,
}

impl From<&Row> for Ticket {
    fn from(row: &Row) -> Self {
        let id = row.get("id");
        let store_id = row.get("store_id");
        let docotor_id = row.get("docotor_id");
        let name = row.get("name");
        let phone = row.get("phone");
        let creation_time = row.get("creation_time");
        let started_time = row.get("started_time");
        let done_time = row.get("done_time");
        let canceled_time = row.get("canceled_time");
        Ticket {
            id,
            store_id,
            docotor_id,
            name,
            phone,
            creation_time,
            started_time,
            done_time,
            canceled_time,
        }
    }
}