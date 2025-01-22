use serde;
use serde::{Deserialize, Serialize};
use tihu::Api;
use tihu::Id;
use tihu::LightString;

pub const UPDATE_JOB_API: &str = "/api/job/updateJob";

#[derive(Serialize, Deserialize, Debug)]
pub enum JobStep {
    Auto {
        id: Option<Id>,
        name: String,                //步骤名称
        schema_resource_id: Id,      //环境规格资源id
        operation_id: String,        //操作id
        operation_parameter: String, //操作参数
        remark: Option<String>,      //备注
        seq: i32,                    //执行顺序
    },
    Manual {
        id: Option<Id>,
        name: String,                //步骤名称
        remark: Option<String>,      //备注
        attachments: Option<String>, //附件
        seq: i32,                    //执行顺序
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateJobReq {
    pub id: Id,
    pub name: String,
    pub remark: Option<String>,
    pub job_step_list: Vec<JobStep>,
}

pub type UpdateJobResp = ();

pub struct UpdateJobApi;
impl Api for UpdateJobApi {
    type Input = UpdateJobReq;
    type Output = UpdateJobResp;
    fn namespace() -> LightString {
        return LightString::from_static(UPDATE_JOB_API);
    }
}
