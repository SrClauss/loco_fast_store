use sea_orm::{PaginatorTrait, QueryOrder, QuerySelect};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use super::_entities::orders::{self, ActiveModel, Entity, Model};
pub use super::_entities::order_items;

use loco_rs::prelude::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateOrderFromCartParams {
    pub customer_id: i32,
    pub shipping_address_id: Option<i32>,
    pub billing_address_id: Option<i32>,
    pub payment_method: Option<String>,
    pub notes: Option<String>,
}

impl ActiveModelBehavior for ActiveModel {}
impl ActiveModelBehavior for order_items::ActiveModel {}

/// Gera número de pedido: LFS-{store_id}-{seq}
fn generate_order_number(store_id: i32, seq: i64) -> String {
    format!("LFS-{:04}-{:06}", store_id, seq)
}

impl Model {
    /// Cria pedido a partir de um carrinho
    pub async fn create_from_cart(
        db: &DatabaseConnection,
        store_id: i32,
        cart: &super::_entities::carts::Model,
        cart_items: &[super::_entities::cart_items::Model],
        params: &CreateOrderFromCartParams,
    ) -> ModelResult<Self> {
        // Conta pedidos existentes para gerar número
        let count = Entity::find()
            .filter(orders::Column::StoreId.eq(store_id))
            .count(db)
            .await?;

        let order_number = generate_order_number(store_id, count as i64 + 1);

        let order = orders::ActiveModel {
            pid: ActiveValue::set(Uuid::new_v4()),
            store_id: ActiveValue::set(store_id),
            customer_id: ActiveValue::set(params.customer_id),
            cart_id: ActiveValue::set(Some(cart.id)),
            order_number: ActiveValue::set(order_number),
            status: ActiveValue::set("pending".to_string()),
            payment_status: ActiveValue::set("awaiting".to_string()),
            fulfillment_status: ActiveValue::set("not_fulfilled".to_string()),
            currency: ActiveValue::set(cart.currency.clone()),
            subtotal: ActiveValue::set(cart.subtotal),
            tax: ActiveValue::set(cart.tax),
            shipping: ActiveValue::set(cart.shipping),
            discount: ActiveValue::set(0),
            total: ActiveValue::set(cart.total),
            shipping_address_id: ActiveValue::set(params.shipping_address_id),
            billing_address_id: ActiveValue::set(params.billing_address_id),
            payment_method: ActiveValue::set(params.payment_method.clone()),
            payment_data: ActiveValue::set(serde_json::json!({})),
            notes: ActiveValue::set(params.notes.clone()),
            metadata: ActiveValue::set(serde_json::json!({})),
            ..Default::default()
        };
        let order = order.insert(db).await?;

        // Cria itens do pedido (snapshot dos itens do carrinho)
        for item in cart_items {
            let order_item = order_items::ActiveModel {
                pid: ActiveValue::set(Uuid::new_v4()),
                order_id: ActiveValue::set(order.id),
                variant_id: ActiveValue::set(Some(item.variant_id)),
                title: ActiveValue::set(String::new()), // será preenchido pelo controller
                sku: ActiveValue::set(String::new()),
                quantity: ActiveValue::set(item.quantity),
                unit_price: ActiveValue::set(item.unit_price),
                total: ActiveValue::set(item.total),
                metadata: ActiveValue::set(serde_json::json!({})),
                ..Default::default()
            };
            order_item.insert(db).await?;
        }

        Ok(order)
    }

    /// Busca pelo PID
    pub async fn find_by_pid(db: &DatabaseConnection, pid: &Uuid) -> ModelResult<Self> {
        let order = Entity::find()
            .filter(orders::Column::Pid.eq(*pid))
            .one(db)
            .await?;
        order.ok_or_else(|| ModelError::EntityNotFound)
    }

