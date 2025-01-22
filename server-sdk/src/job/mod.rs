pub mod continue_job;
pub mod delete_job;
pub mod insert_job;
pub mod query_job;
pub mod read_job;
pub mod start_job;
pub mod update_job;
use chrono;
use chrono::DateTime;
use chrono::Utc;
use serde;
use serde::{Deserialize, Serialize};
use tihu::datetime_format;
use tihu::Id;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Job {
    pub id: Id,
    pub environment_schema_id: Id,
    pub name: String,
    pub remark: Option<String>,
    #[serde(with = "datetime_format")]
    pub created_time: DateTime<Utc>,
    #[serde(with = "datetime_format")]
    pub last_modified_time: DateTime<Utc>,
}
