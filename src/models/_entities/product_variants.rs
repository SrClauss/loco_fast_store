//! `SeaORM` Entity for ProductVariants

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "product_variants")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    pub pid: Uuid,
    pub product_id: i32,
    #[sea_orm(unique)]
    pub sku: String,
    pub title: String,
    pub option_values: Json,
    pub inventory_quantity: i32,
    pub allow_backorder: bool,
    #[sea_orm(column_type = "Decimal(Some((10, 2)))")]
    pub weight: Option<Decimal>,
    pub dimensions: Option<Json>,
    pub metadata: Json,
    pub sort_order: i32,
    pub deleted_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::products::Entity",
        from = "Column::ProductId",
        to = "super::products::Column::Id"
    )]
    Product,
    #[sea_orm(has_many = "super::prices::Entity")]
    Prices,
    #[sea_orm(has_many = "super::product_images::Entity")]
    Images,
}

impl Related<super::products::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Product.def()
    }
}

impl Related<super::prices::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Prices.def()
    }
}

impl Related<super::product_images::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Images.def()
    }
}