    /// Busca pelo order_number
    pub async fn find_by_number(
        db: &DatabaseConnection,
        order_number: &str,
    ) -> ModelResult<Self> {
        let order = Entity::find()
            .filter(orders::Column::OrderNumber.eq(order_number))
            .one(db)
            .await?;
        order.ok_or_else(|| ModelError::EntityNotFound)
    }

    /// Lista pedidos da loja
    pub async fn list_for_store(
        db: &DatabaseConnection,
        store_id: i32,
        status: Option<&str>,
        cursor: Option<i32>,
        limit: u64,
    ) -> ModelResult<Vec<Self>> {
        let mut query = Entity::find()
            .filter(orders::Column::StoreId.eq(store_id));

        if let Some(s) = status {
            query = query.filter(orders::Column::Status.eq(s));
        }

        if let Some(cursor_id) = cursor {
            query = query.filter(orders::Column::Id.gt(cursor_id));
        }

        let orders = query
            .order_by_asc(orders::Column::Id)
            .limit(limit.min(100))
            .all(db)
            .await?;
        Ok(orders)
    }

    /// Lista pedidos de um customer
    pub async fn list_for_customer(
        db: &DatabaseConnection,
        customer_id: i32,
        cursor: Option<i32>,
        limit: u64,
    ) -> ModelResult<Vec<Self>> {
        let mut query = Entity::find()
            .filter(orders::Column::CustomerId.eq(customer_id));

        if let Some(cursor_id) = cursor {
            query = query.filter(orders::Column::Id.gt(cursor_id));
        }

        let orders = query
            .order_by_desc(orders::Column::CreatedAt)
            .limit(limit.min(100))
            .all(db)
            .await?;
        Ok(orders)
    }

    /// Obtém itens do pedido
    pub async fn get_items(
        db: &DatabaseConnection,
        order_id: i32,
    ) -> ModelResult<Vec<order_items::Model>> {
        let items = order_items::Entity::find()
            .filter(order_items::Column::OrderId.eq(order_id))
            .all(db)
            .await?;
        Ok(items)
    }

    /// Atualiza status do pedido
    pub async fn update_status(
        db: &DatabaseConnection,
        order_id: i32,
        status: &str,
    ) -> ModelResult<Self> {
        let order = Entity::find_by_id(order_id)
            .one(db)
            .await?
            .ok_or(ModelError::EntityNotFound)?;

        let mut active: orders::ActiveModel = order.into();
        active.status = ActiveValue::set(status.to_string());

        if status == "canceled" {
            active.canceled_at = ActiveValue::set(Some(chrono::Utc::now().into()));
        }

        let updated = active.update(db).await?;
        Ok(updated)
    }

    /// Atualiza status de pagamento
    pub async fn update_payment_status(
        db: &DatabaseConnection,
        order_id: i32,
        payment_status: &str,
        payment_data: Option<serde_json::Value>,
    ) -> ModelResult<Self> {
        let order = Entity::find_by_id(order_id)
            .one(db)
            .await?
            .ok_or(ModelError::EntityNotFound)?;

        let mut active: orders::ActiveModel = order.into();
        active.payment_status = ActiveValue::set(payment_status.to_string());

        if let Some(data) = payment_data {
            active.payment_data = ActiveValue::set(data);
        }

        if payment_status == "paid" {
            active.paid_at = ActiveValue::set(Some(chrono::Utc::now().into()));
        }

        let updated = active.update(db).await?;
        Ok(updated)
    }

    /// Atualiza status de fulfillment
    pub async fn update_fulfillment_status(
        db: &DatabaseConnection,
        order_id: i32,
        fulfillment_status: &str,
    ) -> ModelResult<Self> {
        let order = Entity::find_by_id(order_id)
            .one(db)
            .await?
            .ok_or(ModelError::EntityNotFound)?;

        let mut active: orders::ActiveModel = order.into();
        active.fulfillment_status = ActiveValue::set(fulfillment_status.to_string());
        let updated = active.update(db).await?;
        Ok(updated)
    }
}
