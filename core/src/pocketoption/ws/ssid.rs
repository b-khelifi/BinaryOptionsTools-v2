use core::fmt;
use std::{error::Error, str::Bytes};

use serde::{de::Error as SerdeError, Deserialize, Serialize};
use serde_json::{Deserializer, Value};

use crate::pocketoption::error::PocketOptionError;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Demo {
    session: String,
    is_demo: u32,
    uid: u32,
    platform: u32
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Real {
    session: String,
    is_demo: u32,
    uid: u32,
    platform: u32,
    raw: String
}


#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum Ssid {
    Demo(Demo),
    Real(Real)
}


impl Ssid {
    pub fn parse(data: impl ToString) -> Result<Self, PocketOptionError> {
        let data = data.to_string();
        let parsed = data.trim().strip_prefix(r#"42["auth","#).ok_or(PocketOptionError::SsidParsingError("Error parsing ssid string into object".into()))?.strip_suffix("]").ok_or(PocketOptionError::SsidParsingError("Error parsing ssid string into object".into()))?;
        let ssid: Demo = serde_json::from_str(parsed).map_err(|e| PocketOptionError::SsidParsingError(e.to_string()))?;
        if ssid.is_demo == 1 {
            Ok(Self::Demo(ssid))
        } else {
            let real = Real {
                raw: data,
                is_demo: ssid.is_demo,
                session: ssid.session,
                uid: ssid.uid,
                platform: ssid.platform
            };
            Ok(Self::Real(real))
        }
    }
}
impl fmt::Display for Demo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ssid = serde_json::to_string(&self).map_err(|_| fmt::Error)?;
        write!(f, r#"42["auth",{}]"#, ssid)
    }
}

impl fmt::Display for Real {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.raw)
    }
}

impl fmt::Display for Ssid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Demo(demo) => demo.fmt(f),
            Self::Real(real) => real.fmt(f),
        }
    }
}