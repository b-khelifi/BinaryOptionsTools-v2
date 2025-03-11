use napi_derive::napi;

mod pocketoption;

pub use pocketoption::RawPocketOption;


#[napi]
pub fn sum(a: i32, b: i32) -> i32 {
  a + b
}
