use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::*;

#[derive(Deserialize)]
pub struct Params {
    limit: Option<i32>,
    offset: Option<i32>,
}

#[derive(Serialize, sqlx::Type)]
#[sqlx(type_name = "category")]
enum Category {
    Game,
    Video,
    Remote,
    Music,
    Tool,
    Proxy,
    Development,
}

#[derive(Serialize, sqlx::Type)]
#[sqlx(type_name = "platform")]
enum Platform {
    Windows,
    Android,
    IOS,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct Resource {
    key: String,
    display_name: String,
    icon_url: Option<String>,
    description: Option<String>,
    category: Category,
    platform: Platform,
    website: Option<String>,
    download_url: String,
    readme_md: Option<String>,
    warning_text: Option<String>,
}

#[axum::debug_handler]
pub async fn handler(
    Query(params): Query<Params>,
    State(db_pool): State<sqlx::postgres::PgPool>,
) -> impl IntoResponse {
    // 检查参数合法性
    // `limit` 和 `offset` 必须被同时指定或未被指定
    if (params.limit.is_some() && params.offset.is_none())
        || (params.limit.is_none() && params.offset.is_some())
    {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            Json(json!({
                "result": "failed",
                "message": "Both `limit` and `offset` must be specified or unspecified."
            })),
        )
            .into_response();
    }

    // 如果同时存在查询参数 `limit` 和 `offset` 则使用分页查询
    let (sql, binds) = if let (Some(limit), Some(offset)) = (params.limit, params.offset) {
        (
            "SELECT * FROM resources LIMIT $1 OFFSET $2;",
            vec![limit, offset],
        )
    // 否则使用正常查询
    } else {
        ("SELECT * FROM resources;", vec![])
    };

    let mut query = sqlx::query_as::<_, Resource>(sql);
    // 动态构建绑定
    for bind in &binds {
        query = query.bind(bind);
    }

    // 执行查询
    let resources = match query.fetch_all(&db_pool).await {
        Ok(resources) => resources,
        Err(e) => {
            error!("Failed to fetch resources from database: {}", e);
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "result": "failed",
                    "message": "Failed to fetch resources from database."
                })),
            )
                .into_response();
        }
    };

    Json(json!({
        "result": "success",
        "length": resources.len(),
        "resources": resources
    }))
    .into_response()
}
