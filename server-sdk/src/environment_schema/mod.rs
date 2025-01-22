pub mod delete_environment_schema;
pub mod query_environment_schema;
pub mod read_environment_schema;
pub mod save_environment_schema;
use chrono;
use chrono::DateTime;
use chrono::Utc;
use serde;
use serde::{Deserialize, Serialize};
use tihu::datetime_format;
use tihu::Id;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EnvironmentSchema {
    pub id: Id,
    pub name: String,
    #[serde(with = "datetime_format")]
    pub created_time: DateTime<Utc>,
    #[serde(with = "datetime_format")]
    pub last_modified_time: DateTime<Utc>,
}
