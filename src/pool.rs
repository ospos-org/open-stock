use std::{env, time::Duration};

use async_trait::async_trait;
use chrono::Utc;
use dotenv::dotenv;
use sea_orm::{EntityTrait, DbConn, QuerySelect, ColumnTrait};
use sea_orm_rocket::{rocket::figment::Figment, Database};
use rocket::tokio;
use crate::entities::session;

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

    async fn init(_: &Figment) -> Result<Self, Self::Error> {
        dotenv().ok();

        let database_url = match env::var("DATABASE_URL") {
            Ok(url) => url,
            Err(err) => {
                panic!("Was unable to initialize, could not determine the database url. Reason: {}", err)
            },
        };

        let conn = sea_orm::Database::connect(database_url).await?;

        let c2 = conn.clone();
        tokio::spawn(async move {
            session_garbage_collector(&c2).await;
        });

        Ok(RocketDbPool { conn })
    }

    fn borrow(&self) -> &Self::Connection {
        &self.conn
    }
}

pub async fn session_garbage_collector(db: &DbConn) {
    let mut interval = tokio::time::interval(Duration::from_secs(5));

    loop {
        interval.tick().await;

        match session::Entity::find()
            .having(session::Column::Expiry.lt(Utc::now().naive_utc()))
            .all(db).await {
                Ok(data) => {
                    for model in data {
                        // Delete all model instances of sessions which have surpassed their existence time-frame.
                        match session::Entity::delete_by_id(model.id).exec(db).await {
                            Ok(_data) => {},
                            Err(err) => {
                                println!("[err]: Error in scheduled cron task: {:?}", err)
                            },
                        }
                    }
                },
                Err(err) => {
                    println!("[err]: Error in scheduled cron task: {:?}", err)
                },
        };
    }
}