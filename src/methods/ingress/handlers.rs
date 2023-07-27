use crate::{
    check_permissions, cookie_status_wrapper, methods::Action, methods::Error, Db, ErrorResponse,
};
use chrono::Utc;

use rocket::{fs::TempFile, http::CookieJar, post, routes};
use sea_orm_rocket::Connection;

pub fn routes() -> Vec<rocket::Route> {
    routes![upload]
}

/// Usage is as follows
///
/// curl -X POST -H "Content-Type: text/plain" -d "@/to/file/location/" http://127.0.0.1:8000/api/ingress/upload
#[post("/upload", format = "plain", data = "<file>")]
async fn upload(
    conn: Connection<'_, Db>,
    file: TempFile<'_>,
    cookies: &CookieJar<'_>,
) -> Result<(), Error> {
    let db = conn.into_inner();

    // Disable verification in testing;
    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::AccessAdminPanel);

    receive_file(file).await
}

async fn receive_file(mut file: TempFile<'_>) -> Result<(), Error> {
    let current_date = Utc::now().to_rfc3339();
    let path = "/ingress/".to_string();
    if let Err(error) = std::fs::create_dir_all(path.clone()) {
        return Err(ErrorResponse::create_error(&format!(
            "Unable to create file path, {}",
            error
        )));
    }

    match file
        .persist_to(format!("{}/{}.os", path, current_date))
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
