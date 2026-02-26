use sea_orm::QueryOrder;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use super::_entities::categories::{self, ActiveModel, Entity, Model};

use loco_rs::prelude::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateCategoryParams {
    pub name: String,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub parent_id: Option<i32>,
    pub image_url: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateCategoryParams {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub parent_id: Option<i32>,
    pub image_url: Option<String>,
    pub sort_order: Option<i32>,
}

impl ActiveModelBehavior for ActiveModel {}

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
    /// Cria uma nova categoria
    pub async fn create_category(
        db: &DatabaseConnection,
        params: &CreateCategoryParams,
    ) -> ModelResult<Self> {
        let slug = params.slug.clone().unwrap_or_else(|| slugify(&params.name));
        let category = categories::ActiveModel {
            pid: ActiveValue::set(Uuid::new_v4()),
            name: ActiveValue::set(params.name.clone()),
            slug: ActiveValue::set(slug),
            description: ActiveValue::set(params.description.clone()),
            parent_id: ActiveValue::set(params.parent_id),
            image_url: ActiveValue::set(params.image_url.clone()),
            sort_order: ActiveValue::set(params.sort_order.unwrap_or(0)),
            ..Default::default()
        };
        let category = category.insert(db).await?;
        Ok(category)
    }

    /// Busca categoria pelo PID
    pub async fn find_by_pid(db: &DatabaseConnection, pid: &Uuid) -> ModelResult<Self> {
        let category = Entity::find()
            .filter(categories::Column::Pid.eq(*pid))
            .filter(categories::Column::DeletedAt.is_null())
            .one(db)
            .await?;
        category.ok_or_else(|| ModelError::EntityNotFound)
    }

    /// Lista categorias (árvore: raízes primeiro)
    pub async fn list_for_store(
        db: &DatabaseConnection,
        parent_id: Option<i32>,
    ) -> ModelResult<Vec<Self>> {
        let mut query = Entity::find()
            .filter(categories::Column::DeletedAt.is_null());

        if let Some(pid) = parent_id {
            query = query.filter(categories::Column::ParentId.eq(pid));
        } else {
            query = query.filter(categories::Column::ParentId.is_null());
        }

        let categories = query
            .order_by_asc(categories::Column::SortOrder)
            .all(db)
            .await?;
        Ok(categories)
    }

    /// Lista filhas de uma categoria
    pub async fn find_children(db: &DatabaseConnection, category_id: i32) -> ModelResult<Vec<Self>> {
        let children = Entity::find()
            .filter(categories::Column::ParentId.eq(category_id))
            .filter(categories::Column::DeletedAt.is_null())
            .order_by_asc(categories::Column::SortOrder)
            .all(db)
            .await?;
        Ok(children)
    }

    /// Busca por slug
    pub async fn find_by_slug(
        db: &DatabaseConnection,
        slug: &str,
    ) -> ModelResult<Self> {
        let category = Entity::find()
            .filter(categories::Column::Slug.eq(slug))
            .filter(categories::Column::DeletedAt.is_null())
            .one(db)
            .await?;
        category.ok_or_else(|| ModelError::EntityNotFound)
    }
}
