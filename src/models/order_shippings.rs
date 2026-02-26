use chrono::Utc;
use sea_orm::{QueryOrder, QuerySelect};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use super::_entities::order_shippings::{self, ActiveModel, Entity, Model};
use loco_rs::prelude::*;

/// Parâmetros para registrar ou atualizar um envio manualmente
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateShippingParams {
    pub carrier: String,
    pub service: Option<String>,
    pub tracking_code: Option<String>,
    pub tracking_url: Option<String>,
    pub estimated_delivery_at: Option<String>,
    pub notes: Option<String>,
}

/// Parâmetros para atualizar o status de um envio
#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateShippingStatusParams {
    pub status: String,
    pub notes: Option<String>,
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Cria registro de envio para um pedido
    pub async fn create(
        db: &DatabaseConnection,
        order_id: i32,
        params: &CreateShippingParams,
        provider: Option<&str>,
        provider_data: Option<serde_json::Value>,
    ) -> ModelResult<Self> {
        let estimated = params
            .estimated_delivery_at
            .as_deref()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::FixedOffset::east_opt(0).unwrap()));

        let shipping = order_shippings::ActiveModel {
            pid: ActiveValue::set(Uuid::new_v4()),
            order_id: ActiveValue::set(order_id),
            carrier: ActiveValue::set(params.carrier.clone()),
            service: ActiveValue::set(params.service.clone()),
            tracking_code: ActiveValue::set(params.tracking_code.clone()),
            tracking_url: ActiveValue::set(params.tracking_url.clone()),
            status: ActiveValue::set("pending".to_string()),
            estimated_delivery_at: ActiveValue::set(estimated),
            provider: ActiveValue::set(provider.map(String::from)),
            provider_data: ActiveValue::set(provider_data.unwrap_or_else(|| serde_json::json!({}))),
            notes: ActiveValue::set(params.notes.clone()),
            ..Default::default()
        };
        let saved = shipping.insert(db).await?;
        Ok(saved)
    }

    /// Busca envio pelo PID
    pub async fn find_by_pid(db: &DatabaseConnection, pid: &Uuid) -> ModelResult<Self> {
        Entity::find()
            .filter(order_shippings::Column::Pid.eq(*pid))
            .one(db)
            .await?
            .ok_or_else(|| ModelError::EntityNotFound)
    }

    /// Busca o envio ativo de um pedido
    pub async fn find_by_order(
        db: &DatabaseConnection,
        order_id: i32,
    ) -> ModelResult<Option<Self>> {
        let shipping = Entity::find()
            .filter(order_shippings::Column::OrderId.eq(order_id))
            .order_by_desc(order_shippings::Column::CreatedAt)
            .one(db)
            .await?;
        Ok(shipping)
    }

    /// Lista envios com filtro opcional de status
    pub async fn list_for_store(
        db: &DatabaseConnection,
        status: Option<&str>,
        cursor: Option<i32>,
        limit: u64,
    ) -> ModelResult<Vec<Self>> {
        let mut q = Entity::find();

        if let Some(s) = status {
            q = q.filter(order_shippings::Column::Status.eq(s));
        }
        if let Some(c) = cursor {
            q = q.filter(order_shippings::Column::Id.gt(c));
        }

        let shippings = q
            .order_by_desc(order_shippings::Column::CreatedAt)
            .limit(limit.min(100))
            .all(db)
            .await?;
        Ok(shippings)
    }

    /// Atualiza status de envio.
    /// Transições permitidas: pending → posted → in_transit → out_for_delivery → delivered
    ///                                                                           ↘ failed / returned
    pub async fn update_status(
        db: &DatabaseConnection,
        shipping_id: i32,
        params: &UpdateShippingStatusParams,
    ) -> ModelResult<Self> {
        let shipping = Entity::find_by_id(shipping_id)
            .one(db)
            .await?
            .ok_or(ModelError::EntityNotFound)?;

        let mut active: order_shippings::ActiveModel = shipping.into();
        active.status = ActiveValue::set(params.status.clone());

        let now: chrono::DateTime<chrono::FixedOffset> =
            Utc::now().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap());

        match params.status.as_str() {
            "posted" => {
                active.shipped_at = ActiveValue::set(Some(now));
            }
            "delivered" => {
                active.delivered_at = ActiveValue::set(Some(now));
            }
            _ => {}
        }

        if let Some(notes) = &params.notes {
            active.notes = ActiveValue::set(Some(notes.clone()));
        }

        let updated = active.update(db).await?;
        Ok(updated)
    }

    /// Atualiza dados do provider externo (ex.: resposta do MelhorEnvio)
    pub async fn update_provider_data(
        db: &DatabaseConnection,
        shipping_id: i32,
        provider: &str,
        data: serde_json::Value,
    ) -> ModelResult<Self> {
        let shipping = Entity::find_by_id(shipping_id)
            .one(db)
            .await?
            .ok_or(ModelError::EntityNotFound)?;

        let mut active: order_shippings::ActiveModel = shipping.into();
        active.provider = ActiveValue::set(Some(provider.to_string()));
        active.provider_data = ActiveValue::set(data);
        let updated = active.update(db).await?;
        Ok(updated)
    }
}
