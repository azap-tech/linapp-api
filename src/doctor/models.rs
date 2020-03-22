use postgres_types::{FromSql, ToSql};
use serde::{Deserialize, Serialize};
use tokio_postgres::row::Row;

#[derive(Debug, Deserialize, Serialize, ToSql, FromSql)]
#[postgres(name = "doctorstatus")]
pub enum DoctorStatus {
    AVAILABLE,
    PAUSED,
    STOPED,
}

#[derive(Debug, Serialize, ToSql, FromSql)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct Doctor {
    pub id: i32,
    pub store_id: i32,
    pub phone: String,
    pub name: String,
    pub status: DoctorStatus,
    pub avatar: Option<String>,
}
impl From<&Row> for Doctor {
    fn from(row: &Row) -> Self {
        let id: i32 = row.get("id");
        let store_id: i32 = row.get("store_id");
        let phone: String = row.get("phone");
        let name: String = row.get("name");
        let status: DoctorStatus = row.get("status");
        let avatar: Option<String> = row.get("avatar");
        Doctor {
            id,
            store_id,
            phone,
            name,
            status,
            avatar,
        }
    }
}
