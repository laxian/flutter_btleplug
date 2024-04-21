use crate::ble::Error;
use tokio::runtime::Runtime;
use once_cell::sync::OnceCell;

pub static RUNTIME: OnceCell<Runtime> = OnceCell::new();

pub fn create_runtime() -> Result<(), Error> {
    let runtime = {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .on_thread_start(|| {})
            .build()
            .unwrap()
    };
    RUNTIME.set(runtime).map_err(|_| Error::Runtime)?;
    Ok(())
}
