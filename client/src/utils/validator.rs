use crate::LightString;
use std::fmt;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;
use yew::prelude::UseStateHandle;

pub trait Validator<T: ?Sized> {
    fn validate(&self, data: &T) -> Option<LightString>;
}

impl<T: ?Sized> fmt::Debug for dyn Validator<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Validator")
    }
}

#[derive(Clone, Debug)]
pub struct ValidatorWrapper<T: ?Sized>(Arc<dyn Validator<T>>);

impl<T: ?Sized> Deref for ValidatorWrapper<T> {
    type Target = dyn Validator<T>;
    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

#[allow(clippy::vtable_address_comparisons)]
impl<T: ?Sized> PartialEq for ValidatorWrapper<T> {
    fn eq(&self, other: &ValidatorWrapper<T>) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Validators<T: ?Sized> {
    pub validators: Vec<ValidatorWrapper<T>>,
}

impl<T: ?Sized> Default for Validators<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ?Sized> Validators<T> {
    pub fn new() -> Self {
        Self {
            validators: Vec::new(),
        }
    }
    pub fn add(mut self, validator: impl Validator<T> + 'static) -> Self {
        self.validators.push(ValidatorWrapper(Arc::new(validator)));
        self
    }
    pub fn validate_into(&self, data: &T, error: &UseStateHandle<Option<LightString>>) {
        error.set(self.validate(data));
    }
}

impl<T: ?Sized> Validator<T> for Validators<T> {
    fn validate(&self, data: &T) -> Option<LightString> {
        self.validators
            .iter()
            .filter_map(|validator| validator.validate(data))
            .next()
    }
}

pub struct RequiredValidator {
    message: LightString,
}

impl RequiredValidator {
    pub fn new(message: impl Into<LightString>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Validator<str> for RequiredValidator {
    fn validate(&self, data: &str) -> Option<LightString> {
        if data.is_empty() {
            Some(self.message.clone())
        } else {
            None
        }
    }
}

impl Validator<String> for RequiredValidator {
    fn validate(&self, data: &String) -> Option<LightString> {
        if data.is_empty() {
            Some(self.message.clone())
        } else {
            None
        }
    }
}

impl Validator<LightString> for RequiredValidator {
    fn validate(&self, data: &LightString) -> Option<LightString> {
        if data.is_empty() {
            Some(self.message.clone())
        } else {
            None
        }
    }
}

impl<T> Validator<Option<T>> for RequiredValidator {
    fn validate(&self, data: &Option<T>) -> Option<LightString> {
        if data.is_none() {
            Some(self.message.clone())
        } else {
            None
        }
    }
}

impl<T> Validator<Vec<T>> for RequiredValidator {
    fn validate(&self, data: &Vec<T>) -> Option<LightString> {
        if data.is_empty() {
            Some(self.message.clone())
        } else {
            None
        }
    }
}

pub struct F64Validator {
    message: LightString,
}

impl F64Validator {
    pub fn new(message: impl Into<LightString>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl<T> Validator<T> for F64Validator
where
    T: AsRef<str>,
{
    fn validate(&self, data: &T) -> Option<LightString> {
        if f64::from_str(data.as_ref()).is_err() {
            Some(self.message.clone())
        } else {
            None
        }
    }
}

pub struct PositiveF64Validator {
    message: LightString,
    equal: bool,
}

impl PositiveF64Validator {
    pub fn new(message: impl Into<LightString>, equal: bool) -> Self {
        Self {
            message: message.into(),
            equal: equal,
        }
    }
}

impl<T> Validator<T> for PositiveF64Validator
where
    T: AsRef<str>,
{
    fn validate(&self, data: &T) -> Option<LightString> {
        match f64::from_str(data.as_ref()) {
            Ok(value) => {
                if 0.0 <= value {
                    if self.equal {
                        None
                    } else {
                        if 0.0 < value {
                            None
                        } else {
                            Some(self.message.clone())
                        }
                    }
                } else {
                    Some(self.message.clone())
                }
            }
            Err(_) => Some(self.message.clone()),
        }
    }
}
