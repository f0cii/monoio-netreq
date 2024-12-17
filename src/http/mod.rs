pub mod client;
pub(crate) mod monoio_body;

#[macro_export]
macro_rules! apply_parameter_from_config {
    ($connector:expr, $method:ident($val:expr)) => {
        match $connector {
            HttpConnectorType::HTTP(ref mut c) => c.$method($val),
            HttpConnectorType::HTTPS(ref mut c) => c.$method($val),
        }
    };

    ($connector:expr, $builder:ident().$method:ident($val:expr)) => {
        match $connector {
            HttpConnectorType::HTTP(ref mut c) => c.$builder().$method($val),
            HttpConnectorType::HTTPS(ref mut c) => c.$builder().$method($val),
        }
    };
}