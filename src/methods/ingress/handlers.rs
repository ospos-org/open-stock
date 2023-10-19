use crate::{
    check_permissions, cookie_status_wrapper, methods::Action, methods::Error, Db, ErrorResponse,
};
use chrono::Utc;
use okapi::openapi3::OpenApi;

use rocket::{fs::TempFile, http::CookieJar, post, routes};
use rocket_db_pools::Connection;
use rocket_okapi::{openapi, openapi_get_routes_spec};
use rocket_okapi::settings::OpenApiSettings;

pub fn documented_routes(settings: &OpenApiSettings) -> (Vec<rocket::Route>, OpenApi) {
    openapi_get_routes_spec![settings: upload]
}

/// Usage is as follows
///
/// curl -X POST -H "Content-Type: text/plain" -d "@/to/file/location/" http://127.0.0.1:8000/api/ingress/upload
#[openapi(tag = "Ingress")]
#[post("/upload", format = "plain", data = "<file>")]
async fn upload(
    conn: Connection<Db>,
    file: TempFile<'_>,
    cookies: &CookieJar<'_>,
) -> Result<(), Error> {
    let db = conn.into_inner();

    // Disable verification in testing;
    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::AccessAdminPanel);

    receive_file(file, session.tenant_id).await
}

async fn receive_file(mut file: TempFile<'_>, tenant_id: String) -> Result<(), Error> {
    let current_date = Utc::now().to_rfc3339();
    let path = "/ingress/".to_string();
    if let Err(error) = std::fs::create_dir_all(path.clone()) {
        return Err(ErrorResponse::create_error(&format!(
            "Unable to create file path, {}",
            error
        )));
    }

    match file
        // We must use `copy_to` due to:
        // https://github.com/SergioBenitez/Rocket/issues/1600
        // Where a cross-device link is made using `link`, for the persistence
        // of the file to a new location, which occurs cross-mount and thus
        // will not work.
        .copy_to(format!("{}/{}_{}.os", path, tenant_id, current_date))
        .await
    {
        Ok(_) => {
            //... Now we let ingress worker take over
            Ok(())
        }
        Err(error) => Err(ErrorResponse::create_error(&format!(
            "Unable to write file, {}",
            error,
        ))),
    }
}
