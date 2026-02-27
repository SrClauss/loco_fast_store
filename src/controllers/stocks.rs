use crate::models::_entities::stocks::Column as StockColumn;
use axum::extract::Query;
use loco_rs::prelude::*;
use sea_orm::QueryFilter;
use serde::Deserialize;

use crate::{
    dto::{entities::StockResponse, response::ApiResponse},
    models::{
        _entities::users,
        stocks::{self, Model as StockModel, UpdateStockParams},
    },
};

#[derive(Debug, Deserialize)]
pub struct StockQuery {
    pub warehouse_id: Option<i32>,
    pub item_id: Option<i32>,
}

/// POST /api/v1/stocks - Upsert stock
#[debug_handler]
async fn upsert(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(params): Json<UpdateStockParams>,
) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    crate::controllers::guards::ensure_warehouse(&user).await?;
    let stock = StockModel::set_stock(&ctx.db, &params).await?;
    format::json(ApiResponse::success(StockResponse::from(stock)))
}

/// GET /api/v1/stocks - Lista estoques
#[debug_handler]
async fn list(
    State(ctx): State<AppContext>,
    Query(query): Query<StockQuery>,
) -> Result<Response> {
    let mut q = stocks::Entity::find();
    if let Some(wid) = query.warehouse_id {
        q = q.filter(StockColumn::WarehouseId.eq(wid));
    }
    if let Some(iid) = query.item_id {
        q = q.filter(StockColumn::ItemId.eq(iid));
    }
    let stocks_list = q.all(&ctx.db).await?;
    let response: Vec<StockResponse> = stocks_list.into_iter().map(StockResponse::from).collect();
    format::json(ApiResponse::success(response))
}

/// GET /api/v1/stocks/:id
#[debug_handler]
async fn get_one(State(ctx): State<AppContext>, Path(id): Path<i32>) -> Result<Response> {
    let stock = stocks::Entity::find_by_id(id)
        .one(&ctx.db)
        .await?
        .ok_or_else(|| Error::NotFound)?;
    format::json(ApiResponse::success(StockResponse::from(stock)))
}

/// DELETE /api/v1/stocks/:id - Remove stock entry
#[debug_handler]
async fn remove(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(id): Path<i32>,
) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    crate::controllers::guards::ensure_warehouse(&user).await?;
    let stock = stocks::Entity::find_by_id(id)
        .one(&ctx.db)
        .await?
        .ok_or_else(|| Error::NotFound)?;
    let active: stocks::ActiveModel = stock.into();
    active.delete(&ctx.db).await?;
    format::json(ApiResponse::<()>::success(()))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/v1/stocks")
        .add("/", post(upsert))
        .add("/", get(list))
        .add("/{id}", get(get_one))
        .add("/{id}", delete(remove))
}
