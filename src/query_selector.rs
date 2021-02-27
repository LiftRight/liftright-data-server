use crate::{
    imurecords::{ImuRecordPair, ImuRecordSet},
    set::Set,
    survey::{Survey, SurveyResponse},
    traits::{BucketUpdate, IdBucketPattern, Sanitize},
    LrdsError, LrdsResult,
};

use mongodb::bson;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use uuid::Uuid;

const BUCKET_IMU_RECORD_LIMIT: i32 = 1000;
const BUCKET_SET_REPETITION_LIMIT: i32 = 1000;

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
enum RecordKind {
    Repetition,
    ImuData,
    Survey,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ImuRecordSetQuery {
    id: bson::Bson,
    kind: RecordKind,
    device_id: Uuid,
    session_id: Uuid,
    imu_record_count: bson::Bson,
}

impl From<&ImuRecordSet> for ImuRecordSetQuery {
    fn from(imu_set: &ImuRecordSet) -> Self {
        ImuRecordSetQuery {
            id: bson::Bson::to_bucket_selector(imu_set.session.device_id),
            kind: RecordKind::ImuData,
            device_id: imu_set.session.device_id,
            session_id: imu_set.session.session_id,
            imu_record_count: bson::bson!({ "$lt": BUCKET_IMU_RECORD_LIMIT }),
        }
    }
}

pub struct ImuRecordSetUpdate<'a> {
    on_field: &'a str,
    with_value: &'a [ImuRecordPair],
    increment_field: &'a str,
    id_prefix: String,
}

impl<'a> From<&'a ImuRecordSet> for ImuRecordSetUpdate<'a> {
    fn from(imu_set: &'a ImuRecordSet) -> Self {
        ImuRecordSetUpdate {
            on_field: "imu_data",
            with_value: &imu_set.data[..],
            increment_field: "imu_record_count",
            id_prefix: imu_set.session.device_id.sanitize(),
        }
    }
}

impl TryFrom<&ImuRecordSetUpdate<'_>> for bson::Document {
    type Error = LrdsError;

    fn try_from(update_values: &ImuRecordSetUpdate) -> LrdsResult<Self> {
        Ok(bson::Document::to_bucket_update(
            update_values.on_field,
            &update_values.with_value,
            update_values.increment_field,
            update_values.with_value.len() as i32,
            &update_values.id_prefix,
        ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SurveyFilter {
    device_id: Uuid,
    kind: RecordKind,
}

impl From<&Survey> for SurveyFilter {
    fn from(survey: &Survey) -> Self {
        SurveyFilter {
            device_id: survey.device_id,
            kind: RecordKind::Survey,
        }
    }
}

impl TryFrom<&Survey> for bson::Document {
    type Error = LrdsError;
    fn try_from(survey: &Survey) -> LrdsResult<Self> {
        bson::to_document(&SurveyFilter::from(survey)).map_err(LrdsError::DbSerializationError)
    }
}

pub struct SurveyUpdate<'a> {
    on_field: &'a str,
    with_value: &'a [SurveyResponse],
    increment_field: &'a str,
    id_prefix: String,
}

impl<'a> From<&'a Survey> for SurveyUpdate<'a> {
    fn from(survey: &'a Survey) -> Self {
        SurveyUpdate {
            on_field: "surveys",
            with_value: &survey.data[..],
            increment_field: "survey_count",
            id_prefix: survey.device_id.sanitize(),
        }
    }
}

impl TryFrom<&SurveyUpdate<'_>> for bson::Document {
    type Error = LrdsError;

    fn try_from(update_values: &SurveyUpdate) -> LrdsResult<Self> {
        Ok(bson::Document::to_bucket_update(
            update_values.on_field,
            &update_values.with_value,
            update_values.increment_field,
            1,
            &update_values.id_prefix,
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetQuery {
    id: bson::Bson,
    kind: RecordKind,
    device_id: Uuid,
    session_id: Uuid,
    set_id: Uuid,
    rep_count: bson::Bson,
}

impl From<&Set> for SetQuery {
    fn from(set: &Set) -> Self {
        SetQuery {
            id: bson::Bson::to_bucket_selector(set.session.device_id),
            kind: RecordKind::Repetition,
            device_id: set.session.device_id,
            session_id: set.session.session_id,
            set_id: set.set_id,
            rep_count: bson::bson!({ "$lt": BUCKET_SET_REPETITION_LIMIT }),
        }
    }
}

impl IdBucketPattern for bson::Bson {
    fn to_bucket_selector(device_id: Uuid) -> Self {
        let id = format!("^{}_", device_id.sanitize());
        let re = mongodb::bson::Regex {
            pattern: id,
            options: String::new(),
        };

        bson::Bson::RegularExpression(re)
    }
}

impl BucketUpdate for bson::Document {
    fn to_bucket_update(
        field_name: &str,
        value: impl Serialize,
        to_increment_field_name: &str,
        increment_by: i32,
        id_prefix: &str,
    ) -> Self {
        let now = chrono::Utc::now().timestamp_nanos();

        bson::doc! {
            "$push": {
                field_name: bson::to_bson(&value).unwrap(),
            },
            "$inc": { to_increment_field_name: increment_by },
            "$setOnInsert": { "id": format!("{}_{}", id_prefix, now) }
        }
    }
}
