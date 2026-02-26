#![allow(clippy::uninlined_format_args, clippy::missing_errors_doc, clippy::doc_markdown)]

//! Serviço de analytics: Redis (cache quente, TTL 7 dias) → Sled (persistência)
//!
//! Eventos rastreados:
//! - product_view: visitou a página de um produto
//! - product_detail_expand: expandiu detalhes (fotos, descrição longa)
//! - product_revisit: revisitou o mesmo produto (namoro)
//! - cart_add: adicionou ao carrinho
//! - cart_abandon: carrinho abandonado (detectado pelo worker)
//! - checkout_start: começou checkout
//! - checkout_complete: finalizou compra
//! - search: busca de produto


use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsEvent {
    pub store_id: i32,
    pub session_id: String,
    pub customer_id: Option<i32>,
    pub event_type: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
    pub metadata: serde_json::Value,
    pub timestamp: i64,
}

/// Chaves Redis para eventos de analytics
fn redis_key_event_stream(store_id: i32) -> String {
    format!("analytics:store:{}:events", store_id)
}

fn redis_key_product_views(store_id: i32, product_id: &str) -> String {
    format!("analytics:store:{}:product:{}:views", store_id, product_id)
}

fn redis_key_session_products(store_id: i32, session_id: &str) -> String {
    format!("analytics:store:{}:session:{}:products", store_id, session_id)
}

fn redis_key_product_visitors(store_id: i32, product_id: &str) -> String {
    format!("analytics:store:{}:product:{}:visitors", store_id, product_id)
}

/// Cliente Redis para analytics
pub struct AnalyticsService {
    redis: redis::Client,
    sled_db: Arc<sled::Db>,
}

impl AnalyticsService {
    /// Inicializa o serviço de analytics
    pub fn new(redis_url: &str, sled_path: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let redis = redis::Client::open(redis_url)?;
        let sled_db = Arc::new(sled::open(sled_path)?);
        Ok(Self { redis, sled_db })
    }

    /// Obtém conexão Redis
    async fn get_conn(&self) -> Result<redis::aio::MultiplexedConnection, redis::RedisError> {
        self.redis.get_multiplexed_tokio_connection().await
    }

    /// Registra evento de analytics no Redis
    pub async fn track_event(&self, event: &AnalyticsEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.get_conn().await?;
        let event_json = serde_json::to_string(event)?;

        // Adiciona ao stream de eventos da loja
        let key = redis_key_event_stream(event.store_id);
        redis::cmd("LPUSH")
            .arg(&key)
            .arg(&event_json)
            .query_async::<()>(&mut conn)
            .await?;

        // TTL de 7 dias no stream
        redis::cmd("EXPIRE")
            .arg(&key)
            .arg(7 * 24 * 3600)
            .query_async::<()>(&mut conn)
            .await?;

        // Se for product_view, incrementa contadores
        if event.event_type == "product_view" {
            if let Some(ref entity_id) = event.entity_id {
                self.track_product_view(event.store_id, entity_id, &event.session_id)
                    .await?;
            }
        }

        // Se for product_revisit, incrementa contador de namoro
        if event.event_type == "product_revisit" {
            if let Some(ref entity_id) = event.entity_id {
                self.track_product_revisit(event.store_id, entity_id, &event.session_id)
                    .await?;
            }
        }

        Ok(())
    }

    /// Incrementa view de produto e rastreia sessão
    async fn track_product_view(
        &self,
        store_id: i32,
        product_id: &str,
        session_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.get_conn().await?;

        // Incrementa contador total de views do produto
        let views_key = redis_key_product_views(store_id, product_id);
        redis::cmd("INCR")
            .arg(&views_key)
            .query_async::<()>(&mut conn)
            .await?;
        redis::cmd("EXPIRE")
            .arg(&views_key)
            .arg(7 * 24 * 3600)
            .query_async::<()>(&mut conn)
            .await?;

        // Rastreia quais produtos a sessão visitou (para detectar revisitas)
        let session_key = redis_key_session_products(store_id, session_id);
        redis::cmd("SADD")
            .arg(&session_key)
            .arg(product_id)
            .query_async::<()>(&mut conn)
            .await?;
        redis::cmd("EXPIRE")
            .arg(&session_key)
            .arg(7 * 24 * 3600)
            .query_async::<()>(&mut conn)
            .await?;

        // Rastreia visitantes únicos do produto (HyperLogLog)
        let visitors_key = redis_key_product_visitors(store_id, product_id);
        redis::cmd("PFADD")
            .arg(&visitors_key)
            .arg(session_id)
            .query_async::<()>(&mut conn)
            .await?;
        redis::cmd("EXPIRE")
            .arg(&visitors_key)
            .arg(7 * 24 * 3600)
            .query_async::<()>(&mut conn)
            .await?;

        Ok(())
    }

