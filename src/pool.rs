use std::{env, fs, sync::Arc, time::Duration};

#[cfg(feature = "process")]
use crate::entities::{session, transactions};
#[cfg(feature = "process")]
use crate::migrator::Migrator;
use crate::{example_employee, Customer, Product, Session, Store, Transaction, Kiosk};
#[cfg(feature = "process")]
use async_trait::async_trait;
use chrono::{Days, Duration as ChronoDuration, Utc};
#[cfg(feature = "process")]
use dotenv::dotenv;
use rocket::request;
#[cfg(feature = "process")]
use rocket::tokio;
use rocket_okapi::gen::OpenApiGenerator;
use rocket_okapi::request::{OpenApiFromRequest, RequestHeaderInput};
#[cfg(feature = "process")]
use sea_orm::{ColumnTrait, ConnectOptions, DbConn, EntityTrait, QuerySelect};
#[cfg(feature = "process")]
use sea_orm_migration::prelude::*;
#[cfg(feature = "process")]
use sea_orm_rocket::{rocket::figment::Figment};
use rocket_db_pools::Database;
#[cfg(feature = "process")]
use tokio::sync::Mutex;
use rocket::request::FromRequest;
use sea_orm::DatabaseConnection;

#[cfg(feature = "process")]
#[derive(Database, Debug)]
#[database("stock")]
pub struct Db(RocketDbPool);

#[rocket::async_trait]
impl<'a> FromRequest<'a> for RocketDbPool {
    type Error = &'static str;
    async fn from_request(
        _request: &'a request::Request<'_>,
    ) -> request::Outcome<Self, Self::Error> {
        request::Outcome::Success(RocketDbPool { conn: DatabaseConnection::Disconnected })
    }
}

#[rocket::async_trait]
impl<'a> FromRequest<'a> for Db {
    type Error = &'static str;
    async fn from_request(
        _request: &'a request::Request<'_>,
    ) -> request::Outcome<Self, Self::Error> {
        request::Outcome::Success(Db(RocketDbPool { conn: DatabaseConnection::Disconnected }))
    }
}

impl<'r> OpenApiFromRequest<'r> for Db {
     fn from_request_input(
         _gen: &mut OpenApiGenerator,
         _name: String,
        _required: bool,
     ) -> rocket_okapi::Result<RequestHeaderInput> {
         Ok(RequestHeaderInput::None)
     }
}

#[cfg(feature = "process")]
#[derive(Debug, Clone, OpenApiFromRequest)]
pub struct RocketDbPool {
    pub conn: DatabaseConnection,
}

#[cfg(feature = "process")]
#[async_trait]
impl rocket_db_pools::Pool for RocketDbPool {
    type Connection = DatabaseConnection;

    type Error = DbErr;

    async fn init(_: &Figment) -> Result<Self, Self::Error> {
        dotenv().ok();

        let database_url = match env::var("DATABASE_URL") {
            Ok(url) => url,
            Err(err) => {
                panic!(
                    "Was unable to initialize, could not determine the database url. Reason: {}",
                    err
                )
            }
        };

        println!("Database URL: {}", database_url);

        let mut options = ConnectOptions::new(database_url);
        options.idle_timeout(Duration::new(3600, 0));
        options.acquire_timeout(Duration::new(3600, 0));
        options.connect_timeout(Duration::new(3600, 0));
        options.min_connections(1);

        let conn = sea_orm::Database::connect(options).await?;

        // Perform all migrations to the DB
        Migrator::up(&conn, None).await?;

        let c2 = conn.clone();
        tokio::spawn(async move {
            session_garbage_collector(&c2).await;
        });

        let c3 = conn.clone();
        tokio::spawn(async move {
            session_ingress_worker(&c3).await;
        });

        Ok(RocketDbPool { conn })
    }

    async fn get(&self) -> Result<Self::Connection, Self::Error> {
        Ok(self.conn.clone())
    }

    async fn close(&self) {
        self.conn.to_owned().close().await.unwrap();
    }
}

