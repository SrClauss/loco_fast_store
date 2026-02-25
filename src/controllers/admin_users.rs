use crate::{
    models::{
        _entities::users,
        users::RegisterParams,
    },
};
use axum::{
    extract::{State, Path},
};
use loco_rs::{prelude::*, hash};
use serde::{Deserialize, Serialize};
use sea_orm::{EntityTrait, PaginatorTrait, QueryOrder};

#[derive(Debug, Deserialize, Serialize)]
pub struct UserParams {
    pub name: String,
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: i32,
    pub pid: String,
    pub name: String,
    pub email: String,
    pub email_verified_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<users::Model> for UserResponse {
    fn from(user: users::Model) -> Self {
        Self {
            id: user.id,
            pid: user.pid.to_string(),
            name: user.name,
            email: user.email,
            email_verified_at: user.email_verified_at.map(|dt| dt.to_rfc3339()),
            created_at: user.created_at.to_rfc3339(),
            updated_at: user.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// Lista todos os usuários (GET /api/admin/users)
#[debug_handler]
pub async fn list_users(
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let users = users::Entity::find()
        .order_by_desc(users::Column::CreatedAt)
        .all(&ctx.db)
        .await?;

    let response: Vec<UserResponse> = users
        .into_iter()
        .map(UserResponse::from)
        .collect();

    format::json(response)
}

/// Cria um novo usuário (POST /api/admin/users)
#[debug_handler]
pub async fn create_user(
    State(ctx): State<AppContext>,
    Json(params): Json<UserParams>,
) -> Result<Response> {
    // Validações
    if params.name.len() < 2 {
        return format::json(ErrorResponse {
            error: "O nome deve ter pelo menos 2 caracteres".to_string(),
        });
    }

    if !params.email.contains('@') {
        return format::json(ErrorResponse {
            error: "Email inválido".to_string(),
        });
    }

    let password = match &params.password {
        Some(pwd) if pwd.len() >= 8 => pwd.clone(),
        Some(_) => {
            return format::json(ErrorResponse {
                error: "A senha deve ter pelo menos 8 caracteres".to_string(),
            });
        }
        None => {
            return format::json(ErrorResponse {
                error: "Senha é obrigatória".to_string(),
            });
        }
    };

    // Verificar se o email já existe
    if let Ok(_existing) = users::Model::find_by_email(&ctx.db, &params.email).await {
        return format::json(ErrorResponse {
            error: "Este email já está cadastrado".to_string(),
        });
    }

    // Criar usuário
    let register_params = RegisterParams {
        name: params.name,
        email: params.email,
        password,
    };

    match users::Model::create_with_password(&ctx.db, &register_params).await {
        Ok(user) => {
            tracing::info!(
                user_pid = user.pid.to_string(),
                user_email = user.email,
                "novo usuário criado pelo admin"
            );

            format::json(UserResponse::from(user))
        }
        Err(err) => {
            tracing::error!(
                error = err.to_string(),
                "erro ao criar usuário"
            );

            format::json(ErrorResponse {
                error: "Erro ao criar usuário. Tente novamente.".to_string(),
            })
        }
    }
}

/// Atualiza um usuário existente (PUT /api/admin/users/:id)
#[debug_handler]
pub async fn update_user(
    State(ctx): State<AppContext>,
    Path(id): Path<i32>,
    Json(params): Json<UserParams>,
) -> Result<Response> {
    // Buscar usuário
    let user = users::Entity::find_by_id(id)
        .one(&ctx.db)
        .await?
        .ok_or_else(|| Error::NotFound)?;

    // Validações
    if params.name.len() < 2 {
        return format::json(ErrorResponse {
            error: "O nome deve ter pelo menos 2 caracteres".to_string(),
        });
    }

    if !params.email.contains('@') {
        return format::json(ErrorResponse {
            error: "Email inválido".to_string(),
        });
    }

    // Verificar se o email já pertence a outro usuário
    if user.email != params.email {
        if let Ok(_existing) = users::Model::find_by_email(&ctx.db, &params.email).await {
            return format::json(ErrorResponse {
                error: "Este email já está cadastrado".to_string(),
            });
        }
    }

    // Atualizar usuário
    let mut user: users::ActiveModel = user.into();
    user.name = sea_orm::Set(params.name);
    user.email = sea_orm::Set(params.email);

    // Atualizar senha se fornecida
    if let Some(password) = params.password {
        if !password.is_empty() {
            if password.len() < 8 {
                return format::json(ErrorResponse {
                    error: "A senha deve ter pelo menos 8 caracteres".to_string(),
                });
            }

            let password_hash = hash::hash_password(&password)
                .map_err(|_| Error::InternalServerError)?;
            user.password = sea_orm::Set(password_hash);
        }
    }

    let updated_user = user.update(&ctx.db).await?;

    tracing::info!(
        user_pid = updated_user.pid.to_string(),
        "usuário atualizado pelo admin"
    );

    format::json(UserResponse::from(updated_user))
}

/// Deleta um usuário (DELETE /api/admin/users/:id)
#[debug_handler]
pub async fn delete_user(
    State(ctx): State<AppContext>,
    Path(id): Path<i32>,
) -> Result<Response> {
    // Verificar se não é o único usuário
    let total_users = users::Entity::find()
        .count(&ctx.db)
        .await?;

    if total_users <= 1 {
        return format::json(ErrorResponse {
            error: "Não é possível excluir o único usuário do sistema".to_string(),
        });
    }

    // Buscar e deletar usuário
    let user = users::Entity::find_by_id(id)
        .one(&ctx.db)
        .await?
        .ok_or_else(|| Error::NotFound)?;

    let user_email = user.email.clone();
    let user_pid = user.pid.to_string();

    users::Entity::delete_by_id(id)
        .exec(&ctx.db)
        .await?;

    tracing::info!(
        user_pid = user_pid,
        user_email = user_email,
        "usuário deletado pelo admin"
    );

    #[derive(Serialize)]
    struct SuccessResponse {
        success: bool,
    }

    format::json(SuccessResponse { success: true })
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/admin/core/users")
        .add("/list", get(list_users))
        .add("/", post(create_user))
        .add("/{id}", put(update_user))
        .add("/{id}", delete(delete_user))
}
