//! Módulo de integração com transportadoras.
//!
//! # Arquitetura
//!
//! Cada transportadora implementa o trait [`ShippingProvider`].
//! O sistema seleciona o provider pelo campo `carrier` do envio.
//!
//! # Como adicionar uma nova transportadora
//!
//! 1. Crie um arquivo `src/shipping/minha_transportadora.rs`
//! 2. Implemente o trait `ShippingProvider` para a sua struct
//! 3. Adicione o módulo em `mod.rs` e registre em `provider_for()`
//!
//! # Providers disponíveis
//!
//! | Carrier slug     | Status       | Módulo                     |
//! |------------------|--------------|----------------------------|
//! | `manual`         | ✅ pronto    | — (sem integração externa) |
//! | `melhor_envio`   | 🚧 stub      | `melhor_envio.rs`          |
//! | `correios_api`   | 🔜 planejado | (não implementado)         |

pub mod melhor_envio;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Opção de frete retornada pelo cálculo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreightOption {
    pub carrier: String,
    pub service: String,
    pub service_code: String,
    pub price_cents: i64,
    pub delivery_days: u32,
    pub currency: String,
}

/// Parâmetros para cálculo de frete
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreightParams {
    pub origin_postal_code: String,
    pub destination_postal_code: String,
    /// Peso em gramas
    pub weight_grams: u32,
    /// Dimensões em cm
    pub length_cm: u32,
    pub width_cm: u32,
    pub height_cm: u32,
    pub declared_value_cents: i64,
}

/// Parâmetros para criação de etiqueta de envio
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateShipmentParams {
    pub service_code: String,
    pub order_number: String,
    pub sender: ContactInfo,
    pub recipient: ContactInfo,
    pub freight: FreightParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactInfo {
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub document: Option<String>,
    pub address_line1: String,
    pub address_line2: Option<String>,
    pub city: String,
    pub state: String,
    pub postal_code: String,
    pub country: String,
}

/// Resultado da criação de envio
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipmentResult {
    /// ID único retornado pelo provider
    pub provider_id: String,
    pub tracking_code: Option<String>,
    pub tracking_url: Option<String>,
    pub label_url: Option<String>,
    /// Dados brutos do provider para armazenar em `provider_data`
    pub raw_data: serde_json::Value,
}

/// Evento de rastreamento
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingEvent {
    pub timestamp: String,
    pub status: String,
    pub description: String,
    pub location: Option<String>,
}

/// Informações de rastreamento
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingInfo {
    pub tracking_code: String,
    pub current_status: String,
    pub events: Vec<TrackingEvent>,
    pub estimated_delivery: Option<String>,
}

/// Erro de integração com provider
#[derive(Debug)]
pub enum ShippingError {
    NotConfigured(String),
    Network(String),
    Parse(String),
    UnsupportedCarrier(String),
}

impl std::fmt::Display for ShippingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotConfigured(m)     => write!(f, "Provider não configurado: {}", m),
            Self::Network(m)           => write!(f, "Erro de comunicação com o provider: {}", m),
            Self::Parse(m)             => write!(f, "Resposta inesperada do provider: {}", m),
            Self::UnsupportedCarrier(m)=> write!(f, "Carrier não suportado: {}", m),
        }
    }
}

impl std::error::Error for ShippingError {}

/// Trait que toda integração de transportadora deve implementar.
///
/// # Exemplo de implementação
///
/// ```rust,ignore
/// use async_trait::async_trait;
/// use crate::shipping::{ShippingProvider, FreightParams, FreightOption, ...};
///
/// pub struct MinhaTransportadora { api_key: String }
///
/// #[async_trait]
/// impl ShippingProvider for MinhaTransportadora {
///     fn name(&self) -> &'static str { "minha_transportadora" }
///
///     async fn calculate_freight(&self, params: FreightParams) -> Result<Vec<FreightOption>, ShippingError> {
///         // chame a API aqui
///         todo!()
///     }
///
///     async fn create_shipment(&self, params: CreateShipmentParams) -> Result<ShipmentResult, ShippingError> {
///         todo!()
///     }
///
///     async fn track(&self, tracking_code: &str) -> Result<TrackingInfo, ShippingError> {
///         todo!()
///     }
/// }
/// ```
#[async_trait]
pub trait ShippingProvider: Send + Sync {
    /// Slug identificador do provider (ex.: "melhor_envio")
    fn name(&self) -> &'static str;

    /// Calcula opções de frete
    async fn calculate_freight(
        &self,
        params: FreightParams,
    ) -> Result<Vec<FreightOption>, ShippingError>;

    /// Cria etiqueta de envio e retorna dados do provider
    async fn create_shipment(
        &self,
        params: CreateShipmentParams,
    ) -> Result<ShipmentResult, ShippingError>;

    /// Consulta rastreamento pelo código
    async fn track(&self, tracking_code: &str) -> Result<TrackingInfo, ShippingError>;
}

/// Retorna o provider correspondente ao slug, se disponível e configurado.
///
/// # Como registrar um novo provider
///
/// Adicione um braço ao `match` abaixo retornando sua implementação.
pub fn provider_for(carrier: &str) -> Option<Box<dyn ShippingProvider>> {
    match carrier {
        "melhor_envio" => {
            // Lê credenciais de variáveis de ambiente
            let token = std::env::var("MELHOR_ENVIO_TOKEN").ok()?;
            let sandbox = std::env::var("MELHOR_ENVIO_SANDBOX")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(true);
            Some(Box::new(melhor_envio::MelhorEnvio::new(token, sandbox)))
        }
        // "correios_api" => Some(Box::new(correios_api::CorreiosApi::new(...))),
        _ => None, // 'manual' e desconhecidos não têm provider externo
    }
}
