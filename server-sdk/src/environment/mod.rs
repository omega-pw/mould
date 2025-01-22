pub mod delete_environment;
pub mod insert_environment;
pub mod query_environment;
pub mod read_environment;
use chrono::DateTime;
use chrono::Utc;
pub mod update_environment;
use chrono;
use serde;
use serde::{Deserialize, Serialize};
use tihu::datetime_format;
use tihu::Id;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Environment {
    pub id: Id,
    pub environment_schema_id: Id,
    pub name: String,
    #[serde(with = "datetime_format")]
    pub created_time: DateTime<Utc>,
    #[serde(with = "datetime_format")]
    pub last_modified_time: DateTime<Utc>,
}

// pub const READ_ENVIRONMENT_API: &str = "/api/environment/readEnvironment";

// #[derive(Serialize, Deserialize, Debug)]
// pub enum KvOperation {
//     Set { key: String, value: String },
//     Del { key: String },
// }

// #[derive(Serialize, Deserialize, Debug)]
// pub enum SqlType {
//     DDL,
//     DML,
// }

// #[derive(Serialize, Deserialize, Debug)]
// pub struct PostgresqlConfig {
//     pub env_key: String,
//     pub sql_type: SqlType,
//     pub sql: String,
// }

// #[derive(Serialize, Deserialize, Debug)]
// pub struct RedisOperation {
//     pub env_key: String,
//     pub kv_operation: KvOperation,
// }

// #[derive(Serialize, Deserialize, Debug)]
// pub struct EtcdOperation {
//     pub env_key: String,
//     pub kv_operation: KvOperation,
// }

// #[derive(Serialize, Deserialize, Debug)]
// pub struct ManualOperation {
//     pub content: String,
// }

// #[derive(Serialize, Deserialize, Debug)]
// pub enum ResourceGroup {
//     PostgresqlConfig(PostgresqlConfig),
//     RedisOperation(RedisOperation),
//     EtcdOperation(EtcdOperation),
//     ManualOperation(ManualOperation),
// }

// #[derive(Serialize, Deserialize, Debug)]
// pub struct Environment {
//     pub id: Id,
//     pub name: String,
//     pub resource_groups: Vec<ResourceGroup>,
// }

// pub type ReadTaskReq = PriKey;
// pub type ReadTaskResp = Option<Environment>;
// pub struct ReadTaskApi;
// impl Api for ReadTaskApi {
//     type Input = ReadTaskReq;
//     type Output = ReadTaskResp;
//     fn namespace() -> LightString {
//         return LightString::from_static(READ_ENVIRONMENT_API);
//     }
// }