    /// Rastreia revisita (namoro) - session já visitou este produto antes
    async fn track_product_revisit(
        &self,
        store_id: i32,
        product_id: &str,
        session_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.get_conn().await?;

        // Incrementa contador de revisitas
        let key = format!(
            "analytics:store:{}:product:{}:revisits",
            store_id, product_id
        );
        redis::cmd("INCR")
            .arg(&key)
            .query_async::<()>(&mut conn)
            .await?;
        redis::cmd("EXPIRE")
            .arg(&key)
            .arg(7 * 24 * 3600)
            .query_async::<()>(&mut conn)
            .await?;

        // Rastreia sessões que "namoraram" este produto
        let namoro_key = format!(
            "analytics:store:{}:product:{}:namoro_sessions",
            store_id, product_id
        );
        redis::cmd("SADD")
            .arg(&namoro_key)
            .arg(session_id)
            .query_async::<()>(&mut conn)
            .await?;
        redis::cmd("EXPIRE")
            .arg(&namoro_key)
            .arg(7 * 24 * 3600)
            .query_async::<()>(&mut conn)
            .await?;

        Ok(())
    }

    /// Verifica se sessão já visitou o produto (para detectar revisita)
    pub async fn has_visited_product(
        &self,
        store_id: i32,
        session_id: &str,
        product_id: &str,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.get_conn().await?;
        let key = redis_key_session_products(store_id, session_id);
        let result: bool = redis::cmd("SISMEMBER")
            .arg(&key)
            .arg(product_id)
            .query_async(&mut conn)
            .await?;
        Ok(result)
    }

    /// Obtém contagem de views de um produto
    pub async fn get_product_views(
        &self,
        store_id: i32,
        product_id: &str,
    ) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.get_conn().await?;
        let key = redis_key_product_views(store_id, product_id);
        let count: Option<i64> = redis::cmd("GET")
            .arg(&key)
            .query_async(&mut conn)
            .await?;
        Ok(count.unwrap_or(0))
    }

    /// Obtém visitantes únicos de um produto (HyperLogLog)
    pub async fn get_product_unique_visitors(
        &self,
        store_id: i32,
        product_id: &str,
    ) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.get_conn().await?;
        let key = redis_key_product_visitors(store_id, product_id);
        let count: i64 = redis::cmd("PFCOUNT")
            .arg(&key)
            .query_async(&mut conn)
            .await?;
        Ok(count)
    }

    /// Flush: move dados quentes do Redis para Sled (persistência)
    /// Chamado periodicamente pelo AnalyticsFlushWorker
    pub async fn flush_to_sled(&self, store_id: i32) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.get_conn().await?;
        let key = redis_key_event_stream(store_id);

        // Pega todos os eventos do stream
        let events: Vec<String> = redis::cmd("LRANGE")
            .arg(&key)
            .arg(0)
            .arg(-1)
            .query_async(&mut conn)
            .await?;

        if events.is_empty() {
            return Ok(0);
        }

        let count = events.len();

        // Salva no Sled com chave baseada em timestamp
        let tree = self.sled_db.open_tree(format!("store:{}", store_id))?;
        let batch_key = format!("events:{}:{}", store_id, chrono::Utc::now().timestamp_millis());
        let batch_json = serde_json::to_vec(&events)?;
        tree.insert(batch_key.as_bytes(), batch_json)?;
        tree.flush()?;

        // Limpa o stream do Redis após persistir
        redis::cmd("DEL")
            .arg(&key)
            .query_async::<()>(&mut conn)
            .await?;

        tracing::info!(
            store_id = store_id,
            events_flushed = count,
            "Analytics flush to Sled completed"
        );

        Ok(count)
    }

    /// Lê dados persistidos no Sled para um período
    pub fn read_persisted_events(
        &self,
        store_id: i32,
    ) -> Result<Vec<Vec<String>>, Box<dyn std::error::Error + Send + Sync>> {
        let tree = self.sled_db.open_tree(format!("store:{}", store_id))?;
        let prefix = format!("events:{}", store_id);
        let mut all_events = Vec::new();

        for item in tree.scan_prefix(prefix.as_bytes()) {
            let (_, value) = item?;
            let events: Vec<String> = serde_json::from_slice(&value)?;
            all_events.push(events);
        }

        Ok(all_events)
    }

    /// Calcula lead score para um session_id
    /// Score baseado em: views, revisitas (namoro), add-to-cart, checkout_start
    pub async fn calculate_lead_score(
        &self,
        store_id: i32,
        session_id: &str,
    ) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.get_conn().await?;
        let mut score: f64 = 0.0;

        // Quantos produtos visitou
        let session_key = redis_key_session_products(store_id, session_id);
        let products_visited: i64 = redis::cmd("SCARD")
            .arg(&session_key)
            .query_async(&mut conn)
            .await
            .unwrap_or(0);
        score += products_visited as f64 * 1.0;

        // Eventos do session (buscar no stream)
        let events_key = redis_key_event_stream(store_id);
        let all_events: Vec<String> = redis::cmd("LRANGE")
            .arg(&events_key)
            .arg(0)
            .arg(-1)
            .query_async(&mut conn)
            .await
            .unwrap_or_default();

        for event_json in &all_events {
            if let Ok(evt) = serde_json::from_str::<AnalyticsEvent>(event_json) {
                if evt.session_id == session_id {
                    match evt.event_type.as_str() {
                        "product_detail_expand" => score += 2.0,
                        "product_revisit" => score += 5.0, // namoro vale muito
                        "cart_add" => score += 10.0,
                        "checkout_start" => score += 20.0,
                        _ => {}
                    }
                }
            }
        }

        Ok(score)
    }
}
