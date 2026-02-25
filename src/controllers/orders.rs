use axum::extract::Query;
use loco_rs::prelude::*;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    dto::{
        entities::{OrderResponse, OrderItemResponse},
        response::ApiResponse,
    },
    models::{
        _entities::users,
        carts::Model as CartModel,
        orders::{CreateOrderFromCartParams, Model as OrderModel},
    },
};

#[derive(Debug, Deserialize)]
pub struct OrderListQuery {
    pub status: Option<String>,
    pub cursor: Option<i32>,
    pub limit: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOrderStatusParams {
    pub status: Option<String>,
    pub payment_status: Option<String>,
    pub fulfillment_status: Option<String>,
    pub payment_data: Option<serde_json::Value>,
}

/// POST /api/stores/:store_pid/orders - Cria pedido a partir de carrinho
#[debug_handler]
async fn create(
    State(ctx): State<AppContext>,
    Path(store_pid): Path<Uuid>,
    Json(params): Json<CreateOrderFromCartParams>,
) -> Result<Response> {
    let store = crate::models::stores::Model::find_by_pid(&ctx.db, &store_pid).await?;

    // Busca o último carrinho ativo do customer
    let carts = crate::models::_entities::carts::Entity::find()
        .filter(crate::models::_entities::carts::Column::StoreId.eq(store.id))
        .filter(crate::models::_entities::carts::Column::CustomerId.eq(params.customer_id))
        .filter(crate::models::_entities::carts::Column::Status.eq("active"))
        .one(&ctx.db)
        .await?;

    let cart = carts.ok_or_else(|| loco_rs::Error::NotFound)?;
    let cart_items = CartModel::get_items(&ctx.db, cart.id).await?;

    if cart_items.is_empty() {
        return format::json(ApiResponse::<()>::error("EMPTY_CART", "Carrinho está vazio"));
    }

    let order =
        OrderModel::create_from_cart(&ctx.db, store.id, &cart, &cart_items, &params).await?;

    // Marca carrinho como completed
    CartModel::complete(&ctx.db, cart.id).await?;

    let items = OrderModel::get_items(&ctx.db, order.id).await?;
    let item_responses: Vec<OrderItemResponse> =
        items.into_iter().map(OrderItemResponse::from).collect();
    let mut response = OrderResponse::from(order);
    response.items = Some(item_responses);

    format::json(ApiResponse::success(response))
}

/// GET /api/stores/:store_pid/orders - Lista pedidos
#[debug_handler]
async fn list(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(store_pid): Path<Uuid>,
    Query(query): Query<OrderListQuery>,
) -> Result<Response> {
    let _user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let store = crate::models::stores::Model::find_by_pid(&ctx.db, &store_pid).await?;

    let limit = query.limit.unwrap_or(20);
    let orders = OrderModel::list_for_store(
        &ctx.db,
        store.id,
        query.status.as_deref(),
        query.cursor,
        limit,
    )
    .await?;

    let has_more = orders.len() as u64 >= limit.min(100);
    let cursor = orders.last().map(|o| o.id.to_string());
    let count = orders.len();

    let response: Vec<OrderResponse> = orders.into_iter().map(OrderResponse::from).collect();
    format::json(ApiResponse::paginated(response, cursor, has_more, count))
}

/// GET /api/stores/:store_pid/orders/:pid - Detalhes do pedido
#[debug_handler]
async fn get_one(
    State(ctx): State<AppContext>,
    Path((_store_pid, pid)): Path<(Uuid, Uuid)>,
) -> Result<Response> {
    let order = OrderModel::find_by_pid(&ctx.db, &pid).await?;
    let items = OrderModel::get_items(&ctx.db, order.id).await?;
    let item_responses: Vec<OrderItemResponse> =
        items.into_iter().map(OrderItemResponse::from).collect();
    let mut response = OrderResponse::from(order);
    response.items = Some(item_responses);
    format::json(ApiResponse::success(response))
}

/// PUT /api/stores/:store_pid/orders/:pid/status - Atualiza status do pedido
#[debug_handler]
async fn update_status(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path((_store_pid, pid)): Path<(Uuid, Uuid)>,
    Json(params): Json<UpdateOrderStatusParams>,
) -> Result<Response> {
    let _user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let order = OrderModel::find_by_pid(&ctx.db, &pid).await?;

    let mut updated = order;
    if let Some(status) = params.status {
        updated = OrderModel::update_status(&ctx.db, updated.id, &status).await?;
    }
    if let Some(payment_status) = params.payment_status {
        updated = OrderModel::update_payment_status(
            &ctx.db,
            updated.id,
            &payment_status,
            params.payment_data,
        )
        .await?;
    }
    if let Some(fulfillment_status) = params.fulfillment_status {
        updated =
            OrderModel::update_fulfillment_status(&ctx.db, updated.id, &fulfillment_status)
                .await?;
    }

    format::json(ApiResponse::success(OrderResponse::from(updated)))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/stores/{store_pid}/orders")
        .add("/", post(create))
        .add("/", get(list))
        .add("/{pid}", get(get_one))
        .add("/{pid}/status", put(update_status))
}
