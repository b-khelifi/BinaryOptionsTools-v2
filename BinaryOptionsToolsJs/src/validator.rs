use binary_options_tools::{
    pocketoption::types::base::RawWebsocketMessage, reimports::ValidatorTrait,
};
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
    Not(BoxedValidator),
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
            Self::Regex(regex) => regex.is_match(&message.to_string()),
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

/// A validator for WebSocket messages that provides various matching strategies.
///
/// # Examples
/// ```javascript
/// const validator = new Validator();
/// const regexValidator = Validator.regex("^Hello");
/// const containsValidator = Validator.contains("World");
///
/// console.log(validator.check("Hello World")); // true
/// cons-.log(regexValidator.check("Hello World")); // true
/// console.log(containsValidator.check("Hello World")); // true
/// ```
#[napi]
#[derive(Clone, Default)]
pub struct Validator {
    inner: RawValidator,
}

#[napi]
impl Validator {
    /// Creates a new empty validator that matches any message.
    ///
    /// # Examples
    /// ```javascript
    /// const validator = new Validator();
    /// console.log(validator.check("any message")); // true
    /// ```
    #[napi(constructor)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new regex validator that matches messages using a regular expression pattern.
    ///
    /// # Arguments
    /// * `pattern` - A string containing a valid regular expression pattern
    ///
    /// # Examples
    /// ```javascript
    /// const validator = Validator.regex("^Hello\\s\\w+");
    /// console.log(validator.check("Hello World")); // true
    /// console.log(validator.check("Hi World")); // false
    /// ```
    #[napi(factory)]
    pub fn regex(pattern: String) -> Result<Self> {
        Ok(Self {
            inner: RawValidator::new_regex(pattern)?,
        })
    }

    /// Creates a new validator that checks if a message contains the specified pattern.
    ///
    /// # Arguments
    /// * `pattern` - The substring to search for in the message
    ///
    /// # Examples
    /// ```javascript
    /// const validator = Validator.contains("World");
    /// console.log(validator.check("Hello World")); // true
    /// console.log(validator.check("Hello")); // false
    /// ```
    #[napi(factory)]
    pub fn contains(pattern: String) -> Self {
        Self {
            inner: RawValidator::new_contains(pattern),
        }
    }

    /// Creates a new validator that checks if a message starts with the specified pattern.
    ///
    /// # Arguments
    /// * `pattern` - The prefix to match at the start of the message
    ///
    /// # Examples
    /// ```javascript
    /// const validator = Validator.starts_with("Hello");
    /// console.log(validator.check("Hello World")); // true
    /// console.log(validator.check("World Hello")); // false
    /// ```
    #[napi(factory)]
    pub fn starts_with(pattern: String) -> Self {
        Self {
            inner: RawValidator::new_starts_with(pattern),
        }
    }

    /// Creates a new validator that checks if a message ends with the specified pattern.
    ///
    /// # Arguments
    /// * `pattern` - The suffix to match at the end of the message
    ///
    /// # Examples
    /// ```javascript
    /// const validator = Validator.ends_with("World");
    /// console.log(validator.check("Hello World")); // true
    /// console.log(validator.check("World Hello")); // false
    /// ```
    #[napi(factory)]
    pub fn ends_with(pattern: String) -> Self {
        Self {
            inner: RawValidator::new_ends_with(pattern),
        }
    }

    /// Creates a new validator that negates the result of another validator.
    ///
    /// # Arguments
    /// * `validator` - The validator whose result should be negated
    ///
    /// # Examples
    /// ```javascript
    /// const contains = Validator.contains("World");
    /// const notContains = Validator.ne(contains);
    /// console.log(notContains.check("Hello Universe")); // true
    /// console.log(notContains.check("Hello World")); // false
    /// ```
    #[napi(factory)]
    pub fn ne(validator: &Validator) -> Self {
        Self {
            inner: RawValidator::new_not(validator.inner.clone()),
        }
    }

    /// Creates a new validator that requires all provided validators to match.
    ///
    /// # Arguments
    /// * `validators` - An array of validators that must all match for this validator to match
    ///
    /// # Examples
    /// ```javascript
    /// const startsHello = Validator.starts_with("Hello");
    /// const endsWorld = Validator.ends_with("World");
    /// const both = Validator.all([startsHello, endsWorld]);
    /// console.log(both.check("Hello Beautiful World")); // true
    /// console.log(both.check("Hello Universe")); // false
    /// ```
    #[napi(factory)]
    pub fn all(validators: Vec<&Validator>) -> Self {
        Self {
            inner: RawValidator::new_all(validators.into_iter().map(|v| v.inner.clone()).collect()),
        }
    }

    /// Creates a new validator that requires at least one of the provided validators to match.
    ///
    /// # Arguments
    /// * `validators` - An array of validators where at least one must match for this validator to match
    ///
    /// # Examples
    /// ```javascript
    /// const containsHello = Validator.contains("Hello");
    /// const containsHi = Validator.contains("Hi");
    /// const either = Validator.any([containsHello, containsHi]);
    /// console.log(either.check("Hello World")); // true
    /// console.log(either.check("Hi there")); // true
    /// console.log(either.check("Hey there")); // false
    /// ```
    #[napi(factory)]
    pub fn any(validators: Vec<&Validator>) -> Self {
        Self {
            inner: RawValidator::new_any(validators.into_iter().map(|v| v.inner.clone()).collect()),
        }
    }

    /// Checks if a message matches this validator's conditions.
    ///
    /// # Arguments
    /// * `msg` - The message string to validate
    ///
    /// # Returns
    /// * `true` if the message matches the validator's conditions
    /// * `false` otherwise
    ///
    /// # Examples
    /// ```javascript
    /// const validator = Validator.contains("World");
    /// console.log(validator.check("Hello World")); // true
    /// console.log(validator.check("Hello Universe")); // false
    /// ```
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
