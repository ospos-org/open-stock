use chrono::Utc;
use rocket::{fs::TempFile, post, routes};

use crate::{methods::Error, ErrorResponse};

pub fn routes() -> Vec<rocket::Route> {
    routes![upload]
}

#[post("/upload", format = "plain", data = "<file>")]
async fn upload(mut file: TempFile<'_>) -> Result<(), Error> {
    let current_date = Utc::now().to_rfc3339();
    let path = format!("/ingress/{current_date}");
    if let Err(error) = std::fs::create_dir_all(path.clone()) {
        return Err(ErrorResponse::create_error(&format!(
            "Unable to create file path, {}",
            error
        )));
    }

    match file.persist_to(path).await {
        Ok(_) => {
            //... now spawn worker to ingress the DB
            Ok(())
        }
        Err(error) => Err(ErrorResponse::create_error(&format!(
            "Unable to write file, {}",
            error,
        ))),
    }
}
