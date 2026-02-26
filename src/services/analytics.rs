#![allow(
    clippy::uninlined_format_args,
    clippy::missing_errors_doc,
    clippy::doc_markdown
)]

//! Serviço de analytics: Moka (cache quente, in-memory) → Sled (persistência)
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

use moka::future::Cache;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsEvent {
    pub session_id: String,
    pub customer_id: Option<i32>,
    pub event_type: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
    pub metadata: serde_json::Value,
    pub timestamp: i64,
}

/// In-memory analytics backend powered by Moka
pub struct AnalyticsService {
    /// Event stream (equivalent to Redis LPUSH/LRANGE)
    event_stream: Arc<Mutex<Vec<String>>>,
    /// Integer counters (equivalent to Redis INCR/GET) — protected by a mutex for atomic updates
    counters: Arc<Mutex<HashMap<String, i64>>>,
    /// Sets per key (equivalent to Redis SADD/SISMEMBER/SCARD) — each set is atomically inserted via moka
    sets: Cache<String, Arc<Mutex<std::collections::HashSet<String>>>>,
    sled_db: Arc<sled::Db>,
}

impl AnalyticsService {
    /// Inicializa o serviço de analytics
    pub fn new(sled_path: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let sled_db = Arc::new(sled::open(sled_path)?);
        Ok(Self {
            event_stream: Arc::new(Mutex::new(Vec::new())),
            counters: Arc::new(Mutex::new(HashMap::new())),
            sets: Cache::builder().max_capacity(10_000).build(),
            sled_db,
        })
    }

    /// Incrementa contador para uma chave (operação atômica)
    async fn incr(&self, key: &str) {
        let mut map = self.counters.lock().await;
        let counter = map.entry(key.to_string()).or_insert(0);
        *counter += 1;
    }

    /// Adiciona membro a um set in-memory (operação atômica via moka get_with)
    async fn sadd(&self, key: &str, member: &str) {
        let key_owned = key.to_string();
        let set = self
            .sets
            .get_with(key_owned, async {
                Arc::new(Mutex::new(std::collections::HashSet::new()))
            })
            .await;
        set.lock().await.insert(member.to_string());
    }

    /// Verifica se membro pertence a um set
    async fn sismember(&self, key: &str, member: &str) -> bool {
        if let Some(set) = self.sets.get(key).await {
            set.lock().await.contains(member)
        } else {
            false
        }
    }

    /// Retorna cardinalidade de um set
    async fn scard(&self, key: &str) -> i64 {
        if let Some(set) = self.sets.get(key).await {
            set.lock().await.len() as i64
        } else {
            0
        }
    }

    /// Registra evento de analytics no cache in-memory
    pub async fn track_event(
        &self,
        event: &AnalyticsEvent,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let event_json = serde_json::to_string(event)?;
        self.event_stream.lock().await.push(event_json);

        if event.event_type == "product_view" {
            if let Some(ref entity_id) = event.entity_id {
                self.track_product_view(entity_id, &event.session_id).await;
            }
        }

        if event.event_type == "product_revisit" {
            if let Some(ref entity_id) = event.entity_id {
                self.track_product_revisit(entity_id, &event.session_id)
                    .await;
            }
        }

        Ok(())
    }

    /// Incrementa view de produto e rastreia sessão
    async fn track_product_view(&self, product_id: &str, session_id: &str) {
        self.incr(&format!("analytics:product:{}:views", product_id))
            .await;
        self.sadd(
            &format!("analytics:session:{}:products", session_id),
            product_id,
        )
        .await;
        self.sadd(
            &format!("analytics:product:{}:visitors", product_id),
            session_id,
        )
        .await;
    }

    /// Rastreia revisita (namoro) - session já visitou este produto antes
    async fn track_product_revisit(&self, product_id: &str, session_id: &str) {
        self.incr(&format!("analytics:product:{}:revisits", product_id))
            .await;
        self.sadd(
            &format!("analytics:product:{}:namoro_sessions", product_id),
            session_id,
        )
        .await;
    }

    /// Verifica se sessão já visitou o produto (para detectar revisita)
    pub async fn has_visited_product(
        &self,
        session_id: &str,
        product_id: &str,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self
            .sismember(
                &format!("analytics:session:{}:products", session_id),
                product_id,
            )
            .await)
    }

    /// Obtém contagem de views de um produto
    pub async fn get_product_views(
        &self,
        product_id: &str,
    ) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
        let map = self.counters.lock().await;
        Ok(*map
            .get(&format!("analytics:product:{}:views", product_id))
            .unwrap_or(&0))
    }

    /// Obtém visitantes únicos de um produto (aproximado via set cardinality)
    pub async fn get_product_unique_visitors(
        &self,
        product_id: &str,
    ) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self
            .scard(&format!("analytics:product:{}:visitors", product_id))
            .await)
    }

    /// Flush: move dados quentes do cache para Sled (persistência)
    /// Chamado periodicamente pelo AnalyticsFlushWorker
    pub async fn flush_to_sled(&self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let mut stream = self.event_stream.lock().await;
        if stream.is_empty() {
            return Ok(0);
        }

        let events: Vec<String> = stream.drain(..).collect();
        let count = events.len();

        let tree = self.sled_db.open_tree("analytics")?;
        let batch_key = format!("events:{}", chrono::Utc::now().timestamp_millis());
        let batch_json = serde_json::to_vec(&events)?;
        tree.insert(batch_key.as_bytes(), batch_json)?;
        tree.flush()?;

        tracing::info!(events_flushed = count, "Analytics flush to Sled completed");

        Ok(count)
    }

    /// Lê dados persistidos no Sled
    pub fn read_persisted_events(
        &self,
    ) -> Result<Vec<Vec<String>>, Box<dyn std::error::Error + Send + Sync>> {
        let tree = self.sled_db.open_tree("analytics")?;
        let prefix = "events:";
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
        session_id: &str,
    ) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
        let mut score: f64 = 0.0;

        let products_visited = self
            .scard(&format!("analytics:session:{}:products", session_id))
            .await;
        score += products_visited as f64;

        let stream = self.event_stream.lock().await;
        for event_json in stream.iter() {
            if let Ok(evt) = serde_json::from_str::<AnalyticsEvent>(event_json) {
                if evt.session_id == session_id {
                    match evt.event_type.as_str() {
                        "product_detail_expand" => score += 2.0,
                        "product_revisit" => score += 5.0,
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
