//! `SeaORM` Entity for Orders

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "orders")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    pub pid: Uuid,
    pub customer_id: i32,
    pub cart_id: Option<i32>,
    #[sea_orm(unique)]
    pub order_number: String,
    pub status: String,
    pub payment_status: String,
    pub fulfillment_status: String,
    pub currency: String,
    pub subtotal: i64,
    pub tax: i64,
    pub shipping: i64,
    pub discount: i64,
    pub total: i64,
    pub shipping_address_id: Option<i32>,
    pub billing_address_id: Option<i32>,
    pub payment_method: Option<String>,
    pub payment_provider: Option<String>,
    pub payment_data: Json,
    pub notes: Option<String>,
    pub metadata: Json,
    pub canceled_at: Option<DateTimeWithTimeZone>,
    pub paid_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::customers::Entity",
        from = "Column::CustomerId",
        to = "super::customers::Column::Id"
    )]
    Customer,
    #[sea_orm(
        belongs_to = "super::carts::Entity",
        from = "Column::CartId",
        to = "super::carts::Column::Id"
    )]
    Cart,
    #[sea_orm(
        belongs_to = "super::addresses::Entity",
        from = "Column::ShippingAddressId",
        to = "super::addresses::Column::Id"
    )]
    ShippingAddress,
    #[sea_orm(has_many = "super::order_items::Entity")]
    Items,
}

impl Related<super::customers::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Customer.def()
    }
}

impl Related<super::carts::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Cart.def()
    }
}

impl Related<super::order_items::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Items.def()
    }
}
