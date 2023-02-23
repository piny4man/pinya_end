use actix_cors::Cors;
use actix_web::{middleware::NormalizePath, web, web::ServiceConfig, HttpResponse};
use serde::{Deserialize, Serialize};
use shuttle_service::ShuttleActixWeb;
use sqlx::{FromRow, PgPool};
// use std::env;

#[derive(FromRow, Deserialize, Serialize, Debug)]
pub struct Pinya {
    pub id: String,
    pub alias: String
}

async fn get_all_pinyas(pool: web::Data<PgPool>) -> HttpResponse {
    let pool = pool.as_ref();
    match sqlx::query_as::<_, Pinya>("SELECT id, alias FROM pinyas")
        .fetch_all(pool)
        .await
    {
        Ok(pinya) => HttpResponse::Ok().json(pinya),
        Err(e) => HttpResponse::InternalServerError().body(format!("{:?}", e)),
    }
}

async fn post_pinya(pinya: web::Json<Pinya>, pool: web::Data<PgPool>) -> HttpResponse {
    let pool = pool.as_ref();
    match sqlx::query_as::<_, Pinya>(
        "INSERT INTO pinyas (id, alias) VALUES ($1, $2) RETURNING id, alias",
    )
    .bind(&pinya.id)
    .bind(&pinya.alias)
    .fetch_one(pool)
    .await
    {
        Ok(pinya) => HttpResponse::Ok().json(pinya),
        Err(e) => HttpResponse::InternalServerError().body(format!("{:?}", e)),
    }
}

//TODO: Implement get Pinya by ID
// #[handler]
// pub async fn get_pinya(req: &mut Request, res: &mut Response) {
//     let uid = req.param::<String>("uid").unwrap();
//     let data = sqlx::query_as::<_, Pinya>("SELECT id, alias FROM pinyas WHERE id = $1")
//         .bind(uid)
//         .fetch_one(get_postgres())
//         .await
//         .unwrap();
//     res.render(serde_json::to_string(&data).unwrap());
// }

#[shuttle_service::main]
async fn actix_web(
    #[shuttle_shared_db::Postgres(
        local_uri = "postgresql://postgres:Test@123@localhost:5432/postgres"
    )]
    pool: PgPool,
) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Sync + Send + Clone + 'static> {
    let pool = web::Data::new(pool);

    Ok(move |cfg: &mut ServiceConfig| {
        let cors = Cors::default()
            .allow_any_header()
            .allow_any_method()
            .allow_any_origin();

        cfg.app_data(pool).service(
            web::scope("/pinyas")
                .wrap(cors)
                .wrap(NormalizePath::new(
                    actix_web::middleware::TrailingSlash::Always,
                ))
                .route("/", web::get().to(get_all_pinyas))
                .route("/", web::post().to(post_pinya)),
        );
    })
}
