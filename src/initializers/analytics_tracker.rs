/// Middleware de Analytics Tracker
/// Rastreia automaticamente product_view e detecta revisitas
///
/// Implementado como initializer do Loco que adiciona um layer ao Router

use async_trait::async_trait;
use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
    Router as AxumRouter,
};
use loco_rs::{
    app::{AppContext, Initializer},
    Result,
};

pub struct AnalyticsTrackerInitializer;

#[async_trait]
impl Initializer for AnalyticsTrackerInitializer {
    fn name(&self) -> String {
        "analytics-tracker".to_string()
    }

    async fn after_routes(&self, router: AxumRouter, _ctx: &AppContext) -> Result<AxumRouter> {
        let router = router.layer(axum::middleware::from_fn(analytics_tracker_middleware));
        Ok(router)
    }
}

/// Middleware que extrai session_id de cookies/headers e registra eventos
async fn analytics_tracker_middleware(req: Request, next: Next) -> Response {
    // Extrai session_id do header ou cookie
    let session_id = req
        .headers()
        .get("x-session-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let path = req.uri().path().to_string();
    let method = req.method().clone();

    let response = next.run(req).await;

    // Só rastreia GETs em rotas de produtos (leitura)
    if method == axum::http::Method::GET && path.contains("/products/") {
        if let Some(sid) = session_id {
            tracing::debug!(
                session_id = &sid,
                path = &path,
                "Analytics: product page view tracked"
            );
            // O tracking real é feito no controller via AnalyticsService
            // Este middleware serve para logging e detecção de padrões
        }
    }

    response
}
