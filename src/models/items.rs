use sea_orm::{ActiveModelBehavior, ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use loco_rs::prelude::*;

pub use super::_entities::items::{self, ActiveModel, Entity, Model};

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateItemParams {
    pub variant_id: i32,
    pub batch: Option<String>,
    pub expiration: Option<chrono::NaiveDate>,
}

impl Model {
    pub async fn create_item(
        db: &sea_orm::DatabaseConnection,
        params: &CreateItemParams,
    ) -> loco_rs::Result<Self> {
        let item = items::ActiveModel {
            pid: sea_orm::ActiveValue::set(Uuid::new_v4()),
            variant_id: sea_orm::ActiveValue::set(params.variant_id),
            batch: sea_orm::ActiveValue::set(params.batch.clone()),
            expiration: sea_orm::ActiveValue::set(params.expiration),
            ..Default::default()
        };
        Ok(item.insert(db).await?)
    }

    pub async fn find_by_pid(
        db: &sea_orm::DatabaseConnection,
        pid: &Uuid,
    ) -> loco_rs::Result<Option<Self>> {
        Ok(Entity::find()
            .filter(items::Column::Pid.eq(*pid))
            .filter(items::Column::DeletedAt.is_null())
            .one(db)
            .await?)
    }
}
