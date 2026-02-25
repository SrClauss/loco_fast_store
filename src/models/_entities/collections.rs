//! `SeaORM` Entity for Collections

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "collections")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    pub pid: Uuid,
    pub store_id: i32,
    pub title: String,
    pub slug: String,
    pub description: String,
    pub image_url: Option<String>,
    pub published: bool,
    pub sort_order: i32,
    pub metadata: Json,
    pub deleted_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::stores::Entity",
        from = "Column::StoreId",
        to = "super::stores::Column::Id"
    )]
    Store,
}

impl Related<super::stores::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Store.def()
    }
}

impl Related<super::products::Entity> for Entity {
    fn to() -> RelationDef {
        super::collection_products::Relation::Product.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::collection_products::Relation::Collection.def().rev())
    }
}
