use axum::extract::Query;
use loco_rs::prelude::*;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    dto::{
        entities::CategoryResponse,
        response::ApiResponse,
    },
    models::{
        _entities::users,
        categories::{CreateCategoryParams, UpdateCategoryParams},
    },
};

#[derive(Debug, Deserialize)]
pub struct CategoryQuery {
    pub parent_id: Option<i32>,
}

/// POST /api/stores/:store_pid/categories - Cria categoria
#[debug_handler]
async fn create(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(store_pid): Path<Uuid>,
    Json(params): Json<CreateCategoryParams>,
) -> Result<Response> {
    let _user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let store = crate::models::stores::Model::find_by_pid(&ctx.db, &store_pid).await?;
    let category =
        crate::models::categories::Model::create_category(&ctx.db, store.id, &params).await?;
    format::json(ApiResponse::success(CategoryResponse::from(category)))
}

/// GET /api/stores/:store_pid/categories - Lista categorias
#[debug_handler]
async fn list(
    State(ctx): State<AppContext>,
    Path(store_pid): Path<Uuid>,
    Query(query): Query<CategoryQuery>,
) -> Result<Response> {
    let store = crate::models::stores::Model::find_by_pid(&ctx.db, &store_pid).await?;
    let categories =
        crate::models::categories::Model::list_for_store(&ctx.db, store.id, query.parent_id)
            .await?;
    let response: Vec<CategoryResponse> =
        categories.into_iter().map(CategoryResponse::from).collect();
    format::json(ApiResponse::success(response))
}

/// GET /api/stores/:store_pid/categories/:pid
#[debug_handler]
async fn get_one(
    State(ctx): State<AppContext>,
    Path((_store_pid, pid)): Path<(Uuid, Uuid)>,
) -> Result<Response> {
    let category = crate::models::categories::Model::find_by_pid(&ctx.db, &pid).await?;
    format::json(ApiResponse::success(CategoryResponse::from(category)))
}

/// PUT /api/stores/:store_pid/categories/:pid
#[debug_handler]
async fn update(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path((_store_pid, pid)): Path<(Uuid, Uuid)>,
    Json(params): Json<UpdateCategoryParams>,
) -> Result<Response> {
    let _user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let category = crate::models::categories::Model::find_by_pid(&ctx.db, &pid).await?;

    let mut active: crate::models::_entities::categories::ActiveModel = category.into();
    if let Some(name) = params.name {
        active.name = ActiveValue::set(name);
    }
    if let Some(slug) = params.slug {
        active.slug = ActiveValue::set(slug);
    }
    if let Some(description) = params.description {
        active.description = ActiveValue::set(Some(description));
    }
    if let Some(parent_id) = params.parent_id {
        active.parent_id = ActiveValue::set(Some(parent_id));
    }
    if let Some(image_url) = params.image_url {
        active.image_url = ActiveValue::set(Some(image_url));
    }
    if let Some(sort_order) = params.sort_order {
        active.sort_order = ActiveValue::set(sort_order);
    }

    let updated = active.update(&ctx.db).await?;
    format::json(ApiResponse::success(CategoryResponse::from(updated)))
}

/// DELETE /api/stores/:store_pid/categories/:pid - Soft delete
#[debug_handler]
async fn remove(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path((_store_pid, pid)): Path<(Uuid, Uuid)>,
) -> Result<Response> {
    let _user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let category = crate::models::categories::Model::find_by_pid(&ctx.db, &pid).await?;
    let mut active: crate::models::_entities::categories::ActiveModel = category.into();
    active.deleted_at = ActiveValue::set(Some(chrono::Utc::now().into()));
    active.update(&ctx.db).await?;
    format::json(ApiResponse::<()>::success(()))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/stores/{store_pid}/categories")
        .add("/", post(create))
        .add("/", get(list))
        .add("/{pid}", get(get_one))
        .add("/{pid}", put(update))
        .add("/{pid}", delete(remove))
}
