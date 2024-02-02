#[macro_export]
macro_rules! header {
    ($name: expr, $value: expr) => {
        Header::new($name, $value)
    };
}

#[macro_export]
macro_rules! location {
    ($name: expr, $session_id: expr) => {
        location!($name, $session_id, false)
    };
    ($name: expr, $session_id: expr, $is_chunked: expr) => {
        header!(
            $crate::api::container_spec::LOCATION_HEADER_NAME,
            format!(
                "/v2/{}/blobs/{}uploads/{}",
                $name,
                if $is_chunked == true { "/chunked" } else { "" },
                $session_id
            )
        )
    };
}

#[macro_export]
macro_rules! check_auth {
    ($auth_result: expr, $err_type: ident) => {
        match $auth_result {
            Ok(a) => a,
            Err($crate::api::container_spec::AuthFailure::Unauthorized(resp)) => {
                return $err_type::Unauthorized(resp);
            }
            Err($crate::api::container_spec::AuthFailure::InternalServerError(err)) => {
                error!("Unexpected auth failure {err:?}");
                return $err_type::Failure("An unexpected error occurred");
            }
        }
    };
}