#[cfg(feature = "process")]
pub async fn session_ingress_worker(db: &DbConn) {
    let currently_ingesting = Arc::new(Mutex::new(vec![]));
    let mut interval = tokio::time::interval(Duration::from_secs(5));

    loop {
        interval.tick().await;

        if let Ok(dir) = fs::read_dir("/ingress/") {
            let found_files = dir
                .map(|directory| directory.unwrap().path().to_str().unwrap().to_string())
                .collect::<Vec<String>>();

            let loop_ingest_clone = currently_ingesting.clone();

            // Find discontinuity between the files in the directory and those being ingested.
            for file in found_files {
                if !loop_ingest_clone.lock().await.contains(&file) {
                    // We need to start an ingest for it.
                    loop_ingest_clone.lock().await.push(file.clone());

                    let cloned_ingest = loop_ingest_clone.clone();
                    let cloned_connection = db.clone();

                    // Spawn worker on another thread.
                    tokio::spawn(async move {
                        ingest_file(&cloned_connection, file.clone()).await;
                        match fs::remove_file(file.clone()) {
                            Ok(_) => cloned_ingest.lock().await.retain(|x| *x != file),
                            Err(error) => {
                                // As we don't want to infinitely ingest the file,
                                // if it cannot be deleted we shall preserve it as
                                // continually being in the "currently_ingesting" state.
                                println!("Failed to remove file after ingest; {}", error)
                            }
                        }
                    });
                }
            }
        }
    }
}

#[cfg(feature = "process")]
pub async fn ingest_file(db: &DbConn, file_path: String) {
    // Read in the file to memory, hoping the memory is sufficient to do so.
    let to_ingest = fs::read_to_string(file_path.clone());

    if let Err(error) = to_ingest {
        println!("Failed to ingest file, {}", error);
        return;
    }

    let objectified: (Vec<Product>, Vec<Customer>, Vec<Transaction>, Vec<Store>, Vec<Kiosk>) =
        serde_json::from_str(&to_ingest.unwrap()).unwrap();

    let file_ending = file_path.split('/').last();

    if file_ending.is_none() {
        return;
    }

    let (tenant_id, _date_saved) = file_ending.unwrap().split_once('_').unwrap();

    let default_employee = example_employee();

    let session = Session {
        id: String::new(),
        key: String::new(),
        employee: default_employee.into(),
        expiry: Utc::now().checked_add_days(Days::new(1)).unwrap(),
        tenant_id: tenant_id.to_string().clone(),
    };

    for store in objectified.3 {
        let _ = Store::insert(store, session.clone(), db).await;
    }

    for product in objectified.0 {
        let _ = Product::insert(product, session.clone(), db).await;
    }

    for customer in objectified.1 {
        let _ = Customer::insert_raw(customer, session.clone(), db).await;
    }

    for transaction in objectified.2 {
        let _ = Transaction::insert_raw(transaction, session.clone(), db).await;
    }

    for kiosk in objectified.4 {
        let _ = Kiosk::insert_raw(kiosk, session.clone(), db).await;
    }
}

#[cfg(feature = "process")]
pub async fn session_garbage_collector(db: &DbConn) {
    let mut interval = tokio::time::interval(Duration::from_secs(5));

    loop {
        interval.tick().await;

        match session::Entity::find()
            .having(session::Column::Expiry.lt(Utc::now().naive_utc()))
            .all(db)
            .await
        {
            Ok(data) => {
                for model in data {
                    // Delete all model instances of sessions which have surpassed their existence time-frame.
                    match session::Entity::delete_by_id(model.id).exec(db).await {
                        Ok(_data) => {}
                        Err(err) => {
                            println!("[err]: Error in scheduled cron task: {:?}", err)
                        }
                    }
                }
            }
            Err(err) => {
                println!("[err]: Error in scheduled cron task: {:?}", err)
            }
        };

        let time = Utc::now().checked_sub_signed(ChronoDuration::seconds(3600));

        match time {
            Some(val) => {
                match transactions::Entity::find()
                    .having(transactions::Column::TransactionType.eq("saved"))
                    .having(transactions::Column::OrderDate.lte(val.naive_utc()))
                    .all(db)
                    .await
                {
                    Ok(data) => {
                        for model in data {
                            // Delete all model instances of sessions which have surpassed their existence time-frame.
                            match transactions::Entity::delete_by_id(model.id).exec(db).await {
                                Ok(_data) => {
                                    println!("[log]: Culled transaction")
                                }
                                Err(err) => {
                                    println!("[err]: Error in scheduled cron task: {:?}", err)
                                }
                            }
                        }
                    }
                    Err(err) => {
                        println!("[err]: Error in scheduled cron task: {:?}", err)
                    }
                };
            }
            None => {
                println!("[err]: Error in cron task: Unable to format DateTime")
            }
        };
    }
}
