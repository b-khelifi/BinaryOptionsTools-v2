use binary_option_tools::{pocketoption::types::base::RawWebsocketMessage, reimports::ValidatorTrait};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use regex::Regex;

use crate::error::BinaryErrorJs;

#[derive(Clone)]
pub struct ArrayRawValidator(Vec<RawValidator>);

#[derive(Clone)]
pub struct BoxedValidator(Box<RawValidator>);

#[derive(Clone)]
enum RawValidator {
    None(),
    Regex(Regex),
    StartsWith(String),
    EndsWith(String),
    Contains(String),
    All(ArrayRawValidator),
    Any(ArrayRawValidator),
    Not(BoxedValidator)
}

impl RawValidator {
    fn new_regex(regex: String) -> Result<Self> {
        let regex = Regex::new(&regex).map_err(BinaryErrorJs::from)?;
        Ok(Self::Regex(regex))
    }

    fn new_all(validators: Vec<RawValidator>) -> Self {
        Self::All(ArrayRawValidator(validators))
    }

    fn new_any(validators: Vec<RawValidator>) -> Self {
        Self::Any(ArrayRawValidator(validators))
    }

    fn new_not(validator: RawValidator) -> Self {
        Self::Not(BoxedValidator(Box::new(validator)))
    }

    fn new_contains(pattern: String) -> Self {
        Self::Contains(pattern)
    }

    fn new_starts_with(pattern: String) -> Self {
        Self::StartsWith(pattern)
    }

    fn new_ends_with(pattern: String) -> Self {
        Self::EndsWith(pattern)
    }
}

impl Default for RawValidator {
    fn default() -> Self {
        Self::None()
    }
}

impl ValidatorTrait<RawWebsocketMessage> for RawValidator {
    fn validate(&self, message: &RawWebsocketMessage) -> bool {
        match self {
            Self::None() => true,
            Self::Contains(pat) => message.to_string().contains(pat),
            Self::StartsWith(pat) => message.to_string().starts_with(pat),
            Self::EndsWith(pat) => message.to_string().ends_with(pat),
            Self::Not(val) => !val.validate(message),
            Self::All(val) => val.validate_all(message),
            Self::Any(val) => val.validate_any(message),
            Self::Regex(regex) => regex.is_match(&message.to_string())
        }
    }
}

impl ArrayRawValidator {
    fn validate_all(&self, message: &RawWebsocketMessage) -> bool {
        self.0.iter().all(|d| d.validate(message))
    }

    fn validate_any(&self, message: &RawWebsocketMessage) -> bool {
        self.0.iter().any(|d| d.validate(message))
    }
}

impl ValidatorTrait<RawWebsocketMessage> for BoxedValidator {
    fn validate(&self, message: &RawWebsocketMessage) -> bool {
        self.0.validate(message)
    }
}

#[napi]
#[derive(Clone, Default)]
pub struct Validator {
    inner: RawValidator
}

#[napi]
impl Validator {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self::default()
    }

    #[napi(factory)]
    pub fn regex(pattern: String) -> Result<Self> {
        Ok(Self {
            inner: RawValidator::new_regex(pattern)?
        })
    }

    #[napi(factory)]
    pub fn contains(pattern: String) -> Self {
        Self {
            inner: RawValidator::new_contains(pattern)
        }
    }

    #[napi(factory)]
    pub fn starts_with(pattern: String) -> Self {
        Self {
            inner: RawValidator::new_starts_with(pattern)
        }
    }

    #[napi(factory)]
    pub fn ends_with(pattern: String) -> Self {
        Self {
            inner: RawValidator::new_ends_with(pattern)
        }
    }

    #[napi(factory)]
    pub fn ne(validator: &Validator) -> Self {
        Self {
            inner: RawValidator::new_not(validator.inner.clone())
        }
    }

    #[napi(factory)]
    pub fn all(validators: Vec<&Validator>) -> Self {
        Self {
            inner: RawValidator::new_all(validators.into_iter().map(|v| v.inner.clone()).collect())
        }
    }

    #[napi(factory)]
    pub fn any(validators: Vec<&Validator>) -> Self {
        Self {
            inner: RawValidator::new_any(validators.into_iter().map(|v| v.inner.clone()).collect())
        }
    }

    #[napi]
    pub fn check(&self, msg: String) -> bool {
        let raw = RawWebsocketMessage::from(msg);
        self.inner.validate(&raw)
    }
}

impl ValidatorTrait<RawWebsocketMessage> for Validator {
    fn validate(&self, message: &RawWebsocketMessage) -> bool {
        self.inner.validate(message)
    }
}

impl Validator {
    pub fn to_val(&self) -> Box<dyn ValidatorTrait<RawWebsocketMessage> + Send + Sync> {
        Box::new(self.inner.clone())
    }
}