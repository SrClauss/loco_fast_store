use axum::extract::Query;
use loco_rs::prelude::*;
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, PaginatorTrait};
use serde::{Deserialize, Serialize};
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

/// POST /api/v1/orders - Cria pedido a partir de carrinho
#[debug_handler]
async fn create(
    State(ctx): State<AppContext>,
    Json(params): Json<CreateOrderFromCartParams>,
) -> Result<Response> {
    // Busca o último carrinho ativo do customer
    let carts = crate::models::_entities::carts::Entity::find()
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
        OrderModel::create_from_cart(&ctx.db, &cart, &cart_items, &params).await?;

    // Marca carrinho como completed
    CartModel::complete(&ctx.db, cart.id).await?;

    let items = OrderModel::get_items(&ctx.db, order.id).await?;
    let item_responses: Vec<OrderItemResponse> =
        items.into_iter().map(OrderItemResponse::from).collect();
    let mut response = OrderResponse::from(order);
    response.items = Some(item_responses);

    format::json(ApiResponse::success(response))
}

/// GET /api/v1/orders - Lista pedidos
#[debug_handler]
async fn list(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Query(query): Query<OrderListQuery>,
) -> Result<Response> {
    let _user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;

    let limit = query.limit.unwrap_or(20);
    let orders = OrderModel::list_for_store(
        &ctx.db,
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

/// GET /api/v1/orders/:pid - Detalhes do pedido
#[debug_handler]
async fn get_one(
    State(ctx): State<AppContext>,
    Path(pid): Path<Uuid>,
) -> Result<Response> {
    let order = OrderModel::find_by_pid(&ctx.db, &pid).await?;
    let items = OrderModel::get_items(&ctx.db, order.id).await?;
    let item_responses: Vec<OrderItemResponse> =
        items.into_iter().map(OrderItemResponse::from).collect();
    let mut response = OrderResponse::from(order);
    response.items = Some(item_responses);
    format::json(ApiResponse::success(response))
}

/// PUT /api/v1/orders/:pid/status - Atualiza status do pedido
#[debug_handler]
async fn update_status(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(pid): Path<Uuid>,
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

/// Stats para painel admin
#[derive(Debug, Serialize)]
pub struct OrderStats {
    pub total: u64,
    pub pending: u64,
    pub confirmed: u64,
    pub processing: u64,
    pub shipped: u64,
    pub delivered: u64,
    pub cancelled: u64,
}

/// GET /api/admin/orders/stats
#[debug_handler]
async fn admin_stats(
    State(ctx): State<AppContext>,
) -> Result<Response> {
    use crate::models::_entities::orders;
    
    let total = orders::Entity::find().count(&ctx.db).await?;
    let pending = orders::Entity::find()
        .filter(orders::Column::Status.eq("pending"))
        .count(&ctx.db)
        .await?;
    let confirmed = orders::Entity::find()
        .filter(orders::Column::Status.eq("confirmed"))
        .count(&ctx.db)
        .await?;
    let processing = orders::Entity::find()
        .filter(orders::Column::Status.eq("processing"))
        .count(&ctx.db)
        .await?;
    let shipped = orders::Entity::find()
        .filter(orders::Column::Status.eq("shipped"))
        .count(&ctx.db)
        .await?;
    let delivered = orders::Entity::find()
        .filter(orders::Column::Status.eq("delivered"))
        .count(&ctx.db)
        .await?;
    let cancelled = orders::Entity::find()
        .filter(orders::Column::Status.eq("cancelled"))
        .count(&ctx.db)
        .await?;
    
    let stats = OrderStats {
        total,
        pending,
        confirmed,
        processing,
        shipped,
        delivered,
        cancelled,
    };
    
    format::json(stats)
}

/// GET /api/admin/orders
#[debug_handler]
async fn admin_list(
    State(ctx): State<AppContext>,
) -> Result<Response> {
    use crate::models::_entities::orders;
    
    let orders_list = orders::Entity::find()
        .all(&ctx.db)
        .await?;
    let response: Vec<OrderResponse> =
        orders_list.into_iter().map(OrderResponse::from).collect();
    format::json(ApiResponse::success(response))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/v1/orders")
        .add("/", post(create))
        .add("/", get(list))
        .add("/{pid}", get(get_one))
        .add("/{pid}/status", put(update_status))
}

pub fn admin_routes() -> Routes {
    Routes::new()
        .prefix("/api/admin")
        .add("/orders/stats", get(admin_stats))
        .add("/orders", get(admin_list))
}
