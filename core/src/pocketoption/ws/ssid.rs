use core::fmt;
use std::ops::Deref;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    general::traits::Credentials,
    pocketoption::error::{PocketOptionError, PocketResult},
};

use super::regions::Regions;

#[derive(Clone)]
pub struct PocketCreds {
    pub ssid: Ssid,
    pub demo: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionData {
    session_id: String,
    ip_address: String,
    user_agent: String,
    last_activity: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Demo {
    session: String,
    is_demo: u32,
    uid: u32,
    platform: u32,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Real {
    session: SessionData,
    is_demo: u32,
    uid: u32,
    platform: u32,
    raw: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(untagged)]
pub enum Ssid {
    Demo(Demo),
    Real(Real),
}

impl Ssid {
    pub fn parse(data: impl ToString) -> PocketResult<Self> {
        let data = data.to_string();
        let parsed = data
            .trim()
            .strip_prefix(r#"42["auth","#)
            .ok_or(PocketOptionError::SsidParsingError(
                "Error parsing ssid string into object".into(),
            ))?
            .strip_suffix("]")
            .ok_or(PocketOptionError::SsidParsingError(
                "Error parsing ssid string into object".into(),
            ))?;
        let ssid: Demo = serde_json::from_str(parsed)
            .map_err(|e| PocketOptionError::SsidParsingError(e.to_string()))?;
        if ssid.is_demo == 1 {
            Ok(Self::Demo(ssid))
        } else {
            let real = Real {
                raw: data,
                is_demo: ssid.is_demo,
                session: php_serde::from_bytes(ssid.session.as_bytes()).map_err(|e| {
                    PocketOptionError::SsidParsingError(format!("Error parsing session data, {e}"))
                })?,
                uid: ssid.uid,
                platform: ssid.platform,
            };
            Ok(Self::Real(real))
        }
    }

    pub async fn server(&self) -> PocketResult<String> {
        match self {
            Self::Demo(_) => Ok(Regions::DEMO.to_string()),
            Self::Real(_) => Regions.get_server().await.map(|s| s.to_string()),
        }
    }

    pub async fn servers(&self) -> PocketResult<Vec<String>> {
        match self {
            Self::Demo(_) => Ok(vec![Regions::DEMO.to_string()]),
            Self::Real(_) => Ok(Regions
                .get_servers()
                .await?
                .iter()
                .map(|s| s.to_string())
                .collect()),
        }
    }

    pub fn user_agent(&self) -> String {
        match self {
            Self::Demo(_) => "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36".into(),
            Self::Real(real) => real.session.user_agent.clone()
        }
    }

    pub fn demo(&self) -> bool {
        match self {
            Self::Demo(_) => true,
            Self::Real(_) => false,
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

impl<'de> Deserialize<'de> for Ssid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let data: Value = Value::deserialize(deserializer)?;
        Ssid::parse(data).map_err(serde::de::Error::custom)
    }
}

impl Credentials for PocketCreds {}

impl Deref for PocketCreds {
    type Target = Ssid;

    fn deref(&self) -> &Self::Target {
        &self.ssid
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_descerialize_session() -> Result<(), Box<dyn Error>> {
        let session_raw = b"a:4:{s:10:\"session_id\";s:32:\"ae3aa847add89c341ec18d8ae5bf8527\";s:10:\"ip_address\";s:15:\"191.113.157.139\";s:10:\"user_agent\";s:120:\"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36 OPR/114.\";s:13:\"last_activity\";i:1732926685;}31666d2dc07fdd866353937b97901e2b";
        let session: SessionData = php_serde::from_bytes(session_raw)?;
        dbg!(&session);
        let session_php = php_serde::to_vec(&session)?;
        dbg!(String::from_utf8(session_php).unwrap());
        Ok(())
    }

    #[test]
    fn test_parse_ssid() -> Result<(), Box<dyn Error>> {
        let ssids = [
            r#"42["auth",{"session":"looc69ct294h546o368s0lct7d","isDemo":1,"uid":87742848,"platform":2}]	"#,
            r#"42["auth",{"session":"a:4:{s:10:\"session_id\";s:32:\"ae3aa847add89c341ec18d8ae5bf8527\";s:10:\"ip_address\";s:15:\"191.113.157.139\";s:10:\"user_agent\";s:120:\"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36 OPR/114.\";s:13:\"last_activity\";i:1732926685;}31666d2dc07fdd866353937b97901e2b","isDemo":0,"uid":87742848,"platform":2}]	"#,
            r#"42["auth",{"session":"vtftn12e6f5f5008moitsd6skl","isDemo":1,"uid":27658142,"platform":2}]"#,
        ];
        for ssid in ssids {
            let valid = Ssid::parse(ssid)?;
            dbg!(valid);
        }
        Ok(())
    }
}
