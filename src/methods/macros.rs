#[macro_export]
macro_rules! check_permissions {
    ($session:expr, $permission:expr) => {
        if !$session.has_permission($permission) {
            return Err($crate::methods::ErrorResponse::custom_unauthorized(
                "User is unauthorized, may not have a valid session.",
            ));
        }
    };
}
