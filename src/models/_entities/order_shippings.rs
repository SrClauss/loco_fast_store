//! `SeaORM` Entity — Envios de pedidos

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "order_shippings")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub pid: Uuid,
    pub order_id: i32,
    pub carrier: String,
    pub service: Option<String>,
    pub tracking_code: Option<String>,
    pub tracking_url: Option<String>,
    /// Status: 'pending' | 'posted' | 'in_transit' | 'out_for_delivery' | 'delivered' | 'failed' | 'returned'
    pub status: String,
    pub shipped_at: Option<DateTimeWithTimeZone>,
    pub estimated_delivery_at: Option<DateTimeWithTimeZone>,
    pub delivered_at: Option<DateTimeWithTimeZone>,
    /// Provider externo: 'melhor_envio' | 'correios_api' | null = manual
    pub provider: Option<String>,
    pub provider_data: Json,
    pub notes: Option<String>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::orders::Entity",
        from = "Column::OrderId",
        to = "super::orders::Column::Id"
    )]
    Order,
}

impl Related<super::orders::Entity> for Entity {
    fn to() -> RelationDef { Relation::Order.def() }
}
