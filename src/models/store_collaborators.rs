use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use super::_entities::store_collaborators::{self, ActiveModel, Entity, Model};
use loco_rs::prelude::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct AddCollaboratorParams {
    pub user_id: i32,
    /// 'owner' | 'admin' | 'shipping' | 'viewer'
    pub role: String,
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Roles com permissão de leitura de pedidos
    pub fn can_view_orders(&self) -> bool {
        self.active
            && matches!(
                self.role.as_str(),
                "owner" | "admin" | "shipping" | "viewer"
            )
    }

    /// Roles com permissão de atualizar status de pedidos/envios
    pub fn can_update_orders(&self) -> bool {
        self.active && matches!(self.role.as_str(), "owner" | "admin" | "shipping")
    }

    /// Roles com permissão de gerenciar colaboradores
    pub fn can_manage_collaborators(&self) -> bool {
        self.active && matches!(self.role.as_str(), "owner" | "admin")
    }

    /// Adiciona colaborador
    pub async fn add_collaborator(
        db: &DatabaseConnection,
        params: &AddCollaboratorParams,
    ) -> ModelResult<Self> {
        let collab = store_collaborators::ActiveModel {
            pid: ActiveValue::set(Uuid::new_v4()),
            user_id: ActiveValue::set(params.user_id),
            role: ActiveValue::set(params.role.clone()),
            active: ActiveValue::set(true),
            ..Default::default()
        };
        let saved = collab.insert(db).await?;
        Ok(saved)
    }

    /// Lista colaboradores ativos
    pub async fn list_for_store(db: &DatabaseConnection) -> ModelResult<Vec<Self>> {
        let collabs = Entity::find()
            .filter(store_collaborators::Column::Active.eq(true))
            .all(db)
            .await?;
        Ok(collabs)
    }

    /// Lista os vínculos de um usuário
    pub async fn list_for_user(db: &DatabaseConnection, user_id: i32) -> ModelResult<Vec<Self>> {
        let collabs = Entity::find()
            .filter(store_collaborators::Column::UserId.eq(user_id))
            .filter(store_collaborators::Column::Active.eq(true))
            .all(db)
            .await?;
        Ok(collabs)
    }

    /// Busca o vínculo de um usuário
    pub async fn find_for_user(db: &DatabaseConnection, user_id: i32) -> ModelResult<Self> {
        Entity::find()
            .filter(store_collaborators::Column::UserId.eq(user_id))
            .filter(store_collaborators::Column::Active.eq(true))
            .one(db)
            .await?
            .ok_or_else(|| ModelError::EntityNotFound)
    }

    /// Desativa colaborador (soft-disable)
    pub async fn deactivate(db: &DatabaseConnection, user_id: i32) -> ModelResult<()> {
        let collab = Self::find_for_user(db, user_id).await?;
        let mut active: store_collaborators::ActiveModel = collab.into();
        active.active = ActiveValue::set(false);
        active.update(db).await?;
        Ok(())
    }
}
