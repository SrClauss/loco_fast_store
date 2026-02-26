use loco_rs::prelude::*;
use uuid::Uuid;

use crate::{
    dto::{
        entities::CollectionResponse,
        response::ApiResponse,
    },
    models::{
        _entities::users,
        collections::{CreateCollectionParams, Model as CollectionModel},
    },
};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AddProductParams {
    pub product_pid: Uuid,
    pub sort_order: Option<i32>,
}

/// POST /api/v1/collections
#[debug_handler]
async fn create(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(params): Json<CreateCollectionParams>,
) -> Result<Response> {
    let _user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let collection =
        CollectionModel::create_collection(&ctx.db, &params).await?;
    format::json(ApiResponse::success(CollectionResponse::from(collection)))
}

/// GET /api/v1/collections
#[debug_handler]
async fn list(
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let collections = CollectionModel::list_for_store(&ctx.db).await?;
    let response: Vec<CollectionResponse> =
        collections.into_iter().map(CollectionResponse::from).collect();
    format::json(ApiResponse::success(response))
}

/// GET /api/v1/collections/:pid
#[debug_handler]
async fn get_one(
    State(ctx): State<AppContext>,
    Path(pid): Path<Uuid>,
) -> Result<Response> {
    let collection = CollectionModel::find_by_pid(&ctx.db, &pid).await?;
    format::json(ApiResponse::success(CollectionResponse::from(collection)))
}

/// POST /api/v1/collections/:pid/products - Adiciona produto
#[debug_handler]
async fn add_product(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(pid): Path<Uuid>,
    Json(params): Json<AddProductParams>,
) -> Result<Response> {
    let _user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let collection = CollectionModel::find_by_pid(&ctx.db, &pid).await?;
    let product =
        crate::models::products::Model::find_by_pid(&ctx.db, &params.product_pid).await?;
    CollectionModel::add_product(&ctx.db, collection.id, product.id, params.sort_order).await?;
    format::json(ApiResponse::<()>::success(()))
}

/// DELETE /api/v1/collections/:pid/products/:product_pid
#[debug_handler]
async fn remove_product(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path((pid, product_pid)): Path<(Uuid, Uuid)>,
) -> Result<Response> {
    let _user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let collection = CollectionModel::find_by_pid(&ctx.db, &pid).await?;
    let product =
        crate::models::products::Model::find_by_pid(&ctx.db, &product_pid).await?;
    CollectionModel::remove_product(&ctx.db, collection.id, product.id).await?;
    format::json(ApiResponse::<()>::success(()))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/v1/collections")
        .add("/", post(create))
        .add("/", get(list))
        .add("/{pid}", get(get_one))
        .add("/{pid}/products", post(add_product))
        .add("/{pid}/products/{product_pid}", delete(remove_product))
}
