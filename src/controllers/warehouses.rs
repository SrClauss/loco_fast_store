use crate::models::_entities::warehouses::Column as WarehouseColumn;
use axum::extract::Query;
use loco_rs::prelude::*;
use sea_orm::QueryFilter;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    dto::{entities::WarehouseResponse, response::ApiResponse},
    models::{
        _entities::users,
        warehouses::{self, CreateWarehouseParams, Model as WarehouseModel},
    },
};

#[derive(Debug, Deserialize)]
pub struct WarehouseQuery {
    pub q: Option<String>,
}

/// POST /api/v1/warehouses - Cria armazém
#[debug_handler]
async fn create(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(params): Json<CreateWarehouseParams>,
) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    crate::controllers::guards::ensure_admin(&user).await?;
    let wh = WarehouseModel::create_warehouse(&ctx.db, &params).await?;
    format::json(ApiResponse::success(WarehouseResponse::from(wh)))
}

/// GET /api/v1/warehouses - Lista armazéns
#[debug_handler]
async fn list(
    State(ctx): State<AppContext>,
    Query(_query): Query<WarehouseQuery>,
) -> Result<Response> {
    let whs: Vec<crate::models::_entities::warehouses::Model> =
        <crate::models::_entities::warehouses::Model as sea_orm::ModelTrait>::Entity::find()
            .filter(WarehouseColumn::DeletedAt.is_null())
            .all(&ctx.db)
            .await?;
    let response: Vec<WarehouseResponse> = whs.into_iter().map(WarehouseResponse::from).collect();
    format::json(ApiResponse::success(response))
}

/// GET /api/v1/warehouses/:pid
#[debug_handler]
async fn get_one(State(ctx): State<AppContext>, Path(pid): Path<Uuid>) -> Result<Response> {
    let wh: WarehouseModel = WarehouseModel::find_by_pid(&ctx.db, &pid).await?;
    format::json(ApiResponse::success(WarehouseResponse::from(wh)))
}

/// PUT /api/v1/warehouses/:pid
#[debug_handler]
async fn update(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(pid): Path<Uuid>,
    Json(params): Json<CreateWarehouseParams>,
) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    crate::controllers::guards::ensure_admin(&user).await?;
    let wh: WarehouseModel = WarehouseModel::find_by_pid(&ctx.db, &pid).await?;
    let mut active: warehouses::ActiveModel = wh.into();
    active.name = ActiveValue::set(params.name.clone());
    active.latitude = ActiveValue::set(params.latitude);
    active.longitude = ActiveValue::set(params.longitude);
    let updated = active.update(&ctx.db).await?;
    format::json(ApiResponse::success(WarehouseResponse::from(updated)))
}

/// DELETE /api/v1/warehouses/:pid - Soft delete
#[debug_handler]
async fn remove(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(pid): Path<Uuid>,
) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    crate::controllers::guards::ensure_admin(&user).await?;
    let wh: WarehouseModel = WarehouseModel::find_by_pid(&ctx.db, &pid).await?;
    let mut active: warehouses::ActiveModel = wh.into();
    active.deleted_at = ActiveValue::set(Some(chrono::Utc::now().into()));
    active.update(&ctx.db).await?;
    format::json(ApiResponse::<()>::success(()))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/v1/warehouses")
        .add("/", post(create))
        .add("/", get(list))
        .add("/{pid}", get(get_one))
        .add("/{pid}", put(update))
        .add("/{pid}", delete(remove))
}
