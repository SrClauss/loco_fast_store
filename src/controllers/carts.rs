use axum::extract::Query;
use loco_rs::prelude::*;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    dto::{
        entities::{CartResponse, CartItemResponse},
        response::ApiResponse,
    },
    models::{
        carts::{AddToCartParams, Model as CartModel},
        product_variants::Model as VariantModel,
    },
};

#[derive(Debug, Deserialize)]
pub struct CartQuery {
    pub session_id: String,
}

/// POST /api/stores/:store_pid/carts - Cria ou retorna carrinho pela session
#[debug_handler]
async fn get_or_create(
    State(ctx): State<AppContext>,
    Path(store_pid): Path<Uuid>,
    Query(query): Query<CartQuery>,
) -> Result<Response> {
    let store = crate::models::stores::Model::find_by_pid(&ctx.db, &store_pid).await?;

    let existing = CartModel::find_active_by_session(&ctx.db, store.id, &query.session_id).await?;
    let cart = if let Some(c) = existing {
        c
    } else {
        CartModel::create_cart(&ctx.db, store.id, &query.session_id, None, None, None).await?
    };

    let items = CartModel::get_items(&ctx.db, cart.id).await?;
    let item_responses: Vec<CartItemResponse> = items.into_iter().map(CartItemResponse::from).collect();
    let mut response = CartResponse::from(cart);
    response.items = Some(item_responses);

    format::json(ApiResponse::success(response))
}

/// GET /api/stores/:store_pid/carts/:pid - Busca carrinho por PID
#[debug_handler]
async fn get_one(
    State(ctx): State<AppContext>,
    Path((_store_pid, pid)): Path<(Uuid, Uuid)>,
) -> Result<Response> {
    let cart = CartModel::find_by_pid(&ctx.db, &pid).await?;
    let items = CartModel::get_items(&ctx.db, cart.id).await?;
    let item_responses: Vec<CartItemResponse> = items.into_iter().map(CartItemResponse::from).collect();
    let mut response = CartResponse::from(cart);
    response.items = Some(item_responses);
    format::json(ApiResponse::success(response))
}

/// POST /api/stores/:store_pid/carts/:pid/items - Adiciona item ao carrinho
#[debug_handler]
async fn add_item(
    State(ctx): State<AppContext>,
    Path((_store_pid, pid)): Path<(Uuid, Uuid)>,
    Json(params): Json<AddToCartParams>,
) -> Result<Response> {
    let cart = CartModel::find_by_pid(&ctx.db, &pid).await?;

    // Busca preço da variante
    let price = VariantModel::get_active_price(&ctx.db, params.variant_id, &cart.currency, params.quantity)
        .await
        .map(|p| p.amount)
        .unwrap_or(0);

    CartModel::add_item(&ctx.db, cart.id, params.variant_id, params.quantity, price).await?;
    let cart = CartModel::recalculate_totals(&ctx.db, cart.id).await?;

    let items = CartModel::get_items(&ctx.db, cart.id).await?;
    let item_responses: Vec<CartItemResponse> = items.into_iter().map(CartItemResponse::from).collect();
    let mut response = CartResponse::from(cart);
    response.items = Some(item_responses);
    format::json(ApiResponse::success(response))
}

/// PUT /api/stores/:store_pid/carts/:pid/items/:item_id - Atualiza quantidade
#[debug_handler]
async fn update_item(
    State(ctx): State<AppContext>,
    Path((_store_pid, pid, item_id)): Path<(Uuid, Uuid, i32)>,
    Json(params): Json<crate::models::carts::UpdateCartItemParams>,
) -> Result<Response> {
    let cart = CartModel::find_by_pid(&ctx.db, &pid).await?;

    if params.quantity <= 0 {
        CartModel::remove_item(&ctx.db, item_id).await?;
    } else {
        CartModel::update_item_quantity(&ctx.db, item_id, params.quantity).await?;
    }

    let cart = CartModel::recalculate_totals(&ctx.db, cart.id).await?;
    let items = CartModel::get_items(&ctx.db, cart.id).await?;
    let item_responses: Vec<CartItemResponse> = items.into_iter().map(CartItemResponse::from).collect();
    let mut response = CartResponse::from(cart);
    response.items = Some(item_responses);
    format::json(ApiResponse::success(response))
}

/// DELETE /api/stores/:store_pid/carts/:pid/items/:item_id - Remove item
#[debug_handler]
async fn remove_item(
    State(ctx): State<AppContext>,
    Path((_store_pid, pid, item_id)): Path<(Uuid, Uuid, i32)>,
) -> Result<Response> {
    let cart = CartModel::find_by_pid(&ctx.db, &pid).await?;
    CartModel::remove_item(&ctx.db, item_id).await?;
    let cart = CartModel::recalculate_totals(&ctx.db, cart.id).await?;

    let items = CartModel::get_items(&ctx.db, cart.id).await?;
    let item_responses: Vec<CartItemResponse> = items.into_iter().map(CartItemResponse::from).collect();
    let mut response = CartResponse::from(cart);
    response.items = Some(item_responses);
    format::json(ApiResponse::success(response))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/stores/{store_pid}/carts")
        .add("/", post(get_or_create))
        .add("/{pid}", get(get_one))
        .add("/{pid}/items", post(add_item))
        .add("/{pid}/items/{item_id}", put(update_item))
        .add("/{pid}/items/{item_id}", delete(remove_item))
}
