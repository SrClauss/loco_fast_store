use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::services::analytics::AnalyticsService;

pub struct AnalyticsFlushWorker {
    pub ctx: AppContext,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct AnalyticsFlushWorkerArgs {
    pub store_id: i32,
}

#[async_trait]
impl BackgroundWorker<AnalyticsFlushWorkerArgs> for AnalyticsFlushWorker {
    fn build(ctx: &AppContext) -> Self {
        Self { ctx: ctx.clone() }
    }

    async fn perform(&self, args: AnalyticsFlushWorkerArgs) -> Result<()> {
        let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1".to_string());
        let sled_path = std::env::var("SLED_PATH").unwrap_or_else(|_| "./data/analytics_sled".to_string());

        let analytics = AnalyticsService::new(&redis_url, &sled_path)
            .map_err(|e| loco_rs::Error::Message(format!("Failed to init analytics: {}", e)))?;

        let count = analytics
            .flush_to_sled(args.store_id)
            .await
            .map_err(|e| loco_rs::Error::Message(format!("Failed to flush analytics: {}", e)))?;

        tracing::info!(
            store_id = args.store_id,
            events_flushed = count,
            "Analytics flush worker completed"
        );

        Ok(())
    }
}
