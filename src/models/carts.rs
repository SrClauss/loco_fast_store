
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use super::_entities::carts::{self, ActiveModel, Entity, Model};
pub use super::_entities::cart_items;

use loco_rs::prelude::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct AddToCartParams {
    pub variant_id: i32,
    pub quantity: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateCartItemParams {
    pub quantity: i32,
}

impl ActiveModelBehavior for ActiveModel {}
impl ActiveModelBehavior for cart_items::ActiveModel {}

impl Model {
    /// Cria um novo carrinho (pode ser anônimo via session_id)
    pub async fn create_cart(
        db: &DatabaseConnection,
        store_id: i32,
        session_id: &str,
        customer_id: Option<i32>,
        email: Option<String>,
        currency: Option<String>,
    ) -> ModelResult<Self> {
        let now = chrono::Utc::now();
        let cart = carts::ActiveModel {
            pid: ActiveValue::set(Uuid::new_v4()),
            store_id: ActiveValue::set(store_id),
            customer_id: ActiveValue::set(customer_id),
            session_id: ActiveValue::set(session_id.to_string()),
            status: ActiveValue::set("active".to_string()),
            email: ActiveValue::set(email),
            currency: ActiveValue::set(currency.unwrap_or_else(|| "BRL".to_string())),
            subtotal: ActiveValue::set(0),
            tax: ActiveValue::set(0),
            shipping: ActiveValue::set(0),
            total: ActiveValue::set(0),
            metadata: ActiveValue::set(serde_json::json!({})),
            last_activity_at: ActiveValue::set(now.into()),
            recovery_token: ActiveValue::set(Some(Uuid::new_v4().to_string())),
            ..Default::default()
        };
        let cart = cart.insert(db).await?;
        Ok(cart)
    }

    /// Busca carrinho ativo por session_id
    pub async fn find_active_by_session(
        db: &DatabaseConnection,
        store_id: i32,
        session_id: &str,
    ) -> ModelResult<Option<Self>> {
        let cart = Entity::find()
            .filter(carts::Column::StoreId.eq(store_id))
            .filter(carts::Column::SessionId.eq(session_id))
            .filter(carts::Column::Status.eq("active"))
            .one(db)
            .await?;
        Ok(cart)
    }

    /// Busca pelo PID
    pub async fn find_by_pid(db: &DatabaseConnection, pid: &Uuid) -> ModelResult<Self> {
        let cart = Entity::find()
            .filter(carts::Column::Pid.eq(*pid))
            .one(db)
            .await?;
        cart.ok_or_else(|| ModelError::EntityNotFound)
    }

    /// Adiciona item ao carrinho
    pub async fn add_item(
        db: &DatabaseConnection,
        cart_id: i32,
        variant_id: i32,
        quantity: i32,
        unit_price: i64,
    ) -> ModelResult<cart_items::Model> {
        // Verifica se já existe item com essa variante
        let existing = cart_items::Entity::find()
            .filter(cart_items::Column::CartId.eq(cart_id))
            .filter(cart_items::Column::VariantId.eq(variant_id))
            .one(db)
            .await?;

        if let Some(existing_item) = existing {
            // Atualiza quantidade
            let new_qty = existing_item.quantity + quantity;
            let mut active: cart_items::ActiveModel = existing_item.into();
            active.quantity = ActiveValue::set(new_qty);
            active.total = ActiveValue::set(unit_price * new_qty as i64);
            let updated = active.update(db).await?;
            return Ok(updated);
        }

        let item = cart_items::ActiveModel {
            pid: ActiveValue::set(Uuid::new_v4()),
            cart_id: ActiveValue::set(cart_id),
            variant_id: ActiveValue::set(variant_id),
            quantity: ActiveValue::set(quantity),
            unit_price: ActiveValue::set(unit_price),
            total: ActiveValue::set(unit_price * quantity as i64),
            metadata: ActiveValue::set(serde_json::json!({})),
            ..Default::default()
        };
        let item = item.insert(db).await?;
        Ok(item)
    }

    /// Atualiza quantidade de um item
    pub async fn update_item_quantity(
        db: &DatabaseConnection,
        item_id: i32,
        quantity: i32,
    ) -> ModelResult<cart_items::Model> {
        let item = cart_items::Entity::find_by_id(item_id)
            .one(db)
            .await?
            .ok_or(ModelError::EntityNotFound)?;

        let mut active: cart_items::ActiveModel = item.into();
        active.quantity = ActiveValue::set(quantity);
        // Recalcula total com unit_price existente
        let unit_price_val = cart_items::Entity::find_by_id(item_id)
            .one(db)
            .await?
            .map(|i| i.unit_price)
            .unwrap_or(0);
        active.total = ActiveValue::set(unit_price_val * quantity as i64);
        let updated = active.update(db).await?;
        Ok(updated)
    }

    /// Remove item do carrinho
    pub async fn remove_item(db: &DatabaseConnection, item_id: i32) -> ModelResult<()> {
        cart_items::Entity::delete_by_id(item_id).exec(db).await?;
        Ok(())
    }

    /// Lista itens do carrinho
    pub async fn get_items(db: &DatabaseConnection, cart_id: i32) -> ModelResult<Vec<cart_items::Model>> {
        let items = cart_items::Entity::find()
            .filter(cart_items::Column::CartId.eq(cart_id))
            .all(db)
            .await?;
        Ok(items)
    }

    /// Recalcula totais do carrinho
    pub async fn recalculate_totals(db: &DatabaseConnection, cart_id: i32) -> ModelResult<Self> {
        let items = Self::get_items(db, cart_id).await?;
        let subtotal: i64 = items.iter().map(|i| i.total).sum();

        let cart = Entity::find_by_id(cart_id)
            .one(db)
            .await?
            .ok_or(ModelError::EntityNotFound)?;

        let total = subtotal + cart.tax + cart.shipping;
        let mut active: carts::ActiveModel = cart.into();
        active.subtotal = ActiveValue::set(subtotal);
        active.total = ActiveValue::set(total);
        active.last_activity_at = ActiveValue::set(chrono::Utc::now().into());
        let updated = active.update(db).await?;
        Ok(updated)
    }

    /// Marca carrinho como completed
    pub async fn complete(db: &DatabaseConnection, cart_id: i32) -> ModelResult<Self> {
        let cart = Entity::find_by_id(cart_id)
            .one(db)
            .await?
            .ok_or(ModelError::EntityNotFound)?;

        let mut active: carts::ActiveModel = cart.into();
        active.status = ActiveValue::set("completed".to_string());
        active.completed_at = ActiveValue::set(Some(chrono::Utc::now().into()));
        let updated = active.update(db).await?;
        Ok(updated)
    }

    /// Busca carrinhos abandonados (sem atividade a mais de X minutos)
    pub async fn find_abandoned(
        db: &DatabaseConnection,
        store_id: i32,
        minutes_threshold: i64,
    ) -> ModelResult<Vec<Self>> {
        let threshold = chrono::Utc::now() - chrono::Duration::minutes(minutes_threshold);
        let carts = Entity::find()
            .filter(carts::Column::StoreId.eq(store_id))
            .filter(carts::Column::Status.eq("active"))
            .filter(carts::Column::LastActivityAt.lt(threshold))
            .filter(carts::Column::Email.is_not_null())
            .all(db)
            .await?;
        Ok(carts)
    }

    /// Associa customer ao carrinho (quando faz login ou se identifica)
    pub async fn attach_customer(
        db: &DatabaseConnection,
        cart_id: i32,
        customer_id: i32,
        email: &str,
    ) -> ModelResult<Self> {
        let cart = Entity::find_by_id(cart_id)
            .one(db)
            .await?
            .ok_or(ModelError::EntityNotFound)?;

        let mut active: carts::ActiveModel = cart.into();
        active.customer_id = ActiveValue::set(Some(customer_id));
        active.email = ActiveValue::set(Some(email.to_string()));
        let updated = active.update(db).await?;
        Ok(updated)
    }
}
