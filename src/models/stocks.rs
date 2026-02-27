use sea_orm::{ActiveModelBehavior, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

use loco_rs::prelude::*;

pub use super::_entities::stocks::{self, ActiveModel, Entity, Model};

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateStockParams {
    pub warehouse_id: i32,
    pub item_id: i32,
    pub quantity: i32,
}

impl Model {
    pub async fn set_stock(
        db: &sea_orm::DatabaseConnection,
        params: &UpdateStockParams,
    ) -> loco_rs::Result<Self> {
        // upsert behaviour
        let existing = Entity::find()
            .filter(stocks::Column::WarehouseId.eq(params.warehouse_id))
            .filter(stocks::Column::ItemId.eq(params.item_id))
            .one(db)
            .await?;
        if let Some(e) = existing {
            let mut am: stocks::ActiveModel = e.into();
            am.quantity = sea_orm::ActiveValue::set(params.quantity);
            Ok(am.update(db).await?)
        } else {
            let am = stocks::ActiveModel {
                warehouse_id: sea_orm::ActiveValue::set(params.warehouse_id),
                item_id: sea_orm::ActiveValue::set(params.item_id),
                quantity: sea_orm::ActiveValue::set(params.quantity),
                ..Default::default()
            };
            Ok(am.insert(db).await?)
        }
    }
}
