//! `SeaORM` Entity for Carts

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "carts")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    pub pid: Uuid,
    pub store_id: i32,
    pub customer_id: Option<i32>,
    pub session_id: String,
    pub status: String,
    pub email: Option<String>,
    pub currency: String,
    pub subtotal: i64,
    pub tax: i64,
    pub shipping: i64,
    pub total: i64,
    pub metadata: Json,
    pub expires_at: Option<DateTimeWithTimeZone>,
    pub completed_at: Option<DateTimeWithTimeZone>,
    pub last_activity_at: DateTimeWithTimeZone,
    pub recovery_token: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::stores::Entity",
        from = "Column::StoreId",
        to = "super::stores::Column::Id"
    )]
    Store,
    #[sea_orm(
        belongs_to = "super::customers::Entity",
        from = "Column::CustomerId",
        to = "super::customers::Column::Id"
    )]
    Customer,
    #[sea_orm(has_many = "super::cart_items::Entity")]
    Items,
}

impl Related<super::stores::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Store.def()
    }
}

impl Related<super::customers::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Customer.def()
    }
}

impl Related<super::cart_items::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Items.def()
    }
}
