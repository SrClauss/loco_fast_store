use sea_orm::{QueryOrder, QuerySelect};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use super::_entities::customers::{self, ActiveModel, Entity, Model};
pub use super::_entities::addresses;

use loco_rs::prelude::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateCustomerParams {
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub phone: Option<String>,
    pub has_account: Option<bool>,
    pub user_id: Option<i32>,
    pub marketing_consent: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateCustomerParams {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
    pub marketing_consent: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateAddressParams {
    pub first_name: String,
    pub last_name: String,
    pub company: Option<String>,
    pub address_line_1: String,
    pub address_line_2: Option<String>,
    pub city: String,
    pub state: String,
    pub postal_code: String,
    pub country: Option<String>,
    pub phone: Option<String>,
    pub is_default_shipping: Option<bool>,
    pub is_default_billing: Option<bool>,
}

impl ActiveModelBehavior for ActiveModel {}
impl ActiveModelBehavior for addresses::ActiveModel {}

impl Model {
    /// Cria um novo cliente na loja
    pub async fn create_customer(
        db: &DatabaseConnection,
        store_id: i32,
        params: &CreateCustomerParams,
    ) -> ModelResult<Self> {
        let customer = customers::ActiveModel {
            pid: ActiveValue::set(Uuid::new_v4()),
            store_id: ActiveValue::set(store_id),
            email: ActiveValue::set(params.email.clone()),
            first_name: ActiveValue::set(params.first_name.clone()),
            last_name: ActiveValue::set(params.last_name.clone()),
            phone: ActiveValue::set(params.phone.clone()),
            has_account: ActiveValue::set(params.has_account.unwrap_or(false)),
            user_id: ActiveValue::set(params.user_id),
            marketing_consent: ActiveValue::set(params.marketing_consent.unwrap_or(false)),
            metadata: ActiveValue::set(serde_json::json!({})),
            ..Default::default()
        };
        let customer = customer.insert(db).await?;
        Ok(customer)
    }

    /// Busca ou cria customer anônimo (checkout sem conta)
    pub async fn find_or_create_anonymous(
        db: &DatabaseConnection,
        store_id: i32,
        email: &str,
    ) -> ModelResult<Self> {
        let existing = Entity::find()
            .filter(customers::Column::StoreId.eq(store_id))
            .filter(customers::Column::Email.eq(email))
            .filter(customers::Column::DeletedAt.is_null())
            .one(db)
            .await?;

        if let Some(customer) = existing {
            return Ok(customer);
        }

        let params = CreateCustomerParams {
            email: email.to_string(),
            first_name: String::new(),
            last_name: String::new(),
            phone: None,
            has_account: Some(false),
            user_id: None,
            marketing_consent: None,
        };
        Self::create_customer(db, store_id, &params).await
    }

    /// Busca pelo PID
    pub async fn find_by_pid(db: &DatabaseConnection, pid: &Uuid) -> ModelResult<Self> {
        let customer = Entity::find()
            .filter(customers::Column::Pid.eq(*pid))
            .filter(customers::Column::DeletedAt.is_null())
            .one(db)
            .await?;
        customer.ok_or_else(|| ModelError::EntityNotFound)
    }

    /// Busca por email na loja
    pub async fn find_by_email(
        db: &DatabaseConnection,
        store_id: i32,
        email: &str,
    ) -> ModelResult<Self> {
        let customer = Entity::find()
            .filter(customers::Column::StoreId.eq(store_id))
            .filter(customers::Column::Email.eq(email))
            .filter(customers::Column::DeletedAt.is_null())
            .one(db)
            .await?;
        customer.ok_or_else(|| ModelError::EntityNotFound)
    }

    /// Lista clientes da loja
    pub async fn list_for_store(
        db: &DatabaseConnection,
        store_id: i32,
        cursor: Option<i32>,
        limit: u64,
    ) -> ModelResult<Vec<Self>> {
        let mut query = Entity::find()
            .filter(customers::Column::StoreId.eq(store_id))
            .filter(customers::Column::DeletedAt.is_null());

        if let Some(cursor_id) = cursor {
            query = query.filter(customers::Column::Id.gt(cursor_id));
        }

        let customers = query
            .order_by_asc(customers::Column::Id)
            .limit(limit.min(100))
            .all(db)
            .await?;
        Ok(customers)
    }

    /// Adiciona um endereço ao customer
    pub async fn add_address(
        db: &DatabaseConnection,
        customer_id: i32,
        params: &CreateAddressParams,
    ) -> ModelResult<addresses::Model> {
        let address = addresses::ActiveModel {
            pid: ActiveValue::set(Uuid::new_v4()),
            customer_id: ActiveValue::set(customer_id),
            first_name: ActiveValue::set(params.first_name.clone()),
            last_name: ActiveValue::set(params.last_name.clone()),
            company: ActiveValue::set(params.company.clone()),
            address_line_1: ActiveValue::set(params.address_line_1.clone()),
            address_line_2: ActiveValue::set(params.address_line_2.clone()),
            city: ActiveValue::set(params.city.clone()),
            state: ActiveValue::set(params.state.clone()),
            postal_code: ActiveValue::set(params.postal_code.clone()),
            country: ActiveValue::set(
                params.country.clone().unwrap_or_else(|| "BR".to_string()),
            ),
            phone: ActiveValue::set(params.phone.clone()),
            is_default_shipping: ActiveValue::set(params.is_default_shipping.unwrap_or(false)),
            is_default_billing: ActiveValue::set(params.is_default_billing.unwrap_or(false)),
            ..Default::default()
        };
        let address = address.insert(db).await?;
        Ok(address)
    }

    /// Lista endereços de um customer
    pub async fn get_addresses(
        db: &DatabaseConnection,
        customer_id: i32,
    ) -> ModelResult<Vec<addresses::Model>> {
        let addrs = addresses::Entity::find()
            .filter(addresses::Column::CustomerId.eq(customer_id))
            .all(db)
            .await?;
        Ok(addrs)
    }

    /// Atualiza analytics_session_id e last_seen_at
    pub async fn update_analytics_session(
        db: &DatabaseConnection,
        customer_id: i32,
        session_id: &str,
    ) -> ModelResult<Self> {
        let mut active: customers::ActiveModel = Entity::find_by_id(customer_id)
            .one(db)
            .await?
            .ok_or(ModelError::EntityNotFound)?
            .into();

        active.analytics_session_id = ActiveValue::set(Some(session_id.to_string()));
        active.last_seen_at = ActiveValue::set(Some(chrono::Utc::now().into()));
        let updated = active.update(db).await?;
        Ok(updated)
    }
}
