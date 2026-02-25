use sea_orm::{QueryOrder, QuerySelect};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use super::_entities::products::{self, ActiveModel, Entity, Model};

use loco_rs::prelude::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateProductParams {
    pub title: String,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub handle: Option<String>,
    pub product_type: Option<String>,
    pub category_id: Option<i32>,
    pub tags: Option<Vec<String>>,
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
    pub weight: Option<f64>,
    pub featured: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateProductParams {
    pub title: Option<String>,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub product_type: Option<String>,
    pub category_id: Option<i32>,
    pub tags: Option<Vec<String>>,
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
    pub weight: Option<f64>,
    pub featured: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ProductListParams {
    pub cursor: Option<String>,
    pub limit: Option<u64>,
    pub status: Option<String>,
    pub collection: Option<String>,
    pub category_id: Option<i32>,
    pub featured: Option<bool>,
    pub q: Option<String>,
    pub sort: Option<String>,
}

impl ActiveModelBehavior for ActiveModel {}

/// Gera slug a partir do título
fn slugify(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .map(|c| match c {
            'a'..='z' | '0'..='9' => c,
            ' ' | '-' | '_' => '-',
            'á' | 'à' | 'â' | 'ã' | 'ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'í' | 'ì' | 'î' | 'ï' => 'i',
            'ó' | 'ò' | 'ô' | 'õ' | 'ö' => 'o',
            'ú' | 'ù' | 'û' | 'ü' => 'u',
            'ç' => 'c',
            'ñ' => 'n',
            _ => '-',
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

impl Model {
    /// Cria um novo produto
    pub async fn create_product(
        db: &DatabaseConnection,
        store_id: i32,
        params: &CreateProductParams,
    ) -> ModelResult<Self> {
        let slug = params.slug.clone().unwrap_or_else(|| slugify(&params.title));
        let handle = params.handle.clone().unwrap_or_else(|| slug.clone());

        let product = products::ActiveModel {
            pid: ActiveValue::set(Uuid::new_v4()),
            store_id: ActiveValue::set(store_id),
            title: ActiveValue::set(params.title.clone()),
            slug: ActiveValue::set(slug),
            description: ActiveValue::set(params.description.clone().unwrap_or_default()),
            handle: ActiveValue::set(handle),
            status: ActiveValue::set("draft".to_string()),
            product_type: ActiveValue::set(
                params.product_type.clone().unwrap_or_else(|| "physical".to_string()),
            ),
            category_id: ActiveValue::set(params.category_id),
            tags: ActiveValue::set(serde_json::json!(params.tags.clone().unwrap_or_default())),
            metadata: ActiveValue::set(
                params.metadata.clone().unwrap_or(serde_json::json!({})),
            ),
            seo_title: ActiveValue::set(params.seo_title.clone()),
            seo_description: ActiveValue::set(params.seo_description.clone()),
            weight: ActiveValue::set(params.weight.map(|w| rust_decimal::Decimal::from_f64_retain(w).unwrap_or_default())),
            featured: ActiveValue::set(params.featured.unwrap_or(false)),
            ..Default::default()
        };
        let product = product.insert(db).await?;
        Ok(product)
    }

    /// Busca produto pelo PID
    pub async fn find_by_pid(db: &DatabaseConnection, pid: &Uuid) -> ModelResult<Self> {
        let product = Entity::find()
            .filter(products::Column::Pid.eq(*pid))
            .filter(products::Column::DeletedAt.is_null())
            .one(db)
            .await?;
        product.ok_or_else(|| ModelError::EntityNotFound)
    }

    /// Lista produtos da loja com paginação
    pub async fn list_for_store(
        db: &DatabaseConnection,
        store_id: i32,
        params: &ProductListParams,
    ) -> ModelResult<Vec<Self>> {
        let limit = params.limit.unwrap_or(20).min(100);

        let mut query = Entity::find()
            .filter(products::Column::StoreId.eq(store_id))
            .filter(products::Column::DeletedAt.is_null());

        if let Some(ref status) = params.status {
            query = query.filter(products::Column::Status.eq(status.as_str()));
        }

        if let Some(cat_id) = params.category_id {
            query = query.filter(products::Column::CategoryId.eq(cat_id));
        }

        if let Some(featured) = params.featured {
            query = query.filter(products::Column::Featured.eq(featured));
        }

        if let Some(ref q) = params.q {
            query = query.filter(products::Column::Title.contains(q));
        }

        // Cursor-based pagination (usando id > cursor)
        if let Some(ref cursor) = params.cursor {
            if let Ok(cursor_id) = cursor.parse::<i32>() {
                query = query.filter(products::Column::Id.gt(cursor_id));
            }
        }

        let products = query
            .order_by_asc(products::Column::Id)
            .limit(limit)
            .all(db)
            .await?;

        Ok(products)
    }
}
