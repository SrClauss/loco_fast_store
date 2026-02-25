use axum::extract::Query;
use loco_rs::prelude::*;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    dto::{
        entities::StoreResponse,
        response::ApiResponse,
    },
    models::{
        _entities::users,
        stores::{CreateStoreParams, UpdateStoreParams},
    },
};

#[derive(Debug, Deserialize)]
pub struct StoreQuery {
    pub slug: Option<String>,
}

/// POST /api/stores - Cria uma nova loja
#[debug_handler]
async fn create(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(params): Json<CreateStoreParams>,
) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let store = crate::models::stores::Model::create_store(&ctx.db, user.id, &params).await?;
    format::json(ApiResponse::success(StoreResponse::from(store)))
}

/// GET /api/stores/:pid - Busca loja por PID
#[debug_handler]
async fn get_one(
    State(ctx): State<AppContext>,
    Path(pid): Path<Uuid>,
) -> Result<Response> {
    let store = crate::models::stores::Model::find_by_pid(&ctx.db, &pid).await?;
    format::json(ApiResponse::success(StoreResponse::from(store)))
}

/// GET /api/stores - Lista lojas do owner ou busca por slug
#[debug_handler]
async fn list(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Query(query): Query<StoreQuery>,
) -> Result<Response> {
    if let Some(slug) = query.slug {
        let store = crate::models::stores::Model::find_by_slug(&ctx.db, &slug).await?;
        return format::json(ApiResponse::success(vec![StoreResponse::from(store)]));
    }

    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let stores = crate::models::stores::Model::find_by_owner(&ctx.db, user.id).await?;
    let response: Vec<StoreResponse> = stores.into_iter().map(StoreResponse::from).collect();
    format::json(ApiResponse::success(response))
}

/// PUT /api/stores/:pid - Atualiza loja
#[debug_handler]
async fn update(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(pid): Path<Uuid>,
    Json(params): Json<UpdateStoreParams>,
) -> Result<Response> {
    let _user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let store = crate::models::stores::Model::find_by_pid(&ctx.db, &pid).await?;

    let mut active: crate::models::_entities::stores::ActiveModel = store.into();

    if let Some(name) = params.name {
        active.name = ActiveValue::set(name);
    }
    if let Some(domain) = params.domain {
        active.domain = ActiveValue::set(Some(domain));
    }
    if let Some(currency) = params.default_currency {
        active.default_currency = ActiveValue::set(currency);
    }
    if let Some(config) = params.config {
        active.config = ActiveValue::set(config);
    }
    if let Some(status) = params.status {
        active.status = ActiveValue::set(status);
    }
    if let Some(metadata) = params.metadata {
        active.metadata = ActiveValue::set(metadata);
    }

    let updated = active.update(&ctx.db).await?;
    format::json(ApiResponse::success(StoreResponse::from(updated)))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/stores")
        .add("/", post(create))
        .add("/", get(list))
        .add("/{pid}", get(get_one))
        .add("/{pid}", put(update))
}
