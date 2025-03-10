use super::error;
use std::fmt::Display;

pub trait ResultExt {
    fn display_error(&self);
}

impl<T, E> ResultExt for Result<T, E>
where
    E: Display,
{
    fn display_error(&self) {
        match self {
            Result::Err(err) => {
                error(err.to_string().into());
            }
            Result::Ok(_) => (),
        }
    }
}
