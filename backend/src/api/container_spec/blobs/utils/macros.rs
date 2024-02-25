#[macro_export]
macro_rules! header {
    ($name: expr, $value: expr) => {
        Header::new($name, $value)
    };
}

#[macro_export]
macro_rules! location {
    ($name: expr, $session_id: expr) => {
        header!(
            $crate::api::container_spec::LOCATION_HEADER_NAME,
            format!("/v2/{}/blobs/uploads/{}", $name, $session_id)
        )
    };
}

#[macro_export]
macro_rules! range {
    ($session: expr) => {
        header!(
            $crate::api::container_spec::RANGE_HEADER_NAME,
            format!("0-{}", $session.starting_byte_index - 1)
        )
    };
}
