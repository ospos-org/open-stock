use crate::{methods::Error, ErrorResponse};
use chrono::Utc;

use rocket::{fs::TempFile, post, routes};

pub fn routes() -> Vec<rocket::Route> {
    routes![upload]
}

/// Usage is as follows
///
/// curl -X POST -H "Content-Type: text/plain" -d "@/to/file/location/" http://127.0.0.1:8000/api/ingress/upload
#[post("/upload", format = "plain", data = "<file>")]
async fn upload(file: TempFile<'_>) -> Result<(), Error> {
    receive_file(file).await
}

async fn receive_file(mut file: TempFile<'_>) -> Result<(), Error> {
    let current_date = Utc::now().to_rfc3339();
    let path = format!("/ingress/{current_date}");
    if let Err(error) = std::fs::create_dir_all(path.clone()) {
        return Err(ErrorResponse::create_error(&format!(
            "Unable to create file path, {}",
            error
        )));
    }

    match file.persist_to(format!("{}/processable.os", path)).await {
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
