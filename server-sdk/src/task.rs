use serde;
use serde::{Deserialize, Serialize};
use tihu::Api;
use tihu::Id;
use tihu::LightString;
use tihu::PrimaryKey;

pub const READ_TASK_API: &str = "/api/task/readTask";

#[derive(Serialize, Deserialize, Debug)]
pub enum KvOperation {
    Set { key: String, value: String },
    Del { key: String },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SqlType {
    DDL,
    DML,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PostgresqlOperation {
    pub env_key: String,
    pub sql_type: SqlType,
    pub sql: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RedisOperation {
    pub env_key: String,
    pub kv_operation: KvOperation,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EtcdOperation {
    pub env_key: String,
    pub kv_operation: KvOperation,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ManualOperation {
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Operation {
    PostgresqlOperation(PostgresqlOperation),
    RedisOperation(RedisOperation),
    EtcdOperation(EtcdOperation),
    ManualOperation(ManualOperation),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Task {
    pub id: Id,
    pub name: String,
    pub operations: Vec<Operation>,
}

pub type ReadTaskReq = PrimaryKey;
pub type ReadTaskResp = Option<Task>;
pub struct ReadTaskApi;
impl Api for ReadTaskApi {
    type Input = ReadTaskReq;
    type Output = ReadTaskResp;
    fn namespace() -> LightString {
        return LightString::from_static(READ_TASK_API);
    }
}
