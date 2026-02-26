///! Painel de colaboradores — rotas HTML (Tera)
use loco_rs::prelude::*;
use uuid::Uuid;

use crate::models::{_entities::users, store_collaborators::Model as CollaboratorModel};

// ── Páginas HTML ─────────────────────────────────────────────────────────────

/// GET /painel/login
#[debug_handler]
pub async fn login_page(ViewEngine(v): ViewEngine<TeraView>) -> Result<Response> {
    format::render().view(&v, "painel/login.html", serde_json::json!({}))
}

/// GET /painel/  →  redireciona para /painel/login
#[debug_handler]
pub async fn index() -> Result<Response> {
    Ok(axum::response::Redirect::to("/painel/login").into_response())
}

/// GET /painel/dashboard  →  dashboard de pedidos
#[debug_handler]
pub async fn dashboard(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    ViewEngine(v): ViewEngine<TeraView>,
) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;

    // Verifica se o usuário é colaborador
    let collab = CollaboratorModel::find_for_user(&ctx.db, user.id)
        .await
        .map_err(|_| loco_rs::Error::Unauthorized("Acesso negado".into()))?;

    if !collab.can_view_orders() {
        return Err(loco_rs::Error::Unauthorized(
            "Permissão insuficiente".into(),
        ));
    }

    format::render().view(
        &v,
        "painel/dashboard.html",
        serde_json::json!({
            "user": {
                "name": user.name,
                "email": user.email,
                "role": collab.role,
            },
            "can_update": collab.can_update_orders(),
            "can_manage": collab.can_manage_collaborators(),
        }),
    )
}

/// GET /painel/pedidos  →  lista de pedidos
#[debug_handler]
pub async fn orders_list(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    ViewEngine(v): ViewEngine<TeraView>,
) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;

    let collab = CollaboratorModel::find_for_user(&ctx.db, user.id)
        .await
        .map_err(|_| loco_rs::Error::Unauthorized("Acesso negado".into()))?;

    if !collab.can_view_orders() {
        return Err(loco_rs::Error::Unauthorized(
            "Permissão insuficiente".into(),
        ));
    }

    format::render().view(
        &v,
        "painel/pedidos/list.html",
        serde_json::json!({
            "user": { "name": user.name, "role": collab.role },
            "can_update": collab.can_update_orders(),
        }),
    )
}

/// GET /painel/pedidos/:order_pid  →  detalhe + gestão de envio
#[debug_handler]
pub async fn order_detail(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(order_pid): Path<Uuid>,
    ViewEngine(v): ViewEngine<TeraView>,
) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;

    let collab = CollaboratorModel::find_for_user(&ctx.db, user.id)
        .await
        .map_err(|_| loco_rs::Error::Unauthorized("Acesso negado".into()))?;

    if !collab.can_view_orders() {
        return Err(loco_rs::Error::Unauthorized(
            "Permissão insuficiente".into(),
        ));
    }

    format::render().view(
        &v,
        "painel/pedidos/detail.html",
        serde_json::json!({
            "order_pid": order_pid.to_string(),
            "user": { "name": user.name, "role": collab.role },
            "can_update": collab.can_update_orders(),
        }),
    )
}

// ── Roteamento ────────────────────────────────────────────────────────────────

pub fn routes() -> Routes {
    Routes::new()
        .add("/painel/login", get(login_page))
        .add("/painel/", get(index))
        .add("/painel/dashboard", get(dashboard))
        .add("/painel/pedidos", get(orders_list))
        .add("/painel/pedidos/{order_pid}", get(order_detail))
}
