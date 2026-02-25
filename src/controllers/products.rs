use axum::extract::Query;
use loco_rs::prelude::*;
use uuid::Uuid;

use crate::{
    dto::{
        entities::{ProductResponse, VariantResponse, PriceResponse},
        response::ApiResponse,
    },
    models::{
        _entities::users,
        products::{CreateProductParams, ProductListParams, UpdateProductParams},
        product_variants::{CreateVariantParams, Model as VariantModel},
    },
};

/// POST /api/stores/:store_pid/products - Cria um produto
#[debug_handler]
async fn create(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(store_pid): Path<Uuid>,
    Json(params): Json<CreateProductParams>,
) -> Result<Response> {
    let _user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let store = crate::models::stores::Model::find_by_pid(&ctx.db, &store_pid).await?;
    let product =
        crate::models::products::Model::create_product(&ctx.db, store.id, &params).await?;
    format::json(ApiResponse::success(ProductResponse::from(product)))
}

/// GET /api/stores/:store_pid/products - Lista produtos
#[debug_handler]
async fn list(
    State(ctx): State<AppContext>,
    Path(store_pid): Path<Uuid>,
    Query(params): Query<ProductListParams>,
) -> Result<Response> {
    let store = crate::models::stores::Model::find_by_pid(&ctx.db, &store_pid).await?;
    let products =
        crate::models::products::Model::list_for_store(&ctx.db, store.id, &params).await?;

    let limit = params.limit.unwrap_or(20).min(100);
    let has_more = products.len() as u64 >= limit;
    let cursor = products.last().map(|p| p.id.to_string());
    let count = products.len();

    let response: Vec<ProductResponse> = products.into_iter().map(ProductResponse::from).collect();
    format::json(ApiResponse::paginated(response, cursor, has_more, count))
}

/// GET /api/stores/:store_pid/products/:pid - Busca produto detalhado (com variantes e preços)
#[debug_handler]
async fn get_one(
    State(ctx): State<AppContext>,
    Path((_store_pid, pid)): Path<(Uuid, Uuid)>,
) -> Result<Response> {
    let product = crate::models::products::Model::find_by_pid(&ctx.db, &pid).await?;

    // Carrega variantes com preços
    let variants = VariantModel::find_by_product(&ctx.db, product.id).await?;
    let mut variant_responses = Vec::new();
    for v in variants {
        let prices = VariantModel::get_prices(&ctx.db, v.id).await?;
        let price_responses: Vec<PriceResponse> =
            prices.into_iter().map(PriceResponse::from).collect();
        let mut vr = VariantResponse::from(v);
        vr.prices = Some(price_responses);
        variant_responses.push(vr);
    }

    let mut response = ProductResponse::from(product);
    response.variants = Some(variant_responses);

    format::json(ApiResponse::success(response))
}

/// PUT /api/stores/:store_pid/products/:pid - Atualiza produto
#[debug_handler]
async fn update(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path((_store_pid, pid)): Path<(Uuid, Uuid)>,
    Json(params): Json<UpdateProductParams>,
) -> Result<Response> {
    let _user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let product = crate::models::products::Model::find_by_pid(&ctx.db, &pid).await?;

    let mut active: crate::models::_entities::products::ActiveModel = product.into();
    if let Some(title) = params.title {
        active.title = ActiveValue::set(title);
    }
    if let Some(slug) = params.slug {
        active.slug = ActiveValue::set(slug);
    }
    if let Some(description) = params.description {
        active.description = ActiveValue::set(description);
    }
    if let Some(status) = params.status {
        active.status = ActiveValue::set(status);
    }
    if let Some(product_type) = params.product_type {
        active.product_type = ActiveValue::set(product_type);
    }
    if let Some(category_id) = params.category_id {
        active.category_id = ActiveValue::set(Some(category_id));
    }
    if let Some(tags) = params.tags {
        active.tags = ActiveValue::set(serde_json::json!(tags));
    }
    if let Some(featured) = params.featured {
        active.featured = ActiveValue::set(featured);
    }
    if let Some(metadata) = params.metadata {
        active.metadata = ActiveValue::set(metadata);
    }

    let updated = active.update(&ctx.db).await?;
    format::json(ApiResponse::success(ProductResponse::from(updated)))
}

/// DELETE /api/stores/:store_pid/products/:pid - Soft delete
#[debug_handler]
async fn remove(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path((_store_pid, pid)): Path<(Uuid, Uuid)>,
) -> Result<Response> {
    let _user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let product = crate::models::products::Model::find_by_pid(&ctx.db, &pid).await?;

    let mut active: crate::models::_entities::products::ActiveModel = product.into();
    active.deleted_at = ActiveValue::set(Some(chrono::Utc::now().into()));
    active.update(&ctx.db).await?;

    format::json(ApiResponse::<()>::success(()))
}

/// POST /api/stores/:store_pid/products/:pid/variants - Cria variante
#[debug_handler]
async fn create_variant(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path((_store_pid, pid)): Path<(Uuid, Uuid)>,
    Json(params): Json<CreateVariantParams>,
) -> Result<Response> {
    let _user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let product = crate::models::products::Model::find_by_pid(&ctx.db, &pid).await?;
    let variant = VariantModel::create_variant(&ctx.db, product.id, &params).await?;
    format::json(ApiResponse::success(VariantResponse::from(variant)))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/stores/{store_pid}/products")
        .add("/", post(create))
        .add("/", get(list))
        .add("/{pid}", get(get_one))
        .add("/{pid}", put(update))
        .add("/{pid}", delete(remove))
        .add("/{pid}/variants", post(create_variant))
}
