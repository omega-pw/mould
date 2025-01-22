use serde;
use serde::{Deserialize, Serialize};
use tihu::Api;
use tihu::Id;
use tihu::LightString;
use tihu::PrimaryKey;

pub const READ_JOB_API: &str = "/api/job/readJob";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum JobStep {
    Auto {
        id: Id,
        name: String,                //步骤名称
        schema_resource_id: Id,      //环境规格资源id
        operation_id: String,        //操作id
        operation_name: String,      //操作名称
        operation_parameter: String, //操作参数
        remark: Option<String>,      //备注
        seq: i32,                    //执行顺序
    },
    Manual {
        id: Id,
        name: String,                //步骤名称
        remark: Option<String>,      //备注
        attachments: Option<String>, //附件
        seq: i32,                    //执行顺序
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Job {
    pub id: Id,
    pub environment_schema_id: Id,
    pub name: String,
    pub remark: Option<String>,
    pub job_step_list: Vec<JobStep>,
}

pub type ReadJobReq = PrimaryKey;
pub type ReadJobResp = Job;
pub struct ReadJobApi;
impl Api for ReadJobApi {
    type Input = ReadJobReq;
    type Output = ReadJobResp;
    fn namespace() -> LightString {
        return LightString::from_static(READ_JOB_API);
    }
}
