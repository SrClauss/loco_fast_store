use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::env;

use crate::services::analytics::AnalyticsService;

pub struct LeadScoringWorker {
    pub ctx: AppContext,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct LeadScoringWorkerArgs {
    pub session_id: String,
}

#[async_trait]
impl BackgroundWorker<LeadScoringWorkerArgs> for LeadScoringWorker {
    fn build(ctx: &AppContext) -> Self {
        Self { ctx: ctx.clone() }
    }

    async fn perform(&self, args: LeadScoringWorkerArgs) -> Result<()> {
        env::load();
        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1".to_string());
        let sled_path =
            std::env::var("SLED_PATH").unwrap_or_else(|_| "./data/analytics_sled".to_string());

        let analytics = AnalyticsService::new(&redis_url, &sled_path)
            .map_err(|e| loco_rs::Error::Message(format!("Failed to init analytics: {}", e)))?;

        let score = analytics
            .calculate_lead_score(&args.session_id)
            .await
            .map_err(|e| loco_rs::Error::Message(format!("Failed to calculate lead score: {}", e)))?;

        tracing::info!(
            session_id = &args.session_id,
            lead_score = score,
            "Lead score calculated"
        );

        // Classifica o lead
        let classification = if score >= 30.0 {
            "hot"
        } else if score >= 15.0 {
            "warm"
        } else if score >= 5.0 {
            "cool"
        } else {
            "cold"
        };

        tracing::info!(
            session_id = &args.session_id,
            score = score,
            classification = classification,
            "Lead classified"
        );

        // Persiste score no Redis para consulta rápida
        if let Ok(client) = redis::Client::open(redis_url.as_str()) {
            if let Ok(mut conn) = client.get_multiplexed_tokio_connection().await {
                let key = format!("analytics:session:{}:lead_score", args.session_id);
                let _ = redis::cmd("SET")
                    .arg(&key)
                    .arg(score.to_string())
                    .query_async::<()>(&mut conn)
                    .await;
                let _ = redis::cmd("EXPIRE")
                    .arg(&key)
                    .arg(7 * 24 * 3600)
                    .query_async::<()>(&mut conn)
                    .await;
            }
        }

        Ok(())
    }
}
