//! Provides a RESTful web server managing some Todos.
//!
//! API will be:
//!
//! - `GET /todos`: return a JSON list of Todos.
//! - `POST /todos`: create a new Todo.
//! - `PUT /todos/:id`: update a specific Todo.
//! - `DELETE /todos/:id`: delete a specific Todo.
//!
//! Run with
//!
//! ```not_rust
//! cd examples && cargo run -p example-todos
//! ```

use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, RwLock},
    time::Duration,
};

use axum::{
    error_handling::HandleErrorLayer,
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
    response::IntoResponse,
    Router, routing::put,
};
use axum::handler::Handler;
use deadpool_diesel::{Manager, Pool};
use diesel::MysqlConnection;
// 这里不引入AsyncConnection还不能使用establish方法。
use diesel_async::{AsyncConnection, AsyncMysqlConnection};
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use tower::{BoxError, ServiceBuilder};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;
use key_replacer_axum_api::control::{query_char_news, create_char};


// use key_replacer_axum_api::establish_query_connection_pool;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "replace_api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // let db = Db::default();
    // let query_db = establish_query_connection();
    // let query_pool = establish_query_connection_pool();
    dotenv().ok();
    let mut async_conn = AsyncMysqlConnection::establish(std::env::var("DATABASE_URL").unwrap().as_str()).await.unwrap();
    let shared_state = Arc::new(AppState { async_conn: Some(async_conn) });
    // let shared_conn = Arc::new(async_conn);
    // let shared_conn = Arc::new(query_db);

    // Compose the routes
    let app = Router::new()
        .route("/replace", put(keywords_update))
        // Add middleware to all routes
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|error: BoxError| async move {
                    if error.is::<tower::timeout::error::Elapsed>() {
                        Ok(StatusCode::REQUEST_TIMEOUT)
                    } else {
                        Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Unhandled internal error: {}", error),
                        ))
                    }
                }))
                .timeout(Duration::from_secs(10))
                .layer(TraceLayer::new_for_http())
                .into_inner(),
        ).with_state(shared_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Debug, Deserialize)]
struct UpdateContent {
    content_id: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
struct Info {
    result: Option<String>,
}

// #[derive(Clone)]
struct AppState {
    async_conn: Option<AsyncMysqlConnection>,
}

async fn keywords_update(
    // State(query_db): State<MysqlConnection>,
    // State(query_pool): State<Pool<Manager<_>, _>>,
    // State(query_pool): State<Arc<MysqlConnection>>,
    // State(query_pool): State<AsyncMysqlConnection>,
    // State(query_pool): State<Arc<AsyncMysqlConnection>>,
    State(shared_state): State<Arc<AppState>>,
    Json(input): Json<UpdateContent>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut info = Info {
        result: Some("全局更新".to_string())
    };
    // 如果传递了id，就是指定更新
    if let Some(content_id) = input.content_id {
        info.result = Some(format!("指定更新{content_id}").to_string());
        // query_char_news(&mut shared_state.async_conn.take().unwrap()).await.unwrap();
        // let mut async_conn = shared_state.async_conn.take().unwrap();
        // let mut new_state = shared_state.clone();
        // query_char_news(&mut new_state.async_conn.unwrap()).await.unwrap();
        // shared_state.async_conn = Some(async_conn);
        let mut query_async_conn = AsyncMysqlConnection::establish(std::env::var("QUERY_DATABASE_URL").unwrap().as_str()).await.unwrap();
        query_char_news(&mut query_async_conn).await.unwrap();
        let mut async_conn = AsyncMysqlConnection::establish(std::env::var("DATABASE_URL").unwrap().as_str()).await.unwrap();
        create_char(&mut async_conn, "title", "content").await;
    }

    Ok((StatusCode::OK, Json(info)))
}
