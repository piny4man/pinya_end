use once_cell::sync::OnceCell;
use salvo::prelude::*;
use salvo::cors::Cors;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
// use std::env;

static DBGRES: OnceCell<PgPool> = OnceCell::new();

#[derive(FromRow, Deserialize, Serialize, Debug)]
pub struct Pinya {
    pub id: String,
    pub alias: String
}

#[inline]
pub fn get_postgres() -> &'static PgPool {
    unsafe { DBGRES.get_unchecked() }
}

#[handler]
pub async fn create_pinya(req: &mut Request, res: &mut Response) {
    let new_pinya = req.parse_body::<Pinya>().await.unwrap();
    let data = sqlx::query_as::<_, Pinya>("INSERT INTO pinyas (id, alias) VALUES ($1, $2) RETURNING id, alias")
        .bind(new_pinya.id)
        .bind(new_pinya.alias)
        .fetch_one(get_postgres())
        .await
        .unwrap();
    // match data {
    //     Ok(id) =>
    // }
    res.render(serde_json::to_string(&data).unwrap());
}

#[handler]
pub async fn get_all_pinyas(_req: &mut Request, res: &mut Response) {
    // let uid = req.param::<String>("uid").unwrap();
    let data = sqlx::query_as::<_, Pinya>("SELECT id, alias FROM pinyas")
        // .bind(uid)
        .fetch_all(get_postgres())
        .await
        .unwrap();
    res.render(serde_json::to_string(&data).unwrap());
}

#[handler]
pub async fn get_pinya(req: &mut Request, res: &mut Response) {
    let uid = req.param::<String>("uid").unwrap();
    let data = sqlx::query_as::<_, Pinya>("SELECT id, alias FROM pinyas WHERE id = $1")
        .bind(uid)
        .fetch_one(get_postgres())
        .await
        .unwrap();
    res.render(serde_json::to_string(&data).unwrap());
}

#[handler]
async fn hello_pinyas(_res: &mut Response) -> Result<&'static str, ()> {
    Ok("Hello Pinyas locas del chat!")
}

#[shuttle_service::main]
async fn salvo(
    #[shuttle_shared_db::Postgres(
        local_uri = "postgresql://postgres:Test@123@localhost:5432/postgres"
    )] pool: PgPool
) -> shuttle_service::ShuttleSalvo {
    DBGRES.set(pool).unwrap();

    let cors_handler = Cors::builder()
        .allow_origin("http://localhost:8080")
        .allow_methods(vec!["OPTIONS", "GET", "POST", "DELETE"])
        .build();

    let router = Router::with_hoop(cors_handler)
        .push(
            Router::with_path("pinyas")
                .post(create_pinya)
                .get(get_all_pinyas)
                // .options(empty_handler)
                .push(Router::with_path("<uid>").get(get_pinya))
        );
        // .push(
        //     Router::new().get(hello_pinyas)
        // );

    Ok(router)
}