use sea_orm::QueryOrder;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use super::_entities::collection_products;
pub use super::_entities::collections::{self, ActiveModel, Entity, Model};

use loco_rs::prelude::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateCollectionParams {
    pub name: String,
    pub slug: Option<String>,
    pub description: Option<String>,
}

impl ActiveModelBehavior for ActiveModel {}
impl ActiveModelBehavior for collection_products::ActiveModel {}

fn slugify(name: &str) -> String {
    name.to_lowercase()
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
    /// Cria uma nova coleção
    pub async fn create_collection(
        db: &DatabaseConnection,
        params: &CreateCollectionParams,
    ) -> ModelResult<Self> {
        let slug = params.slug.clone().unwrap_or_else(|| slugify(&params.name));
        let collection = collections::ActiveModel {
            pid: ActiveValue::set(Uuid::new_v4()),
            title: ActiveValue::set(params.name.clone()),
            slug: ActiveValue::set(slug),
            description: ActiveValue::set(params.description.clone().unwrap_or_default()),
            metadata: ActiveValue::set(serde_json::json!({})),
            ..Default::default()
        };
        let collection = collection.insert(db).await?;
        Ok(collection)
    }

    /// Busca pelo PID
    pub async fn find_by_pid(db: &DatabaseConnection, pid: &Uuid) -> ModelResult<Self> {
        let collection = Entity::find()
            .filter(collections::Column::Pid.eq(*pid))
            .filter(collections::Column::DeletedAt.is_null())
            .one(db)
            .await?;
        collection.ok_or_else(|| ModelError::EntityNotFound)
    }

    /// Lista coleções
    pub async fn list_for_store(db: &DatabaseConnection) -> ModelResult<Vec<Self>> {
        let collections = Entity::find()
            .filter(collections::Column::DeletedAt.is_null())
            .order_by_asc(collections::Column::Title)
            .all(db)
            .await?;
        Ok(collections)
    }

    /// Adiciona produto à coleção
    pub async fn add_product(
        db: &DatabaseConnection,
        collection_id: i32,
        product_id: i32,
        sort_order: Option<i32>,
    ) -> ModelResult<collection_products::Model> {
        let cp = collection_products::ActiveModel {
            collection_id: ActiveValue::set(collection_id),
            product_id: ActiveValue::set(product_id),
            sort_order: ActiveValue::set(sort_order.unwrap_or(0)),
            ..Default::default()
        };
        let cp = cp.insert(db).await?;
        Ok(cp)
    }

    /// Remove produto da coleção
    pub async fn remove_product(
        db: &DatabaseConnection,
        collection_id: i32,
        product_id: i32,
    ) -> ModelResult<()> {
        collection_products::Entity::delete_many()
            .filter(collection_products::Column::CollectionId.eq(collection_id))
            .filter(collection_products::Column::ProductId.eq(product_id))
            .exec(db)
            .await?;
        Ok(())
    }
}
