use axum::Router;

use sqlx::PgPool;

mod challenges;

#[shuttle_runtime::main]
async fn main(#[shuttle_shared_db::Postgres] pool: PgPool) -> shuttle_axum::ShuttleAxum {
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");
    let router = Router::new().nest("/", challenges::router(pool.clone()));

    Ok(router.into())
}
