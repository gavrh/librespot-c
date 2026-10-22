use std::sync::OnceLock;
use tokio::runtime::Runtime;

static RUNTIME: OnceLock<Runtime> = OnceLock::new();

pub(crate) fn try_runtime() -> std::io::Result<&'static Runtime> {
    if let Some(runtime) = RUNTIME.get() {
        return Ok(runtime);
    }

    let runtime = Runtime::new()?;
    let _ = RUNTIME.set(runtime);

    RUNTIME.get().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::Other, "runtime was not initialized")
    })
}

pub fn runtime() -> &'static Runtime {
    try_runtime().expect("failed to create Tokio runtime")
}
