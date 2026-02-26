//! `SeaORM` Entity for Products

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "products")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    pub pid: Uuid,
    pub title: String,
    pub slug: String,
    pub description: String,
    pub handle: String,
    pub status: String,
    pub product_type: String,
    pub category_id: Option<i32>,
    pub tags: Json,
    pub metadata: Json,
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
    #[sea_orm(column_type = "Decimal(Some((10, 2)))")]
    pub weight: Option<Decimal>,
    pub dimensions: Option<Json>,
    pub featured: bool,
    pub deleted_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::categories::Entity",
        from = "Column::CategoryId",
        to = "super::categories::Column::Id"
    )]
    Category,
    #[sea_orm(has_many = "super::product_variants::Entity")]
    Variants,
    #[sea_orm(has_many = "super::product_images::Entity")]
    Images,
}

impl Related<super::categories::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Category.def()
    }
}

impl Related<super::product_variants::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Variants.def()
    }
}

impl Related<super::product_images::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Images.def()
    }
}

impl Related<super::collections::Entity> for Entity {
    fn to() -> RelationDef {
        super::collection_products::Relation::Collection.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::collection_products::Relation::Product.def().rev())
    }
}
