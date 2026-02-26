use crate::models::{_entities::users, users::RegisterParams};
use axum::{
    extract::State,
    http::Method,
    response::{Html, IntoResponse, Redirect},
};
use loco_rs::prelude::*;
use sea_orm::{EntityTrait, PaginatorTrait};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct SetupParams {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct SetupResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Verifica se já existe algum administrador no sistema
pub async fn has_admin(db: &DatabaseConnection) -> ModelResult<bool> {
    let count = users::Entity::find().paginate(db, 1).num_items().await?;

    Ok(count > 0)
}

/// Exibe a página de setup inicial (GET /admin/setup)
#[debug_handler]
pub async fn show_setup(State(ctx): State<AppContext>) -> Result<impl IntoResponse> {
    // Verificar se já existe admin
    if has_admin(&ctx.db).await? {
        return Ok(Redirect::to("/admin/login").into_response());
    }

    // Renderiza o template HTML diretamente
    let html = std::fs::read_to_string("assets/views/admin/setup.html")
        .map_err(|_| Error::InternalServerError)?;

    Ok(Html(html).into_response())
}

/// Cria o primeiro administrador (POST /admin/setup)
#[debug_handler]
pub async fn create_admin(
    State(ctx): State<AppContext>,
    Json(params): Json<SetupParams>,
) -> Result<Response> {
    // Verificar se já existe admin
    if has_admin(&ctx.db).await? {
        return format::json(SetupResponse {
            success: false,
            error: Some("Já existe um administrador cadastrado".to_string()),
        });
    }

    // Validações básicas
    if params.name.len() < 2 {
        return format::json(SetupResponse {
            success: false,
            error: Some("O nome deve ter pelo menos 2 caracteres".to_string()),
        });
    }

    if params.password.len() < 8 {
        return format::json(SetupResponse {
            success: false,
            error: Some("A senha deve ter pelo menos 8 caracteres".to_string()),
        });
    }

    // Validar formato do email
    if !params.email.contains('@') {
        return format::json(SetupResponse {
            success: false,
            error: Some("Email inválido".to_string()),
        });
    }

    // Criar o usuário administrador
    let register_params = RegisterParams {
        name: params.name,
        email: params.email,
        password: params.password,
    };

    match users::Model::create_with_password(&ctx.db, &register_params).await {
        Ok(user) => {
            tracing::info!(
                user_pid = user.pid.to_string(),
                user_email = user.email,
                "primeiro administrador criado com sucesso"
            );

            format::json(SetupResponse {
                success: true,
                error: None,
            })
        }
        Err(err) => {
            tracing::error!(
                error = err.to_string(),
                "erro ao criar primeiro administrador"
            );

            let error_msg = match err {
                ModelError::EntityAlreadyExists {} => "Este email já está cadastrado",
                _ => "Erro ao criar administrador. Tente novamente.",
            };

            format::json(SetupResponse {
                success: false,
                error: Some(error_msg.to_string()),
            })
        }
    }
}

/// Middleware para verificar se o sistema precisa de setup
/// Redireciona para /admin/setup se não houver admins cadastrados
pub async fn check_setup_required(
    State(ctx): State<AppContext>,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<impl IntoResponse> {
    let path = request.uri().path();

    // Não redirecionar se já estiver na página de setup, acessando estáticos
    // ou fazendo qualquer requisição para a API; queremos que endpoints JSON
    // continuem retornando erro em vez de redirecionar com HTML.
    if path.starts_with("/admin/setup") || path.starts_with("/static") || path.starts_with("/api") {
        return Ok(next.run(request).await);
    }

    // Somente redirecionar requisições GET do frontend quando não houver admin
    if request.method() != Method::GET {
        return Ok(next.run(request).await);
    }

    // Verificar se existe admin
    match has_admin(&ctx.db).await {
        Ok(false) => {
            // Não existe admin, redirecionar para setup
            Ok(Redirect::to("/admin/setup").into_response())
        }
        _ => {
            // Existe admin ou erro na verificação, continuar normalmente
            Ok(next.run(request).await)
        }
    }
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("admin")
        .add("/setup", get(show_setup))
        .add("/setup", post(create_admin))
}
