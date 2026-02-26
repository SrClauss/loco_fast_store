///! Painel de colaboradores da loja — rotas HTML (Tera)
use loco_rs::prelude::*;
use uuid::Uuid;

use crate::models::{
    _entities::users,
    store_collaborators::Model as CollaboratorModel,
    stores::Model as StoreModel,
};

// ── Páginas HTML ─────────────────────────────────────────────────────────────

/// GET /painel/login
#[debug_handler]
pub async fn login_page(
    ViewEngine(v): ViewEngine<TeraView>,
) -> Result<Response> {
    format::render().view(&v, "painel/login.html", serde_json::json!({}))
}

/// GET /painel/  →  redireciona para /painel/login
#[debug_handler]
pub async fn index() -> Result<Response> {
    Ok(axum::response::Redirect::to("/painel/login").into_response())
}

/// GET /painel/:store_pid/  →  dashboard de pedidos da loja
#[debug_handler]
pub async fn dashboard(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(store_pid): Path<Uuid>,
    ViewEngine(v): ViewEngine<TeraView>,
) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let store = StoreModel::find_by_pid(&ctx.db, &store_pid).await?;

    // Verifica se o usuário é colaborador da loja
    let collab = CollaboratorModel::find_for_user_and_store(&ctx.db, user.id, store.id)
        .await
        .map_err(|_| loco_rs::Error::Unauthorized("Acesso negado a esta loja".into()))?;

    if !collab.can_view_orders() {
        return Err(loco_rs::Error::Unauthorized("Permissão insuficiente".into()));
    }

    format::render().view(&v, "painel/dashboard.html", serde_json::json!({
        "store": {
            "pid": store.pid.to_string(),
            "name": store.name,
            "slug": store.slug,
        },
        "user": {
            "name": user.name,
            "email": user.email,
            "role": collab.role,
        },
        "can_update": collab.can_update_orders(),
        "can_manage": collab.can_manage_collaborators(),
    }))
}

/// GET /painel/:store_pid/pedidos  →  lista de pedidos
#[debug_handler]
pub async fn orders_list(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(store_pid): Path<Uuid>,
    ViewEngine(v): ViewEngine<TeraView>,
) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let store = StoreModel::find_by_pid(&ctx.db, &store_pid).await?;

    let collab = CollaboratorModel::find_for_user_and_store(&ctx.db, user.id, store.id)
        .await
        .map_err(|_| loco_rs::Error::Unauthorized("Acesso negado".into()))?;

    if !collab.can_view_orders() {
        return Err(loco_rs::Error::Unauthorized("Permissão insuficiente".into()));
    }

    format::render().view(&v, "painel/pedidos/list.html", serde_json::json!({
        "store": {
            "pid": store.pid.to_string(),
            "name": store.name,
        },
        "user": { "name": user.name, "role": collab.role },
        "can_update": collab.can_update_orders(),
    }))
}

/// GET /painel/:store_pid/pedidos/:order_pid  →  detalhe + gestão de envio
#[debug_handler]
pub async fn order_detail(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path((store_pid, order_pid)): Path<(Uuid, Uuid)>,
    ViewEngine(v): ViewEngine<TeraView>,
) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let store = StoreModel::find_by_pid(&ctx.db, &store_pid).await?;

    let collab = CollaboratorModel::find_for_user_and_store(&ctx.db, user.id, store.id)
        .await
        .map_err(|_| loco_rs::Error::Unauthorized("Acesso negado".into()))?;

    if !collab.can_view_orders() {
        return Err(loco_rs::Error::Unauthorized("Permissão insuficiente".into()));
    }

    format::render().view(&v, "painel/pedidos/detail.html", serde_json::json!({
        "store": {
            "pid": store.pid.to_string(),
            "name": store.name,
        },
        "order_pid": order_pid.to_string(),
        "user": { "name": user.name, "role": collab.role },
        "can_update": collab.can_update_orders(),
    }))
}

// ── Roteamento ────────────────────────────────────────────────────────────────

pub fn routes() -> Routes {
    Routes::new()
        .add("/painel/login", get(login_page))
        .add("/painel/", get(index))
        .add("/painel/{store_pid}/", get(dashboard))
        .add("/painel/{store_pid}/pedidos", get(orders_list))
        .add("/painel/{store_pid}/pedidos/{order_pid}", get(order_detail))
}
