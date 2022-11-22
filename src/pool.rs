use std::env;

use async_trait::async_trait;
use dotenv::dotenv;
use sea_orm_rocket::{rocket::figment::Figment, Config, Database};

#[derive(Database, Debug)]
#[database("stock")]
pub struct Db(RocketDbPool);

#[derive(Debug, Clone)]
pub struct RocketDbPool {
    pub conn: sea_orm::DatabaseConnection,
}

#[async_trait]
impl sea_orm_rocket::Pool for RocketDbPool {
    type Error = sea_orm::DbErr;

    type Connection = sea_orm::DatabaseConnection;

    async fn init(figment: &Figment) -> Result<Self, Self::Error> {
        dotenv().ok();

        let database_url = match env::var("DATABASE_URL") {
            Ok(url) => url,
            Err(err) => {
                panic!("Was unable to initialize, could not determine the database url. Reason: {}", err)
            },
        };

        let conn = sea_orm::Database::connect(database_url).await?;

        Ok(RocketDbPool { conn })
    }

    fn borrow(&self) -> &Self::Connection {
        &self.conn
    }
}