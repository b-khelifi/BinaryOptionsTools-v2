use std::sync::Arc;
use tokio::runtime::Runtime;
use once_cell::sync::OnceCell;
use napi::bindgen_prelude::*;

static RUNTIME: OnceCell<Arc<Runtime>> = OnceCell::new();

pub(crate) fn get_runtime() -> Result<Arc<Runtime>> {
    let runtime = RUNTIME.get_or_try_init(|| {
        Ok::<_, Error>(Arc::new(Runtime::new().map_err(|err| {
            Error::from_reason(format!("Could not create tokio runtime. {}", err))
        })?))
    })?;
    Ok(runtime.clone())
}