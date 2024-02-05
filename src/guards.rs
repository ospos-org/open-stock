use futures::TryStreamExt;
use rocket::{
    data::{self, Data, FromData, Limits},
    http::Status,
    request::{local_cache, Request},
};
use rocket::request::{FromRequest, Outcome};
use rocket_okapi::request::{OpenApiFromRequest, RequestHeaderInput};
use rocket_db_pools::Connection;
use rocket_okapi::gen::OpenApiGenerator;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::methods::common::Error;
use crate::{cookie_status_wrapper, Db, ErrorResponse, Session};

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

pub struct SessionReference(pub Session);

impl<'r> OpenApiFromRequest<'r> for SessionReference {
    fn from_request_input(
        _gen: &mut OpenApiGenerator,
        _name: String,
        _required: bool,
    ) -> rocket_okapi::Result<RequestHeaderInput> {
        Ok(RequestHeaderInput::None)
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for SessionReference { // &'r
    type Error = Error;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let cookies = request.cookies();

        let db = match request.guard::<Connection<Db>>().await {
            Outcome::Success(s) => s,
            Outcome::Error(e) => {
                let err =  match e.1 {
                    Some(v) => ErrorResponse::db_err(v),
                    None => ErrorResponse::create_error("")
                };

                return Outcome::Error((e.0, err))
            },
            Outcome::Forward(f) => return Outcome::Forward(f)
        };

        match cookie_status_wrapper(&db, cookies).await {
            Ok(session) => Outcome::Success(SessionReference(session)),
            Err(error) => Outcome::Forward(Status::Unauthorized)
        }

    }
}