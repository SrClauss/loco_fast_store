use sea_orm::QueryOrder;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use super::_entities::product_variants::{self, ActiveModel, Entity, Model};
pub use super::_entities::prices;

use loco_rs::prelude::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateVariantParams {
    pub sku: String,
    pub title: String,
    pub option_values: Option<serde_json::Value>,
    pub inventory_quantity: Option<i32>,
    pub allow_backorder: Option<bool>,
    pub weight: Option<f64>,
    pub sort_order: Option<i32>,
    pub prices: Option<Vec<CreatePriceParams>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CreatePriceParams {
    pub amount: i64,
    pub currency: Option<String>,
    pub region: Option<String>,
    pub min_quantity: Option<i32>,
    pub max_quantity: Option<i32>,
}

impl ActiveModelBehavior for ActiveModel {}
impl ActiveModelBehavior for prices::ActiveModel {}

impl Model {
    /// Cria uma nova variante de produto
    pub async fn create_variant(
        db: &DatabaseConnection,
        product_id: i32,
        params: &CreateVariantParams,
    ) -> ModelResult<Self> {
        let variant = product_variants::ActiveModel {
            pid: ActiveValue::set(Uuid::new_v4()),
            product_id: ActiveValue::set(product_id),
            sku: ActiveValue::set(params.sku.clone()),
            title: ActiveValue::set(params.title.clone()),
            option_values: ActiveValue::set(
                params.option_values.clone().unwrap_or(serde_json::json!({})),
            ),
            inventory_quantity: ActiveValue::set(params.inventory_quantity.unwrap_or(0)),
            allow_backorder: ActiveValue::set(params.allow_backorder.unwrap_or(false)),
            weight: ActiveValue::set(
                params.weight.map(|w| rust_decimal::Decimal::from_f64_retain(w).unwrap_or_default()),
            ),
            dimensions: ActiveValue::set(None),
            sort_order: ActiveValue::set(params.sort_order.unwrap_or(0)),
            metadata: ActiveValue::set(serde_json::json!({})),
            ..Default::default()
        };
        let variant = variant.insert(db).await?;

        // Cria preços se fornecidos
        if let Some(ref price_params) = params.prices {
            for p in price_params {
                Self::create_price(db, variant.id, p).await?;
            }
        }

        Ok(variant)
    }

    /// Cria um preço para a variante
    pub async fn create_price(
        db: &DatabaseConnection,
        variant_id: i32,
        params: &CreatePriceParams,
    ) -> ModelResult<prices::Model> {
        let price = prices::ActiveModel {
            pid: ActiveValue::set(Uuid::new_v4()),
            variant_id: ActiveValue::set(variant_id),
            amount: ActiveValue::set(params.amount),
            currency: ActiveValue::set(params.currency.clone().unwrap_or_else(|| "BRL".to_string())),
            region: ActiveValue::set(params.region.clone()),
            min_quantity: ActiveValue::set(params.min_quantity.unwrap_or(1)),
            max_quantity: ActiveValue::set(params.max_quantity),
            ..Default::default()
        };
        let price = price.insert(db).await?;
        Ok(price)
    }

    /// Lista variantes de um produto
    pub async fn find_by_product(
        db: &DatabaseConnection,
        product_id: i32,
    ) -> ModelResult<Vec<Self>> {
        let variants = Entity::find()
            .filter(product_variants::Column::ProductId.eq(product_id))
            .filter(product_variants::Column::DeletedAt.is_null())
            .order_by_asc(product_variants::Column::SortOrder)
            .all(db)
            .await?;
        Ok(variants)
    }

    /// Busca variante pelo PID
    pub async fn find_by_pid(db: &DatabaseConnection, pid: &Uuid) -> ModelResult<Self> {
        let variant = Entity::find()
            .filter(product_variants::Column::Pid.eq(*pid))
            .filter(product_variants::Column::DeletedAt.is_null())
            .one(db)
            .await?;
        variant.ok_or_else(|| ModelError::EntityNotFound)
    }

    /// Busca preços da variante
    pub async fn get_prices(
        db: &DatabaseConnection,
        variant_id: i32,
    ) -> ModelResult<Vec<prices::Model>> {
        let prices_list = prices::Entity::find()
            .filter(prices::Column::VariantId.eq(variant_id))
            .order_by_asc(prices::Column::MinQuantity)
            .all(db)
            .await?;
        Ok(prices_list)
    }

    /// Retorna o preço ativo para uma moeda e quantidade
    pub async fn get_active_price(
        db: &DatabaseConnection,
        variant_id: i32,
        currency: &str,
        quantity: i32,
    ) -> ModelResult<prices::Model> {
        let now = chrono::Utc::now();
        let price = prices::Entity::find()
            .filter(prices::Column::VariantId.eq(variant_id))
            .filter(prices::Column::Currency.eq(currency))
            .filter(prices::Column::MinQuantity.lte(quantity))
            .filter(
                prices::Column::MaxQuantity
                    .is_null()
                    .or(prices::Column::MaxQuantity.gte(quantity)),
            )
            .filter(
                prices::Column::StartsAt
                    .is_null()
                    .or(prices::Column::StartsAt.lte(now)),
            )
            .filter(
                prices::Column::EndsAt
                    .is_null()
                    .or(prices::Column::EndsAt.gte(now)),
            )
            .order_by_desc(prices::Column::MinQuantity)
            .one(db)
            .await?;
        price.ok_or_else(|| ModelError::EntityNotFound)
    }
}
