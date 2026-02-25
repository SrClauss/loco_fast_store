use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use super::_entities::stores::{self, ActiveModel, Entity, Model};

use loco_rs::prelude::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateStoreParams {
    pub slug: String,
    pub name: String,
    pub default_currency: Option<String>,
    pub config: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateStoreParams {
    pub name: Option<String>,
    pub domain: Option<String>,
    pub default_currency: Option<String>,
    pub config: Option<serde_json::Value>,
    pub status: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Cria uma nova loja vinculada ao owner
    pub async fn create_store(
        db: &DatabaseConnection,
        owner_id: i32,
        params: &CreateStoreParams,
    ) -> ModelResult<Self> {
        let store = stores::ActiveModel {
            pid: ActiveValue::set(Uuid::new_v4()),
            slug: ActiveValue::set(params.slug.clone()),
            name: ActiveValue::set(params.name.clone()),
            default_currency: ActiveValue::set(
                params.default_currency.clone().unwrap_or_else(|| "BRL".to_string()),
            ),
            config: ActiveValue::set(
                params.config.clone().unwrap_or(serde_json::json!({})),
            ),
            status: ActiveValue::set("draft".to_string()),
            metadata: ActiveValue::set(serde_json::json!({})),
            owner_id: ActiveValue::set(owner_id),
            ..Default::default()
        };
        let store = store.insert(db).await?;
        Ok(store)
    }

    /// Busca loja pelo PID (UUID público)
    pub async fn find_by_pid(db: &DatabaseConnection, pid: &Uuid) -> ModelResult<Self> {
        let store = Entity::find()
            .filter(stores::Column::Pid.eq(*pid))
            .filter(stores::Column::DeletedAt.is_null())
            .one(db)
            .await?;
        store.ok_or_else(|| ModelError::EntityNotFound)
    }

    /// Busca loja pelo slug
    pub async fn find_by_slug(db: &DatabaseConnection, slug: &str) -> ModelResult<Self> {
        let store = Entity::find()
            .filter(stores::Column::Slug.eq(slug))
            .filter(stores::Column::DeletedAt.is_null())
            .one(db)
            .await?;
        store.ok_or_else(|| ModelError::EntityNotFound)
    }

    /// Lista lojas do owner
    pub async fn find_by_owner(db: &DatabaseConnection, owner_id: i32) -> ModelResult<Vec<Self>> {
        let stores = Entity::find()
            .filter(stores::Column::OwnerId.eq(owner_id))
            .filter(stores::Column::DeletedAt.is_null())
            .all(db)
            .await?;
        Ok(stores)
    }
}
