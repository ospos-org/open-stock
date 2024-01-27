use crate::{guards::UserErrorMessage};
use rocket::{serde::json::{json, Value}, Request, catch, Data, form};
use rocket::data::{FromData, Outcome as DataOutcome};
use rocket::form::{DataField, FromForm, ValueField};
use rocket::http::Status;
use rocket::outcome::Outcome;
use rocket::request::FromRequest;
use rocket::serde::json::Json;
use rocket_okapi::gen::OpenApiGenerator;
use rocket_okapi::request::OpenApiFromData;
use schemars::JsonSchema;
use validator::{Validate, ValidationErrors};
use okapi::{
    openapi3::{MediaType, RequestBody},
    Map,
};

/*
    The below code is a mix between json_validator
    and serde handling, in order to handle serde validations

    Credit to a large portion of it is to: owlnext-fr
    https://github.com/owlnext-fr/rust-microservice-skeleton/blob/main/src/core/validation.rs
*/


#[derive(Clone, Debug, JsonSchema)]
pub struct Validated<T>(pub T);

#[derive(Clone)]
pub struct CachedValidationErrors(pub Option<ValidationErrors>);

#[derive(Clone)]
pub struct CachedParseErrors(pub Option<String>);

macro_rules! fn_request_body {
    ($gen:ident, $ty:path, $mime_type:expr) => {{
        let schema = $gen.json_schema::<$ty>();
        Ok(RequestBody {
            content: {
                let mut map = Map::new();
                map.insert(
                    $mime_type.to_owned(),
                    MediaType {
                        schema: Some(schema),
                        ..MediaType::default()
                    },
                );
                map
            },
            required: true,
            ..okapi::openapi3::RequestBody::default()
        })
    }};
}

impl<'r, D: validator::Validate + rocket::serde::Deserialize<'r> + JsonSchema> OpenApiFromData<'r> for Validated<Json<D>> {
    fn request_body(gen: &mut OpenApiGenerator) -> rocket_okapi::Result<RequestBody> {
        fn_request_body!(gen, D, "application/json")
    }
}

#[rocket::async_trait]
impl<'r, D: validator::Validate + rocket::serde::Deserialize<'r> + JsonSchema> FromData<'r> for Validated<Json<D>> {
    type Error = Result<ValidationErrors, rocket::serde::json::Error<'r>>;

    async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> DataOutcome<'r, Self> {
        let data_outcome = <Json<D> as FromData<'r>>::from_data(req, data).await;

        match data_outcome {
            Outcome::Error((status, err)) => {
                req.local_cache(|| CachedParseErrors(Some(err.to_string())));
                Outcome::Error((status, Err(err)))
            }
            Outcome::Forward(err) => Outcome::Forward(err),
            Outcome::Success(data) => match data.validate() {
                Ok(_) => Outcome::Success(Validated(data)),
                Err(err) => {
                    req.local_cache(|| CachedValidationErrors(Some(err.to_owned())));
                    Outcome::Error((Status::BadRequest, Ok(err)))
                }
            },
        }
    }
}

#[rocket::async_trait]
impl<'r, D: Validate + FromRequest<'r>> FromRequest<'r> for Validated<D> {
    type Error = Result<ValidationErrors, D::Error>;
    async fn from_request(req: &'r Request<'_>) -> rocket::request::Outcome<Self, Self::Error> {
        let data_outcome = D::from_request(req).await;

        match data_outcome {
            Outcome::Error((status, err)) => {
                let error_message = format!("{err:?}");
                req.local_cache(|| CachedParseErrors(Some(error_message)));
                Outcome::Error((status, Err(err)))
            }
            Outcome::Forward(err) => Outcome::Forward(err),
            Outcome::Success(data) => match data.validate() {
                Ok(_) => Outcome::Success(Validated(data)),
                Err(err) => {
                    req.local_cache(|| CachedValidationErrors(Some(err.to_owned())));
                    Outcome::Error((Status::BadRequest, Ok(err)))
                }
            },
        }
    }
}


#[rocket::async_trait]
impl<'r, T: Validate + FromForm<'r>> FromForm<'r> for Validated<T> {
    type Context = T::Context;

    #[inline]
    fn init(opts: form::Options) -> Self::Context {
        T::init(opts)
    }

    #[inline]
    fn push_value(ctxt: &mut Self::Context, field: ValueField<'r>) {
        T::push_value(ctxt, field)
    }

    #[inline]
    async fn push_data(ctxt: &mut Self::Context, field: DataField<'r, '_>) {
        T::push_data(ctxt, field).await
    }

    fn finalize(this: Self::Context) -> form::Result<'r, Self> {
        match T::finalize(this) {
            Err(err) => Err(err),
            Ok(data) => match data.validate() {
                Ok(_) => Ok(Validated(data)),
                Err(err) => Err(err
                    .into_errors()
                    .into_iter()
                    .map(|e| form::Error {
                        name: Some(e.0.into()),
                        kind: form::error::ErrorKind::Validation(std::borrow::Cow::Borrowed(e.0)),
                        value: None,
                        entity: form::error::Entity::Value,
                    })
                    .collect::<Vec<_>>()
                    .into()),
            },
        }
    }
}

#[catch(400)]
pub fn general_catcher(req: &Request) -> Value {
    json!([{
        "code": "error.general",
        "message": "Bad Request. The request could not be understood by the server due to malformed syntax.",
        "errors": req.local_cache(|| CachedValidationErrors(None)).0.as_ref(),
    }])
}

#[catch(403)]
pub fn not_authorized() -> Value {
    json!([{"code": "error.unauthorized", "message": "Not authorized to make request"}])
}

#[catch(404)]
pub fn not_found() -> Value {
    json!([{"code": "error.not_found", "message": "The requested route was not found."}])
}

#[catch(422)]
pub fn unprocessable_entry(req: &Request) -> Value {
    let possible_parse_violation = req.local_cache(|| CachedParseErrors(None)).0.as_ref();
    let validation_errors = req.local_cache(|| CachedValidationErrors(None)).0.as_ref();

    let mut message =  "Failed to service request, structure parsing failed.".to_string();

    if validation_errors.is_some() {
        message.clear();

        let erros = validation_errors.unwrap().field_errors();

        for (_,val) in erros.iter() {
            for error in val.iter() {
                message.push_str(error.message.as_ref().unwrap());
            }
        }
    } else if possible_parse_violation.is_some() {
        message.clear();
        message.push_str(possible_parse_violation.unwrap());
    }

    json! [{ "code": "error.input", "message": &message }]
}

#[catch(500)]
pub fn internal_server_error(req: &Request) -> Value {
    let error_message = req
        .local_cache(|| Some(UserErrorMessage("Internal server error".to_owned())));

    json! [{"code": "error.internal", "message": error_message}]
}