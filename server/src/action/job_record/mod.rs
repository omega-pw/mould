pub mod query_job_record;
pub mod read_job_record;
use crate::model::job_record::enums::Status as RecordStatus;
use crate::model::job_step_record::enums::Status as StepRecordStatus;
use crate::model::job_step_record::enums::StepType;
use crate::model::job_step_record::JobStepRecord;
use crate::model::job_step_resource_record::enums::Status as StepResourceRecordStatus;
use crate::model::job_step_resource_record::JobStepResourceRecord;
use crate::sdk;
use serde::{Deserialize, Serialize};

pub fn to_sdk_record_status(val: RecordStatus) -> sdk::job_record::enums::RecordStatus {
    match val {
        RecordStatus::Running => sdk::job_record::enums::RecordStatus::Running,
        RecordStatus::Success => sdk::job_record::enums::RecordStatus::Success,
        RecordStatus::Failure => sdk::job_record::enums::RecordStatus::Failure,
    }
}

fn to_sdk_step_record_status(val: StepRecordStatus) -> sdk::job_record::enums::StepRecordStatus {
    match val {
        StepRecordStatus::Pending => sdk::job_record::enums::StepRecordStatus::Pending,
        StepRecordStatus::Running => sdk::job_record::enums::StepRecordStatus::Running,
        StepRecordStatus::Success => sdk::job_record::enums::StepRecordStatus::Success,
        StepRecordStatus::Failure => sdk::job_record::enums::StepRecordStatus::Failure,
    }
}

fn to_sdk_step_resource_record_status(
    val: StepResourceRecordStatus,
) -> sdk::job_record::enums::StepResourceRecordStatus {
    match val {
        StepResourceRecordStatus::Pending => {
            sdk::job_record::enums::StepResourceRecordStatus::Pending
        }
        StepResourceRecordStatus::Running => {
            sdk::job_record::enums::StepResourceRecordStatus::Running
        }
        StepResourceRecordStatus::Success => {
            sdk::job_record::enums::StepResourceRecordStatus::Success
        }
        StepResourceRecordStatus::Failure => {
            sdk::job_record::enums::StepResourceRecordStatus::Failure
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum StepRecord {
    Auto {
        job_step_record: JobStepRecord,
        step_resource_record_list: Vec<JobStepResourceRecord>,
    },
    Manual {
        job_step_record: JobStepRecord,
    },
}

pub fn merge_step_and_resource_record(
    mut step_record_list: Vec<JobStepRecord>,
    mut step_resource_record_list: Vec<JobStepResourceRecord>,
) -> Vec<StepRecord> {
    step_record_list.sort_by_key(|item| item.job_step_seq);
    let mut list = Vec::with_capacity(step_record_list.len());
    for step_record in step_record_list {
        match step_record.step_type {
            StepType::Auto => {
                let (resource_record_list, rest): (
                    Vec<JobStepResourceRecord>,
                    Vec<JobStepResourceRecord>,
                ) = step_resource_record_list
                    .into_iter()
                    .partition(|step_resource_record| {
                        step_record.id == step_resource_record.job_step_record_id
                    });
                list.push(StepRecord::Auto {
                    job_step_record: step_record,
                    step_resource_record_list: resource_record_list,
                });
                step_resource_record_list = rest;
            }
            StepType::Manual => {
                list.push(StepRecord::Manual {
                    job_step_record: step_record,
                });
            }
        }
    }
    return list;
}
