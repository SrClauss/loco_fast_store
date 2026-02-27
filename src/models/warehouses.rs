use sea_orm::{ActiveModelBehavior, ColumnTrait, EntityTrait, QueryFilter};
use loco_rs::model::ModelError;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use loco_rs::prelude::*;

pub use super::_entities::warehouses::{self, ActiveModel, Entity, Model};

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateWarehouseParams {
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
}

impl Model {
    pub async fn create_warehouse(
        db: &sea_orm::DatabaseConnection,
        params: &CreateWarehouseParams,
    ) -> loco_rs::Result<Self> {
        let warehouse = warehouses::ActiveModel {
            pid: sea_orm::ActiveValue::set(Uuid::new_v4()),
            name: sea_orm::ActiveValue::set(params.name.clone()),
            latitude: sea_orm::ActiveValue::set(params.latitude),
            longitude: sea_orm::ActiveValue::set(params.longitude),
            ..Default::default()
        };
        Ok(warehouse.insert(db).await?)
    }

    pub async fn find_by_pid(
        db: &sea_orm::DatabaseConnection,
        pid: &Uuid,
    ) -> ModelResult<Self> {
        let wh = Entity::find()
            .filter(warehouses::Column::Pid.eq(*pid))
            .filter(warehouses::Column::DeletedAt.is_null())
            .one(db)
            .await?;
        wh.ok_or_else(|| ModelError::EntityNotFound)
    }
}
