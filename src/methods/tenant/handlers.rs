use open_stock::InternalDb;
use rocket::get;
use rocket::routes;

use crate::guards::Convert;
use crate::Tenant;

pub fn routes() -> Vec<rocket::Route> {
    routes![get]
}

#[get("/<id>")]
pub async fn get(db: InternalDb, id: String) -> Convert<Tenant> {
    Tenant::fetch_by_id(&id, &db.0).await.into()
}
