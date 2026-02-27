use crate::models::_entities::items::Column as ItemColumn;
use axum::extract::Query;
use loco_rs::prelude::*;
use sea_orm::QueryFilter;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    dto::{entities::ItemResponse, response::ApiResponse},
    models::{
        _entities::users,
        items::{self, CreateItemParams, Model as ItemModel},
    },
};

#[derive(Debug, Deserialize)]
pub struct ItemQuery {
    pub variant_id: Option<i32>,
}

/// POST /api/v1/items - Cria item
#[debug_handler]
async fn create(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(params): Json<CreateItemParams>,
) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    crate::controllers::guards::ensure_warehouse(&user).await?;
    let item = ItemModel::create_item(&ctx.db, &params).await?;
    format::json(ApiResponse::success(ItemResponse::from(item)))
}

/// GET /api/v1/items - Lista itens (filtra deleted_at)
#[debug_handler]
async fn list(
    State(ctx): State<AppContext>,
    Query(query): Query<ItemQuery>,
) -> Result<Response> {
    let mut q = items::Entity::find().filter(ItemColumn::DeletedAt.is_null());
    if let Some(vid) = query.variant_id {
        q = q.filter(ItemColumn::VariantId.eq(vid));
    }
    let items_list = q.all(&ctx.db).await?;
    let response: Vec<ItemResponse> = items_list.into_iter().map(ItemResponse::from).collect();
    format::json(ApiResponse::success(response))
}

/// GET /api/v1/items/:pid
#[debug_handler]
async fn get_one(State(ctx): State<AppContext>, Path(pid): Path<Uuid>) -> Result<Response> {
    let item = ItemModel::find_by_pid(&ctx.db, &pid)
        .await?
        .ok_or_else(|| Error::NotFound)?;
    format::json(ApiResponse::success(ItemResponse::from(item)))
}

/// PUT /api/v1/items/:pid
#[debug_handler]
async fn update(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(pid): Path<Uuid>,
    Json(params): Json<CreateItemParams>,
) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    crate::controllers::guards::ensure_warehouse(&user).await?;
    let item = ItemModel::find_by_pid(&ctx.db, &pid)
        .await?
        .ok_or_else(|| Error::NotFound)?;
    let mut active: items::ActiveModel = item.into();
    active.variant_id = ActiveValue::set(params.variant_id);
    active.batch = ActiveValue::set(params.batch.clone());
    active.expiration = ActiveValue::set(params.expiration);
    let updated = active.update(&ctx.db).await?;
    format::json(ApiResponse::success(ItemResponse::from(updated)))
}

/// DELETE /api/v1/items/:pid - Soft delete
#[debug_handler]
async fn remove(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(pid): Path<Uuid>,
) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    crate::controllers::guards::ensure_warehouse(&user).await?;
    let item = ItemModel::find_by_pid(&ctx.db, &pid)
        .await?
        .ok_or_else(|| Error::NotFound)?;
    let mut active: items::ActiveModel = item.into();
    active.deleted_at = ActiveValue::set(Some(chrono::Utc::now().into()));
    active.update(&ctx.db).await?;
    format::json(ApiResponse::<()>::success(()))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/v1/items")
        .add("/", post(create))
        .add("/", get(list))
        .add("/{pid}", get(get_one))
        .add("/{pid}", put(update))
        .add("/{pid}", delete(remove))
}
