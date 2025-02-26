#[macro_export]
macro_rules! erout {
    ($err:expr) => {
        $err.map_err(|err| {
            ::tracing::error!("{err}");
            err
        })?
    };
}
