use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};


use crate::models::carts::Model as CartModel;

pub struct AbandonedCartWorker {
    pub ctx: AppContext,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct AbandonedCartWorkerArgs {
    /// Tempo em minutos para considerar carrinho abandonado (default: 60)
    pub threshold_minutes: Option<i64>,
}

#[async_trait]
impl BackgroundWorker<AbandonedCartWorkerArgs> for AbandonedCartWorker {
    fn build(ctx: &AppContext) -> Self {
        Self { ctx: ctx.clone() }
    }

    async fn perform(&self, args: AbandonedCartWorkerArgs) -> Result<()> {
        crate::env::load();
        let threshold = args.threshold_minutes.unwrap_or(60);

        let abandoned_carts =
            CartModel::find_abandoned(&self.ctx.db, threshold).await?;

        for cart in &abandoned_carts {
            tracing::info!(
                cart_id = cart.id,
                session_id = &cart.session_id,
                email = cart.email.as_deref().unwrap_or("anon"),
                total_centavos = cart.total,
                "Carrinho abandonado detectado"
            );

            // Registra evento de analytics
            let sled_path =
                std::env::var("SLED_PATH").unwrap_or_else(|_| "./data/analytics_sled".to_string());

            if let Ok(analytics) =
                crate::services::analytics::AnalyticsService::new(&sled_path)
            {
                let event = crate::services::analytics::AnalyticsEvent {
                    session_id: cart.session_id.clone(),
                    customer_id: cart.customer_id,
                    event_type: "cart_abandon".to_string(),
                    entity_type: Some("cart".to_string()),
                    entity_id: Some(cart.pid.to_string()),
                    metadata: serde_json::json!({
                        "total": cart.total,
                        "currency": cart.currency,
                        "email": cart.email,
                    }),
                    timestamp: chrono::Utc::now().timestamp(),
                };

                if let Err(e) = analytics.track_event(&event).await {
                    tracing::warn!("Failed to track cart_abandon event: {}", e);
                }
            }

            // TODO: Disparar email de recuperação de carrinho via mailer
        }

        tracing::info!(
            total_abandoned = abandoned_carts.len(),
            "Abandoned cart detection completed"
        );

        Ok(())
    }
}
