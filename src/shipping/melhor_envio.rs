//! Integração com MelhorEnvio
//!
//! Status: **stub** — estrutura pronta para implementação completa.
//!
//! # Variáveis de ambiente necessárias
//!
//! | Variável                | Descrição                           | Exemplo                        |
//! |-------------------------|-------------------------------------|--------------------------------|
//! | `MELHOR_ENVIO_TOKEN`    | Token OAuth2 (Bearer) da conta      | `eyJ0eXAiOi...`               |
//! | `MELHOR_ENVIO_SANDBOX`  | Usar ambiente de testes (`true`)    | `true`                         |
//!
//! # Documentação oficial
//!
//! - API: <https://docs.menv.io/>
//! - Sandbox: <https://sandbox.melhorenvio.com.br>
//! - Produção: <https://melhorenvio.com.br>
//!
//! # Como completar a implementação
//!
//! 1. Obtenha um token OAuth2 em <https://melhorenvio.com.br/painel/gerenciar/tokens>
//! 2. Implemente `calculate_freight` chamando `POST /api/v2/me/shipment/calculate`
//! 3. Implemente `create_shipment` chamando `POST /api/v2/me/cart` + `POST /api/v2/me/shipment/checkout`
//! 4. Implemente `track` chamando `GET /api/v2/me/shipment/tracking`

use async_trait::async_trait;

use super::{
    CreateShipmentParams, FreightOption, FreightParams, ShipmentResult, ShippingError,
    ShippingProvider, TrackingInfo,
};

const SANDBOX_BASE: &str = "https://sandbox.melhorenvio.com.br/api/v2";
const PROD_BASE: &str = "https://melhorenvio.com.br/api/v2";

pub struct MelhorEnvio {
    token: String,
    base_url: String,
}

impl MelhorEnvio {
    pub fn new(token: String, sandbox: bool) -> Self {
        Self {
            token,
            base_url: if sandbox { SANDBOX_BASE } else { PROD_BASE }.to_string(),
        }
    }

    /// Constrói cliente HTTP com header de autorização
    fn client(&self) -> reqwest::Client {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", self.token).parse().unwrap(),
        );
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            "application/json".parse().unwrap(),
        );
        headers.insert(
            reqwest::header::HeaderName::from_static("user-agent"),
            "LocoFastStore/1.0 (contato@example.com)".parse().unwrap(),
        );
        reqwest::Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("falha ao construir reqwest::Client")
    }
}

#[async_trait]
impl ShippingProvider for MelhorEnvio {
    fn name(&self) -> &'static str {
        "melhor_envio"
    }

    /// Calcula fretes disponíveis.
    ///
    /// Endpoint: `POST /api/v2/me/shipment/calculate`
    ///
    /// Corpo esperado pela API:
    /// ```json
    /// {
    ///   "from": { "postal_code": "01310100" },
    ///   "to":   { "postal_code": "30130010" },
    ///   "package": {
    ///     "height": 10, "width": 15, "length": 20, "weight": 0.3
    ///   },
    ///   "options": { "insurance_value": 50.00 }
    /// }
    /// ```
    async fn calculate_freight(
        &self,
        params: FreightParams,
    ) -> Result<Vec<FreightOption>, ShippingError> {
        let body = serde_json::json!({
            "from": { "postal_code": params.origin_postal_code },
            "to":   { "postal_code": params.destination_postal_code },
            "package": {
                "height": params.height_cm,
                "width":  params.width_cm,
                "length": params.length_cm,
                "weight": params.weight_grams as f64 / 1000.0
            },
            "options": {
                "insurance_value": params.declared_value_cents as f64 / 100.0
            },
            "services": "1,2,3,4,7,8" // Correios + Jadlog
        });

        let resp = self
            .client()
            .post(format!("{}/me/shipment/calculate", self.base_url))
            .json(&body)
            .send()
            .await
            .map_err(|e| ShippingError::Network(e.to_string()))?;

        if !resp.status().is_success() {
            let msg = resp.text().await.unwrap_or_default();
            return Err(ShippingError::Network(msg));
        }

        let data: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ShippingError::Parse(e.to_string()))?;

        let options = data
            .as_array()
            .ok_or_else(|| ShippingError::Parse("resposta não é array".into()))?
            .iter()
            .filter_map(|item| {
                // Ignora serviços com erro (ex.: pacote fora do limite)
                if item.get("error").is_some() {
                    return None;
                }
                Some(FreightOption {
                    carrier: item["company"]["name"].as_str().unwrap_or("").to_string(),
                    service: item["name"].as_str().unwrap_or("").to_string(),
                    service_code: item["id"].as_u64().unwrap_or(0).to_string(),
                    price_cents: (item["price"].as_f64().unwrap_or(0.0) * 100.0) as i64,
                    delivery_days: item["delivery_time"].as_u64().unwrap_or(0) as u32,
                    currency: "BRL".to_string(),
                })
            })
            .collect();

        Ok(options)
    }

    /// Cria etiqueta de envio.
    ///
    /// Fluxo MelhorEnvio:
    /// 1. `POST /api/v2/me/cart`  → adiciona ao carrinho ME
    /// 2. `POST /api/v2/me/shipment/checkout` → confirma pagamento (saldo ME)
    /// 3. `POST /api/v2/me/shipment/generate` → gera etiqueta
    ///
    /// TODO: implementar cada passo chamando os endpoints acima.
    async fn create_shipment(
        &self,
        _params: CreateShipmentParams,
    ) -> Result<ShipmentResult, ShippingError> {
        // ── STUB ────────────────────────────────────────────────────────────
        // Implemente aqui o fluxo de 3 passos descrito no doc acima.
        // ────────────────────────────────────────────────────────────────────
        Err(ShippingError::NotConfigured(
            "create_shipment do MelhorEnvio ainda não implementado. \
             Siga os comentários em src/shipping/melhor_envio.rs"
                .into(),
        ))
    }

    /// Rastreia envio pelo código.
    ///
    /// Endpoint: `GET /api/v2/me/shipment/tracking?orders[]=<order_id>`
    ///
    /// TODO: implementar chamada ao endpoint de rastreamento.
    async fn track(&self, _tracking_code: &str) -> Result<TrackingInfo, ShippingError> {
        Err(ShippingError::NotConfigured(
            "track do MelhorEnvio ainda não implementado. \
             Siga os comentários em src/shipping/melhor_envio.rs"
                .into(),
        ))
    }
}
