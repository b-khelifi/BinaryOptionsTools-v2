use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};


#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateStream {
    active: String,
    #[serde(with = "FloatTime")]
    time: DateTime<Utc>,
    price: f64
}

#[derive(Debug, Deserialize)]
pub struct UpdateStreamItem {
    active: String,
    #[serde(with = "FloatTime")]
    time: DateTime<Utc>,
    price: f64
}


mod FloatTime {
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(
        date: &DateTime<Utc>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = date.timestamp_millis() as f64 / 1000.0;
        serializer.serialize_f64(s)
    }

    // The signature of a deserialize_with function must follow the pattern:
    //
    //    fn deserialize<'de, D>(D) -> Result<T, D::Error>
    //    where
    //        D: Deserializer<'de>
    //
    // although it may also be generic over the output types T.
    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = f64::deserialize(deserializer)?.to_string();
        dbg!(&s);
        let (secs, milis) = match s.split_once(".") {
            Some((seconds, miliseconds)) => {
                let secs: i64 = seconds.parse::<i64>().map_err(|e| serde::de::Error::custom(e.to_string()))?;
                let mut pow = 0;
                if miliseconds.len() <= 9 {
                    pow = 9u32.saturating_sub(miliseconds.len() as u32);
                } 
                let milis = miliseconds.parse::<u32>().map_err(|e| serde::de::Error::custom(e.to_string()))? * 10i32.pow(pow) as u32;
                (secs, milis)
            },
            None => {
                let secs: i64 = s.parse::<i64>().map_err(|e| serde::de::Error::custom(&e.to_string()))?;

                (secs, 0)
            }
        };
        DateTime::from_timestamp(secs, milis).ok_or(serde::de::Error::custom("Error parsing ints to time"))
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    use std::error::Error;

    #[test]
    fn test_descerialize_update_stream() -> Result<(), Box<dyn Error>> {
        let tests = [
            r#"["AUS200_otc",1732830010,6436.06]"#,
            r#"["AUS200_otc",1732830108.205,6435.96]"#,
            r#"["AEDCNY_otc",1732829668.352,1.89817]"#,
            r#"["CADJPY_otc",1732830170.793,109.442]"#,
        ];
        for item in tests.iter() {
            let res: UpdateStream = serde_json::from_str(item)?;
            let time_reparsed = serde_json::to_string(&res)?;
            dbg!(time_reparsed);
            dbg!(res);
        }        
        Ok(())
    }
}