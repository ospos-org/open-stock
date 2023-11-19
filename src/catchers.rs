use crate::{guards::UserErrorMessage};
use rocket::{serde::json::{json, Value}, Request, catch};
use crate::guards::JsonValidationError;

#[catch(403)]
pub fn not_authorized() -> Value {
    json!([{"label": "unauthorized", "message": "Not authorized to make request"}])
}

#[catch(404)]
pub fn not_found() -> Value {
    json!([])
}

#[catch(422)]
pub fn unprocessable_entry(req: &Request) -> Value {
    json! [{"label": "failed.request", "message": "failed to service request"}]
}

#[catch(500)]
pub fn internal_server_error(req: &Request) -> Value {
    let error_message = req
        .local_cache(|| Some(UserErrorMessage("Internal server error".to_owned())));

    json! [{"label": "internal.error", "message": error_message}]
}