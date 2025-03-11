use napi_derive::napi;

mod pocketoption;
mod runtime;
mod error;

pub use pocketoption::RawPocketOption;


#[napi]
pub fn sum(a: i32, b: i32) -> i32 {
  a + b
}
