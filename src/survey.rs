use chrono::{offset::Utc, DateTime};
use serde::{Deserialize, Serialize};

use crate::LiftrightError;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Survey {
    pub submitted: Option<DateTime<Utc>>,
    pub survey_data: Vec<SurveyData>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SurveyData {
    pub question: String,
    pub answer: String,
}

pub fn submit(_collection: mongodb::Collection, _data: Survey) -> Result<usize, LiftrightError> {
    Err(LiftrightError::UnimplementedError)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn deserialize_survey() {
        assert!(serde_json::from_str::<Survey>(&make_valid_survey_json()).is_ok());
    }

    #[test]
    fn error_on_missing_device_id() {
        let raw_json = format!(
            "
            {{
                \"device_id\": \"\",
                \"survey_data\": {}
            }}
            ",
            make_survey_data()
        );

        assert!(serde_json::from_str::<Survey>(&raw_json).is_err())
    }

    #[test]
    #[ignore]
    fn error_on_missing_survey() {
        let raw_json = format!(
            "
            {{
                \"device_id\": \"{}\",
                \"survey_data\": \"\"
            }}
            ",
            Uuid::new_v4()
        );

        assert!(serde_json::from_str::<Survey>(&raw_json).is_err())
    }

    fn make_valid_survey_json() -> String {
        format!(
            "
            {{
                \"device_id\": \"{}\",
                \"survey_data\": {}  
            }}
            ",
            Uuid::new_v4().to_string(),
            make_survey_data()
        )
    }

    fn make_survey_data() -> String {
        let mut survey_data = HashMap::<String, Option<String>>::new();
        survey_data.insert(
            String::from("Was the game fun?"),
            Some(String::from("Very")),
        );
        survey_data.insert(
            String::from("Was the sensor/arm band comfortable?"),
            Some(String::from("Somewhat")),
        );
        survey_data.insert(
            String::from("What Metrics do you find useful to guage your performance over time?"),
            Some(String::from("I 💪🏻 my 🍆 and 🍑 into a 🐉")),
        );

        serde_json::to_string(&survey_data).unwrap()
    }
}
