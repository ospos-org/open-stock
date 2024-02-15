use crate::methods::common::Error;
use crate::{cookie_status_wrapper, Db, ErrorResponse, Session};
use futures::TryStreamExt;
use okapi::openapi3::{MediaType, RefOr, Response, Responses};
use rocket::request::{FromRequest, Outcome};
use rocket::response::Responder;
use rocket::serde::json::Json;
use rocket::{
    data::{self, Data, FromData, Limits},
    http::Status,
    request::{local_cache, Request},
    response,
};
use rocket::form::{FromForm, Options};
use rocket_db_pools::Connection;
use rocket_okapi::gen::OpenApiGenerator;
use rocket_okapi::request::{OpenApiFromRequest, RequestHeaderInput};
use rocket_okapi::response::OpenApiResponderInner;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Copy, Clone, Debug)]
pub struct RequestId(pub Option<Uuid>);

#[derive(Debug)]
pub struct JsonValidation<T>(pub T);

#[derive(Debug)]
pub enum JsonValidationError {
    ParseError(serde_json::Error),
    ReadError,
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

    async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> data::Outcome<'r, Self> {
        match data.open(Limits::JSON).into_string().await {
            Ok(value) => {
                let string = local_cache!(req, value.into_inner());

                match serde_json::from_str::<T>(string).map_err(JsonValidationError::ParseError) {
                    Ok(e) => data::Outcome::Success(JsonValidation(e)),
                    Err(e) => data::Outcome::Error((Status::InternalServerError, e)),
                }
            }
            Err(_) => {
                data::Outcome::Error((Status::InternalServerError, JsonValidationError::ReadError))
            }
        }
    }
}

impl<'r> OpenApiFromRequest<'r> for Session {
    fn from_request_input(
        _gen: &mut OpenApiGenerator,
        _name: String,
        _required: bool,
    ) -> rocket_okapi::Result<RequestHeaderInput> {
        Ok(RequestHeaderInput::None)
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Session {
    // &'r
    type Error = Error;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let cookies = request.cookies();

        let db = match request.guard::<Connection<Db>>().await {
            Outcome::Success(s) => s,
            Outcome::Error(e) => {
                let err = match e.1 {
                    Some(v) => ErrorResponse::db_err(v),
                    None => ErrorResponse::create_error(""),
                };

                return Outcome::Error((e.0, err));
            }
            Outcome::Forward(f) => return Outcome::Forward(f),
        };

        match cookie_status_wrapper(&db, cookies).await {
            Ok(session) => Outcome::Success(session),
            Err(_) => Outcome::Forward(Status::Unauthorized),
        }
    }
}

pub struct Convert<T>(pub Result<Json<T>, Error>);

impl<T> From<Result<T, Error>> for Convert<T> {
    fn from(value: Result<T, Error>) -> Self {
        Convert(value.map(|v| Json(v)))
    }
}

impl<K: Serialize + JsonSchema> OpenApiResponderInner for Convert<K> {
    fn responses(gen: &mut OpenApiGenerator) -> rocket_okapi::Result<Responses> {
        let mut responses = Responses::default();

        // Define response for successful response
        let success_schema = gen.json_schema::<K>();
        let success_response = Response {
            description: "Success".to_string(),
            content:
                vec![("application/json".to_string(), MediaType {
                    schema: Some(success_schema),
                    ..Default::default()
                })].into_iter().collect(),

            ..Default::default()
        };

        // Define response for internal server error
        let internal_error_response = Response {
            description: "Internal Server Error".to_string(),
            ..Default::default()
        };

        // Insert responses into the map
        responses.responses.insert("200".to_string(), RefOr::from(success_response));
        responses.responses.insert("500".to_string(), RefOr::from(internal_error_response));

        Ok(responses)
    }
}

impl<'r, 'o: 'r, K: Serialize> Responder<'r, 'o> for Convert<K> {
    fn respond_to(self, r: &'r Request<'_>) -> response::Result<'o>
    where
        K: Serialize,
    {
        Responder::respond_to(self.0, r)
    }
}

impl<'de, T: Deserialize<'de>> Into<Result<Json<T>, Error>> for Convert<T> {
    fn into(self) -> Result<Json<T>, Error> {
        match self.0 {
            Ok(v) => Ok(v.into()),
            Err(e) => Err(e.into()),
        }
    }
}
