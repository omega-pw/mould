pub mod query_job_record;
pub mod read_job_record;

pub mod enums {
    use serde::{Deserialize, Serialize};
    #[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
    pub enum RecordStatus {
        Running = 1, //进行中
        Success = 2, //成功
        Failure = 3, //失败
    }
    impl ToString for RecordStatus {
        fn to_string(&self) -> String {
            match *self {
                RecordStatus::Running => "进行中".into(),
                RecordStatus::Success => "成功".into(),
                RecordStatus::Failure => "失败".into(),
            }
        }
    }
    #[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
    pub enum StepType {
        Auto = 1,   //自动
        Manual = 2, //手动
    }
    #[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
    pub enum StepRecordStatus {
        Pending = 1, //未开始
        Running = 2, //进行中
        Success = 3, //成功
        Failure = 4, //失败
    }
    impl ToString for StepRecordStatus {
        fn to_string(&self) -> String {
            match *self {
                StepRecordStatus::Pending => "未开始".into(),
                StepRecordStatus::Running => "进行中".into(),
                StepRecordStatus::Success => "成功".into(),
                StepRecordStatus::Failure => "失败".into(),
            }
        }
    }
    #[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
    pub enum StepResourceRecordStatus {
        Pending = 1, //未开始
        Running = 2, //进行中
        Success = 3, //成功
        Failure = 4, //失败
    }
    impl ToString for StepResourceRecordStatus {
        fn to_string(&self) -> String {
            match *self {
                StepResourceRecordStatus::Pending => "未开始".into(),
                StepResourceRecordStatus::Running => "进行中".into(),
                StepResourceRecordStatus::Success => "成功".into(),
                StepResourceRecordStatus::Failure => "失败".into(),
            }
        }
    }
}
