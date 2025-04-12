use std::sync::Arc;

use pyo3::{
    pyclass, pymethods,
    types::{PyAnyMethods, PyList},
    Bound, PyObject, PyResult, Python,
};
use regex::Regex;

use crate::error::BinaryResultPy;
use binary_options_tools::{
    pocketoption::types::base::RawWebsocketMessage, reimports::ValidatorTrait,
};

#[pyclass]
#[derive(Clone)]
pub struct ArrayValidator(Vec<RawValidator>);

#[pyclass]
#[derive(Clone)]
pub struct BoxedValidator(Box<RawValidator>);

#[pyclass]
#[derive(Clone)]
pub struct RegexValidator {
    regex: Regex,
}

#[pyclass]
#[derive(Clone)]
pub struct PyCustom {
    custom: Arc<PyObject>,
}

#[pyclass]
#[derive(Clone)]
pub enum RawValidator {
    None(),
    Regex(RegexValidator),
    StartsWith(String),
    EndsWith(String),
    Contains(String),
    All(ArrayValidator),
    Any(ArrayValidator),
    Not(BoxedValidator),
    Custom(PyCustom),
}

impl RawValidator {
    pub fn new_regex(regex: String) -> BinaryResultPy<Self> {
        let regex = Regex::new(&regex)?;
        Ok(Self::Regex(RegexValidator { regex }))
    }

    pub fn new_all(validators: Vec<RawValidator>) -> Self {
        Self::All(ArrayValidator(validators))
    }

    pub fn new_any(validators: Vec<RawValidator>) -> Self {
        Self::Any(ArrayValidator(validators))
    }

    pub fn new_not(validator: RawValidator) -> Self {
        Self::Not(BoxedValidator(Box::new(validator)))
    }

    pub fn new_contains(pattern: String) -> Self {
        Self::Contains(pattern)
    }

    pub fn new_starts_with(pattern: String) -> Self {
        Self::StartsWith(pattern)
    }

    pub fn new_ends_with(pattern: String) -> Self {
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
            Self::Regex(val) => val.validate(message),
            Self::Custom(val) => val.validate(message),
        }
    }
}

impl ValidatorTrait<RawWebsocketMessage> for PyCustom {
    fn validate(&self, message: &RawWebsocketMessage) -> bool {
        Python::with_gil(|py| {
            let res = self
                .custom
                .call(py, (message.to_string(),), None)
                .expect("Expected provided function to be callable");
            res.extract(py)
                .expect("Expected provided function to return a boolean")
        })
    }
}

impl ArrayValidator {
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

impl ValidatorTrait<RawWebsocketMessage> for RegexValidator {
    fn validate(&self, message: &RawWebsocketMessage) -> bool {
        self.regex.is_match(&message.to_string())
    }
}

#[pymethods]
impl RawValidator {
    #[new]
    pub fn new() -> Self {
        Self::default()
    }

    #[staticmethod]
    pub fn regex(pattern: String) -> PyResult<Self> {
        Ok(Self::new_regex(pattern)?)
    }

    #[staticmethod]
    pub fn contains(pattern: String) -> Self {
        Self::new_contains(pattern)
    }

    #[staticmethod]
    pub fn starts_with(pattern: String) -> Self {
        Self::new_starts_with(pattern)
    }

    #[staticmethod]
    pub fn ends_with(pattern: String) -> Self {
        Self::new_ends_with(pattern)
    }

    #[staticmethod]
    pub fn ne(validator: Bound<'_, RawValidator>) -> Self {
        let val = validator.get();
        Self::new_not(val.clone())
    }

    #[staticmethod]
    pub fn all(validator: Bound<'_, PyList>) -> PyResult<Self> {
        let val = validator.extract::<Vec<RawValidator>>()?;
        Ok(Self::new_all(val))
    }

    #[staticmethod]
    pub fn any(validator: Bound<'_, PyList>) -> PyResult<Self> {
        let val = validator.extract::<Vec<RawValidator>>()?;
        Ok(Self::new_any(val))
    }

    #[staticmethod]
    pub fn custom(func: PyObject) -> Self {
        Self::Custom(PyCustom {
            custom: Arc::new(func),
        })
    }

    pub fn check(&self, msg: String) -> bool {
        let raw = RawWebsocketMessage::from(msg);
        self.validate(&raw)
    }
}
