///! Painel de colaboradores — API JSON
use axum::extract::Query;
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    dto::response::ApiResponse,
    models::{
        _entities::users,
        order_shippings::{CreateShippingParams, Model as ShippingModel, UpdateShippingStatusParams},
        orders::Model as OrderModel,
        store_collaborators::Model as CollaboratorModel,
        stores::Model as StoreModel,
    },
    shipping,
};

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct OrderListQuery {
    pub status: Option<String>,
    pub fulfillment_status: Option<String>,
    pub cursor: Option<i32>,
    pub limit: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateOrderStatusParams {
    pub status: Option<String>,
    pub payment_status: Option<String>,
    pub fulfillment_status: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CalculateFreightParams {
    pub origin_postal_code: String,
    pub destination_postal_code: String,
    pub weight_grams: u32,
    pub length_cm: u32,
    pub width_cm: u32,
    pub height_cm: u32,
    pub declared_value_cents: i64,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Verifica se o usuário autenticado é colaborador da loja e tem a permissão mínima.
/// Retorna o colaborador ou 403.
async fn require_collab(
    db: &DatabaseConnection,
    user_pid: &str,
    store_pid: &Uuid,
    need_update: bool,
) -> Result<(users::Model, crate::models::_entities::stores::Model, crate::models::_entities::store_collaborators::Model)> {
    let user = users::Model::find_by_pid(db, user_pid).await?;
    let store = StoreModel::find_by_pid(db, store_pid).await?;
    let collab = CollaboratorModel::find_for_user_and_store(db, user.id, store.id)
        .await
        .map_err(|_| loco_rs::Error::Unauthorized("Sem acesso a esta loja".into()))?;

    if need_update && !collab.can_update_orders() {
        return Err(loco_rs::Error::Unauthorized("Permissão insuficiente".into()));
    }
    if !need_update && !collab.can_view_orders() {
        return Err(loco_rs::Error::Unauthorized("Permissão insuficiente".into()));
    }

    Ok((user, store, collab))
}

// ── Autenticação ──────────────────────────────────────────────────────────────

/// POST /api/painel/auth/login
/// Usa o mesmo endpoint de login do sistema administrativo.
/// Retorna JWT + lista de lojas acessíveis ao usuário.
#[debug_handler]
pub async fn login(
    State(ctx): State<AppContext>,
    Json(params): Json<crate::models::users::LoginParams>,
) -> Result<Response> {
    let Ok(user) = users::Model::find_by_email(&ctx.db, &params.email).await else {
        return unauthorized("Credenciais inválidas");
    };

    if !user.verify_password(&params.password) {
        return unauthorized("Credenciais inválidas");
    }

    let jwt_secret = ctx.config.get_jwt_config()?;
    let token = user
        .generate_jwt(&jwt_secret.secret, jwt_secret.expiration)
        .or_else(|_| unauthorized("Erro ao gerar token"))?;

    // Lista lojas acessíveis
    let collabs = CollaboratorModel::list_stores_for_user(&ctx.db, user.id).await?;
    let store_ids: Vec<i32> = collabs.iter().map(|c| c.store_id).collect();

    let stores: Vec<serde_json::Value> = {
        let mut result = vec![];
        for store_id in store_ids {
            if let Ok(s) = crate::models::_entities::stores::Entity::find_by_id(store_id)
                .one(&ctx.db)
                .await
            {
                if let Some(s) = s {
                    let role = collabs.iter()
                        .find(|c| c.store_id == store_id)
                        .map(|c| c.role.as_str())
                        .unwrap_or("viewer");
                    result.push(serde_json::json!({
                        "pid": s.pid.to_string(),
                        "name": s.name,
                        "slug": s.slug,
                        "role": role,
                    }));
                }
            }
        }
        result
    };

    format::json(ApiResponse::success(serde_json::json!({
        "token": token,
        "user": { "pid": user.pid.to_string(), "name": user.name, "email": user.email },
        "stores": stores,
    })))
}

// ── Pedidos ───────────────────────────────────────────────────────────────────

/// GET /api/painel/:store_pid/pedidos
/// Lista pedidos da loja com filtros e paginação.
#[debug_handler]
pub async fn list_orders(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(store_pid): Path<Uuid>,
    Query(query): Query<OrderListQuery>,
) -> Result<Response> {
    let (_, store, _) = require_collab(&ctx.db, &auth.claims.pid, &store_pid, false).await?;

    let limit = query.limit.unwrap_or(20).min(100);

    // Filtro por status e fulfillment_status
    let mut db_query = crate::models::_entities::orders::Entity::find()
        .filter(crate::models::_entities::orders::Column::StoreId.eq(store.id));

    if let Some(ref s) = query.status {
        db_query = db_query.filter(crate::models::_entities::orders::Column::Status.eq(s.as_str()));
    }
    if let Some(ref fs) = query.fulfillment_status {
        db_query = db_query.filter(
            crate::models::_entities::orders::Column::FulfillmentStatus.eq(fs.as_str()),
        );
    }
    if let Some(cursor) = query.cursor {
        db_query = db_query.filter(crate::models::_entities::orders::Column::Id.gt(cursor));
    }

    use sea_orm::{QueryOrder, QuerySelect};
    let orders = db_query
        .order_by_desc(crate::models::_entities::orders::Column::CreatedAt)
        .limit(limit)
        .all(&ctx.db)
        .await?;

    // Para cada pedido, busca o envio associado
    let mut result = vec![];
    for order in &orders {
        let shipping = ShippingModel::find_by_order(&ctx.db, order.id).await?;
        result.push(serde_json::json!({
            "pid": order.pid.to_string(),
            "order_number": order.order_number,
            "status": order.status,
            "payment_status": order.payment_status,
            "fulfillment_status": order.fulfillment_status,
            "currency": order.currency,
            "total": order.total,
            "created_at": order.created_at.to_string(),
            "shipping": shipping.map(|s| serde_json::json!({
                "pid": s.pid.to_string(),
                "carrier": s.carrier,
                "service": s.service,
                "tracking_code": s.tracking_code,
                "tracking_url": s.tracking_url,
                "status": s.status,
                "shipped_at": s.shipped_at.map(|t| t.to_string()),
                "estimated_delivery_at": s.estimated_delivery_at.map(|t| t.to_string()),
                "delivered_at": s.delivered_at.map(|t| t.to_string()),
            })),
        }));
    }

    let has_more = orders.len() as u64 >= limit;
    let cursor = orders.last().map(|o| o.id.to_string());
    format::json(ApiResponse::paginated(result, cursor, has_more, orders.len()))
}

/// GET /api/painel/:store_pid/pedidos/:order_pid
/// Detalhe completo do pedido com itens e envio.
#[debug_handler]
pub async fn get_order(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path((store_pid, order_pid)): Path<(Uuid, Uuid)>,
) -> Result<Response> {
    let (_, _, _) = require_collab(&ctx.db, &auth.claims.pid, &store_pid, false).await?;

    let order = OrderModel::find_by_pid(&ctx.db, &order_pid).await?;
    let items = OrderModel::get_items(&ctx.db, order.id).await?;
    let shipping = ShippingModel::find_by_order(&ctx.db, order.id).await?;

    // Tenta buscar endereço de entrega
    let shipping_address = if let Some(addr_id) = order.shipping_address_id {
        crate::models::_entities::addresses::Entity::find_by_id(addr_id)
            .one(&ctx.db)
            .await?
    } else {
        None
    };

    format::json(ApiResponse::success(serde_json::json!({
        "pid": order.pid.to_string(),
        "order_number": order.order_number,
        "status": order.status,
        "payment_status": order.payment_status,
        "fulfillment_status": order.fulfillment_status,
        "currency": order.currency,
        "subtotal": order.subtotal,
        "tax": order.tax,
        "shipping_cost": order.shipping,
        "discount": order.discount,
        "total": order.total,
        "payment_method": order.payment_method,
        "notes": order.notes,
        "created_at": order.created_at.to_string(),
        "paid_at": order.paid_at.map(|t| t.to_string()),
        "canceled_at": order.canceled_at.map(|t| t.to_string()),
        "shipping_address": shipping_address.map(|a| serde_json::json!({
            "first_name": a.first_name,
            "last_name": a.last_name,
            "address_line_1": a.address_line_1,
            "address_line_2": a.address_line_2,
            "city": a.city,
            "state": a.state,
            "postal_code": a.postal_code,
            "country": a.country,
            "phone": a.phone,
        })),
        "items": items.iter().map(|i| serde_json::json!({
            "pid": i.pid.to_string(),
            "title": i.title,
            "sku": i.sku,
            "quantity": i.quantity,
            "unit_price": i.unit_price,
            "total": i.total,
        })).collect::<Vec<_>>(),
        "shipping": shipping.map(|s| serde_json::json!({
            "pid": s.pid.to_string(),
            "carrier": s.carrier,
            "service": s.service,
            "tracking_code": s.tracking_code,
            "tracking_url": s.tracking_url,
            "status": s.status,
            "provider": s.provider,
            "shipped_at": s.shipped_at.map(|t| t.to_string()),
            "estimated_delivery_at": s.estimated_delivery_at.map(|t| t.to_string()),
            "delivered_at": s.delivered_at.map(|t| t.to_string()),
            "notes": s.notes,
        })),
    })))
}

/// PUT /api/painel/:store_pid/pedidos/:order_pid/status
/// Atualiza status do pedido (status, payment_status, fulfillment_status).
#[debug_handler]
pub async fn update_order_status(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path((store_pid, order_pid)): Path<(Uuid, Uuid)>,
    Json(params): Json<UpdateOrderStatusParams>,
) -> Result<Response> {
    let (_, _, _) = require_collab(&ctx.db, &auth.claims.pid, &store_pid, true).await?;

    let order = OrderModel::find_by_pid(&ctx.db, &order_pid).await?;
    let mut updated = order;

    if let Some(ref status) = params.status {
        updated = OrderModel::update_status(&ctx.db, updated.id, status).await?;
    }
    if let Some(ref ps) = params.payment_status {
        updated = OrderModel::update_payment_status(&ctx.db, updated.id, ps, None).await?;
    }
    if let Some(ref fs) = params.fulfillment_status {
        updated = OrderModel::update_fulfillment_status(&ctx.db, updated.id, fs).await?;
    }

    format::json(ApiResponse::success(serde_json::json!({
        "pid": updated.pid.to_string(),
        "order_number": updated.order_number,
        "status": updated.status,
        "payment_status": updated.payment_status,
        "fulfillment_status": updated.fulfillment_status,
    })))
}

// ── Envios ────────────────────────────────────────────────────────────────────

/// POST /api/painel/:store_pid/pedidos/:order_pid/envio
/// Registra envio (manual ou via provider externo).
#[debug_handler]
pub async fn create_shipping(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path((store_pid, order_pid)): Path<(Uuid, Uuid)>,
    Json(params): Json<CreateShippingParams>,
) -> Result<Response> {
    let (_, store, _) = require_collab(&ctx.db, &auth.claims.pid, &store_pid, true).await?;
    let order = OrderModel::find_by_pid(&ctx.db, &order_pid).await?;

    // Tenta usar provider externo se disponível
    let (provider_name, provider_data) = if let Some(_provider) = shipping::provider_for(&params.carrier) {
        // TODO: chamar provider.create_shipment(...) quando implementado
        // Por ora registra como manual mesmo com provider configurado
        (Some(params.carrier.as_str()), None)
    } else {
        (None, None)
    };

    let shipping = ShippingModel::create(
        &ctx.db,
        order.id,
        store.id,
        &params,
        provider_name,
        provider_data,
    ).await?;

    // Atualiza fulfillment_status do pedido para 'fulfilled'
    OrderModel::update_fulfillment_status(&ctx.db, order.id, "fulfilled").await?;

    format::json(ApiResponse::success(serde_json::json!({
        "pid": shipping.pid.to_string(),
        "carrier": shipping.carrier,
        "service": shipping.service,
        "tracking_code": shipping.tracking_code,
        "tracking_url": shipping.tracking_url,
        "status": shipping.status,
        "estimated_delivery_at": shipping.estimated_delivery_at.map(|t| t.to_string()),
    })))
}

/// PUT /api/painel/:store_pid/envios/:shipping_pid/status
/// Atualiza status de um envio (posted, in_transit, delivered, etc.).
#[debug_handler]
pub async fn update_shipping_status(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path((store_pid, shipping_pid)): Path<(Uuid, Uuid)>,
    Json(params): Json<UpdateShippingStatusParams>,
) -> Result<Response> {
    let (_, _, _) = require_collab(&ctx.db, &auth.claims.pid, &store_pid, true).await?;

    let shipping = ShippingModel::find_by_pid(&ctx.db, &shipping_pid).await?;
    let updated = ShippingModel::update_status(&ctx.db, shipping.id, &params).await?;

    // Se o envio foi entregue, atualiza fulfillment do pedido também
    if updated.status == "delivered" {
        let _ = OrderModel::update_fulfillment_status(&ctx.db, updated.order_id, "delivered").await;
    }

    format::json(ApiResponse::success(serde_json::json!({
        "pid": updated.pid.to_string(),
        "status": updated.status,
        "shipped_at": updated.shipped_at.map(|t| t.to_string()),
        "delivered_at": updated.delivered_at.map(|t| t.to_string()),
        "notes": updated.notes,
    })))
}

/// POST /api/painel/:store_pid/pedidos/:order_pid/frete
/// Calcula opções de frete usando um provider externo (ex.: MelhorEnvio).
#[debug_handler]
pub async fn calculate_freight(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path((store_pid, _order_pid)): Path<(Uuid, Uuid)>,
    Json(params): Json<CalculateFreightParams>,
) -> Result<Response> {
    let (_, _, _) = require_collab(&ctx.db, &auth.claims.pid, &store_pid, false).await?;

    let carrier = "melhor_envio"; // pode vir do body futuramente
    let Some(provider) = shipping::provider_for(carrier) else {
        return format::json(ApiResponse::success(serde_json::json!({
            "options": [],
            "message": format!("Provider '{}' não configurado. Defina MELHOR_ENVIO_TOKEN.", carrier),
        })));
    };

    let freight_params = shipping::FreightParams {
        origin_postal_code: params.origin_postal_code,
        destination_postal_code: params.destination_postal_code,
        weight_grams: params.weight_grams,
        length_cm: params.length_cm,
        width_cm: params.width_cm,
        height_cm: params.height_cm,
        declared_value_cents: params.declared_value_cents,
    };

    match provider.calculate_freight(freight_params).await {
        Ok(options) => format::json(ApiResponse::success(serde_json::json!({ "options": options }))),
        Err(e) => format::json(ApiResponse::<()>::error("FREIGHT_ERROR", &e.to_string())),
    }
}

/// GET /api/painel/:store_pid/envios
/// Lista todos os envios da loja com filtro de status.
#[debug_handler]
pub async fn list_shippings(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(store_pid): Path<Uuid>,
    Query(query): Query<OrderListQuery>,
) -> Result<Response> {
    let (_, store, _) = require_collab(&ctx.db, &auth.claims.pid, &store_pid, false).await?;

    let limit = query.limit.unwrap_or(20).min(100);
    let shippings = ShippingModel::list_for_store(
        &ctx.db,
        store.id,
        query.status.as_deref(),
        query.cursor,
        limit,
    ).await?;

    let has_more = shippings.len() as u64 >= limit;
    let cursor = shippings.last().map(|s| s.id.to_string());
    let count = shippings.len();

    let data: Vec<serde_json::Value> = shippings.into_iter().map(|s| serde_json::json!({
        "pid": s.pid.to_string(),
        "order_id": s.order_id,
        "carrier": s.carrier,
        "service": s.service,
        "tracking_code": s.tracking_code,
        "tracking_url": s.tracking_url,
        "status": s.status,
        "shipped_at": s.shipped_at.map(|t| t.to_string()),
        "estimated_delivery_at": s.estimated_delivery_at.map(|t| t.to_string()),
        "delivered_at": s.delivered_at.map(|t| t.to_string()),
        "notes": s.notes,
    })).collect();

    format::json(ApiResponse::paginated(data, cursor, has_more, count))
}

// ── Colaboradores ─────────────────────────────────────────────────────────────

/// GET /api/painel/:store_pid/colaboradores
#[debug_handler]
pub async fn list_collaborators(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(store_pid): Path<Uuid>,
) -> Result<Response> {
    let (_, store, collab) = require_collab(&ctx.db, &auth.claims.pid, &store_pid, false).await?;

    if !collab.can_manage_collaborators() {
        return Err(loco_rs::Error::Unauthorized("Apenas owner/admin podem gerenciar colaboradores".into()));
    }

    let collabs = CollaboratorModel::list_for_store(&ctx.db, store.id).await?;
    format::json(ApiResponse::success(collabs))
}

/// POST /api/painel/:store_pid/colaboradores
#[debug_handler]
pub async fn add_collaborator(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(store_pid): Path<Uuid>,
    Json(params): Json<crate::models::store_collaborators::AddCollaboratorParams>,
) -> Result<Response> {
    let (_, store, collab) = require_collab(&ctx.db, &auth.claims.pid, &store_pid, false).await?;

    if !collab.can_manage_collaborators() {
        return Err(loco_rs::Error::Unauthorized("Apenas owner/admin podem adicionar colaboradores".into()));
    }

    let new_collab = CollaboratorModel::add_collaborator(&ctx.db, store.id, &params).await?;
    format::json(ApiResponse::success(new_collab))
}

/// DELETE /api/painel/:store_pid/colaboradores/:user_id
#[debug_handler]
pub async fn remove_collaborator(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path((store_pid, user_id)): Path<(Uuid, i32)>,
) -> Result<Response> {
    let (_, store, collab) = require_collab(&ctx.db, &auth.claims.pid, &store_pid, false).await?;

    if !collab.can_manage_collaborators() {
        return Err(loco_rs::Error::Unauthorized("Apenas owner/admin podem remover colaboradores".into()));
    }

    CollaboratorModel::deactivate(&ctx.db, store.id, user_id).await?;
    format::json(ApiResponse::<()>::success(()))
}

// ── Roteamento ────────────────────────────────────────────────────────────────

pub fn routes() -> Routes {
    Routes::new()
        // Auth
        .add("/api/painel/auth/login", post(login))
        // Pedidos
        .add("/api/painel/{store_pid}/pedidos", get(list_orders))
        .add("/api/painel/{store_pid}/pedidos/{order_pid}", get(get_order))
        .add("/api/painel/{store_pid}/pedidos/{order_pid}/status", put(update_order_status))
        .add("/api/painel/{store_pid}/pedidos/{order_pid}/envio", post(create_shipping))
        .add("/api/painel/{store_pid}/pedidos/{order_pid}/frete", post(calculate_freight))
        // Envios
        .add("/api/painel/{store_pid}/envios", get(list_shippings))
        .add("/api/painel/{store_pid}/envios/{shipping_pid}/status", put(update_shipping_status))
        // Colaboradores
        .add("/api/painel/{store_pid}/colaboradores", get(list_collaborators))
        .add("/api/painel/{store_pid}/colaboradores", post(add_collaborator))
        .add("/api/painel/{store_pid}/colaboradores/{user_id}", delete(remove_collaborator))
}
