use axum::extract::Query;
use loco_rs::prelude::*;
use sea_orm::{EntityTrait, PaginatorTrait};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    dto::{
        entities::{CustomerResponse, AddressResponse},
        response::ApiResponse,
    },
    models::{
        _entities::users,
        customers::{CreateCustomerParams, UpdateCustomerParams, CreateAddressParams},
    },
};

#[derive(Debug, Deserialize)]
pub struct CustomerListQuery {
    pub cursor: Option<i32>,
    pub limit: Option<u64>,
    pub email: Option<String>,
}

/// POST /api/stores/:store_pid/customers
#[debug_handler]
async fn create(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(store_pid): Path<Uuid>,
    Json(params): Json<CreateCustomerParams>,
) -> Result<Response> {
    let _user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let store = crate::models::stores::Model::find_by_pid(&ctx.db, &store_pid).await?;
    let customer =
        crate::models::customers::Model::create_customer(&ctx.db, store.id, &params).await?;
    format::json(ApiResponse::success(CustomerResponse::from(customer)))
}

/// GET /api/stores/:store_pid/customers
#[debug_handler]
async fn list(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(store_pid): Path<Uuid>,
    Query(query): Query<CustomerListQuery>,
) -> Result<Response> {
    let _user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let store = crate::models::stores::Model::find_by_pid(&ctx.db, &store_pid).await?;

    // Se buscar por email diretamente
    if let Some(email) = query.email {
        let customer =
            crate::models::customers::Model::find_by_email(&ctx.db, store.id, &email).await?;
        return format::json(ApiResponse::success(vec![CustomerResponse::from(customer)]));
    }

    let limit = query.limit.unwrap_or(20);
    let customers = crate::models::customers::Model::list_for_store(
        &ctx.db,
        store.id,
        query.cursor,
        limit,
    )
    .await?;

    let has_more = customers.len() as u64 >= limit.min(100);
    let cursor = customers.last().map(|c| c.id.to_string());
    let count = customers.len();

    let response: Vec<CustomerResponse> =
        customers.into_iter().map(CustomerResponse::from).collect();
    format::json(ApiResponse::paginated(response, cursor, has_more, count))
}

/// GET /api/stores/:store_pid/customers/:pid
#[debug_handler]
async fn get_one(
    State(ctx): State<AppContext>,
    Path((_store_pid, pid)): Path<(Uuid, Uuid)>,
) -> Result<Response> {
    let customer = crate::models::customers::Model::find_by_pid(&ctx.db, &pid).await?;
    format::json(ApiResponse::success(CustomerResponse::from(customer)))
}

/// PUT /api/stores/:store_pid/customers/:pid
#[debug_handler]
async fn update(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path((_store_pid, pid)): Path<(Uuid, Uuid)>,
    Json(params): Json<UpdateCustomerParams>,
) -> Result<Response> {
    let _user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let customer = crate::models::customers::Model::find_by_pid(&ctx.db, &pid).await?;

    let mut active: crate::models::_entities::customers::ActiveModel = customer.into();
    if let Some(first_name) = params.first_name {
        active.first_name = ActiveValue::set(first_name);
    }
    if let Some(last_name) = params.last_name {
        active.last_name = ActiveValue::set(last_name);
    }
    if let Some(phone) = params.phone {
        active.phone = ActiveValue::set(Some(phone));
    }
    if let Some(marketing_consent) = params.marketing_consent {
        active.marketing_consent = ActiveValue::set(marketing_consent);
    }
    if let Some(metadata) = params.metadata {
        active.metadata = ActiveValue::set(metadata);
    }

    let updated = active.update(&ctx.db).await?;
    format::json(ApiResponse::success(CustomerResponse::from(updated)))
}

/// POST /api/stores/:store_pid/customers/:pid/addresses
#[debug_handler]
async fn add_address(
    State(ctx): State<AppContext>,
    Path((_store_pid, pid)): Path<(Uuid, Uuid)>,
    Json(params): Json<CreateAddressParams>,
) -> Result<Response> {
    let customer = crate::models::customers::Model::find_by_pid(&ctx.db, &pid).await?;
    let address =
        crate::models::customers::Model::add_address(&ctx.db, customer.id, &params).await?;
    format::json(ApiResponse::success(AddressResponse::from(address)))
}

/// GET /api/stores/:store_pid/customers/:pid/addresses
#[debug_handler]
async fn list_addresses(
    State(ctx): State<AppContext>,
    Path((_store_pid, pid)): Path<(Uuid, Uuid)>,
) -> Result<Response> {
    let customer = crate::models::customers::Model::find_by_pid(&ctx.db, &pid).await?;
    let addresses =
        crate::models::customers::Model::get_addresses(&ctx.db, customer.id).await?;
    let response: Vec<AddressResponse> =
        addresses.into_iter().map(AddressResponse::from).collect();
    format::json(ApiResponse::success(response))
}

/// Stats para painel admin
#[derive(Debug, Serialize)]
pub struct CustomerStats {
    pub total: u64,
    pub active: u64,
    pub with_orders: u64,
}

/// GET /api/admin/customers/stats
#[debug_handler]
async fn admin_stats(
    State(ctx): State<AppContext>,
) -> Result<Response> {
    use crate::models::_entities::customers;
    
    let total = customers::Entity::find().count(&ctx.db).await?;
    let active = total; // Simplificado por enquanto
    let with_orders = 0; // TODO: implementar query de customers com pedidos
    
    let stats = CustomerStats {
        total,
        active,
        with_orders,
    };
    
    format::json(stats)
}

/// GET /api/admin/customers
#[debug_handler]
async fn admin_list(
    State(ctx): State<AppContext>,
) -> Result<Response> {
    use crate::models::_entities::customers;
    
    let customers_list = customers::Entity::find()
        .all(&ctx.db)
        .await?;
    let response: Vec<CustomerResponse> =
        customers_list.into_iter().map(CustomerResponse::from).collect();
    format::json(ApiResponse::success(response))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/stores/{store_pid}/customers")
        .add("/", post(create))
        .add("/", get(list))
        .add("/{pid}", get(get_one))
        .add("/{pid}", put(update))
        .add("/{pid}/addresses", post(add_address))
        .add("/{pid}/addresses", get(list_addresses))
}

pub fn admin_routes() -> Routes {
    Routes::new()
        .prefix("/api/admin")
        .add("/customers/stats", get(admin_stats))
        .add("/customers", get(admin_list))
}
