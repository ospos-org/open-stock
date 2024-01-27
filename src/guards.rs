use rocket::{
    data::{self, Data, FromData, Limits},
    http::Status,
    request::{local_cache, Request},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Copy, Clone, Debug)]
pub struct RequestId(pub Option<Uuid>);

#[derive(Debug)]
pub struct JsonValidation<T>(pub T);

#[derive(Debug)]
pub enum JsonValidationError {
    ParseError(serde_json::Error),
    ReadError
}

#[derive(Serialize, Debug)]
pub struct UserErrorMessage(pub String);

/// A Json Data Guard that runs valiation on the deserialized types via
/// the valiation crate. The validation crate requires the derserialized
/// type have the `Validate` trait.
#[rocket::async_trait]
impl<'r, T> FromData<'r> for JsonValidation<T>
    where
        T: Deserialize<'r>,
{
    type Error = JsonValidationError;

    async fn from_data(
        req: &'r Request<'_>,
        data: Data<'r>,
    ) -> data::Outcome<'r, Self> {
        match data.open(Limits::JSON).into_string().await {
            Ok(value) => {
                let string = local_cache!(req, value.into_inner());

                match serde_json::from_str::<T>(string)
                    .map_err(JsonValidationError::ParseError)
                {
                    Ok(e) =>
                        data::Outcome::Success(JsonValidation(e)),
                    Err(e) =>
                        data::Outcome::Error((Status::InternalServerError, e))
                }
            }
            Err(_) => {
                data::Outcome::Error(
                    (Status::InternalServerError, JsonValidationError::ReadError)
                )
            }
        }
    }
}