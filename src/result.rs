use std::num::NonZeroU32;
use std::result::Result as StdResult;
use thiserror::Error;

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

fn error_to_string(error: u32) -> &'static str {
    ERROR_CODES.get(&error).copied().unwrap_or("Unknown error")
}

#[derive(Error, Debug)]
pub enum LibnxError {
    #[error("{} ({0:x})", error_to_string(.0.get()))]
    System(NonZeroU32),
}

pub type Result<T> = StdResult<T, LibnxError>;

pub trait IntoResult<T> {
    fn into_result(self) -> Result<T>;
}

impl IntoResult<()> for u32 {
    fn into_result(self) -> Result<()> {
        if let Some(error) = NonZeroU32::new(self) {
            Err(LibnxError::System(error))
        } else {
            Ok(())
        }
    }
}
