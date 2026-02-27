use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "stocks")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    pub warehouse_id: i32,
    pub item_id: i32,
    pub quantity: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::warehouses::Entity", from = "Column::WarehouseId", to = "super::warehouses::Column::Id")]
    Warehouse,
    #[sea_orm(belongs_to = "super::items::Entity", from = "Column::ItemId", to = "super::items::Column::Id")]
    Item,
}
