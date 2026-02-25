use chrono::{Duration, Utc};
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};

#[allow(non_snake_case)]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AsaasCustomer {
    pub id: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub externalReference: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AsaasPayment {
    pub id: String,
    pub status: Option<String>,
    #[serde(default)]
    pub invoiceUrl: Option<String>,
    #[serde(default)]
    pub bankSlipUrl: Option<String>,
    #[serde(default)]
    pub pixQrCodeId: Option<String>,
    #[serde(default)]
    pub pixQrCode: Option<String>,
    #[serde(default)]
    pub checkoutUrl: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize)]
pub struct AsaasWebhookPayment {
    pub id: Option<String>,
    pub status: Option<String>,
    pub externalReference: Option<String>,
    pub customer: Option<String>,
    pub value: Option<f64>,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize)]
pub struct AsaasWebhookPayload {
    pub event: Option<String>,
    #[serde(default)]
    pub payment: Option<AsaasWebhookPayment>,
}

#[derive(Clone)]
pub struct AsaasClient {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
    pub webhook_secret: Option<String>,
}

impl AsaasClient {
    pub fn from_env() -> Result<Self> {
        // ensure the .env file is loaded in case caller didn't use the CLI entry point
        crate::env::load();

        let api_key = std::env::var("ASAAS_API_KEY")
            .map_err(|_| Error::Message("Faltou ASAAS_API_KEY".to_string()))?;
        let base_url = std::env::var("ASAAS_BASE_URL")
            .unwrap_or_else(|_| "https://sandbox.asaas.com/api/v3".to_string());
        let webhook_secret = std::env::var("ASAAS_WEBHOOK_SECRET").ok();

        // build reqwest client with a default user agent to satisfy Asaas requirement
        let client = reqwest::Client::builder()
            .user_agent("LocoFastStore/1.0")
            .build()
            .map_err(|e| Error::Message(format!("Asaas client build erro: {}", e)))?;

        Ok(Self {
            client,
            base_url,
            api_key,
            webhook_secret,
        })
    }

    fn auth(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        // add required headers (user-agent already handled by client builder)
        req.header("access_token", &self.api_key)
            .header("Content-Type", "application/json")
    }

    pub async fn list_webhooks(&self) -> Result<serde_json::Value> {
        let url = format!("{}/webhooks", self.base_url);
        let req_builder = self.auth(self.client.get(&url));
        // debug: show headers we'll send
        if tracing::log::log_enabled!(tracing::log::Level::Debug) {
            if let Some(rb) = req_builder.try_clone() {
                if let Ok(req) = rb.build() {
                    tracing::debug!("list_webhooks headers: {:?}", req.headers());
                }
            }
        }
        let res = req_builder
            .send()
            .await
            .map_err(|e| Error::Message(format!("Asaas list_webhooks erro: {}", e)))?;

        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        tracing::debug!("list_webhooks response status={:?} body={}", status, body);
        if !status.is_success() {
            return Err(Error::Message(format!("Asaas list_webhooks falhou: {}", body)));
        }

        serde_json::from_str(&body)
            .map_err(|e| Error::Message(format!("Asaas parse list_webhooks: {}", e)))
    }

    pub async fn create_webhook(
        &self,
        name: &str,
        url: &str,
        events: &[&str],
        auth_token: Option<&str>,
        email: &str,
        interrupted: &str,
    ) -> Result<serde_json::Value> {
        #[derive(Serialize)]
        struct Payload<'a> {
            name: &'a str,
            url: &'a str,
            events: &'a [&'a str],
            enabled: bool,
            #[serde(rename = "apiVersion")]
            api_version: u8,
            #[serde(rename = "sendType")]
            send_type: &'a str,
            email: &'a str,
            interrupted: &'a str,
            #[serde(skip_serializing_if = "Option::is_none", rename = "authToken")]
            auth_token: Option<&'a str>,
        }

        let payload = Payload {
            name,
            url,
            events,
            enabled: true,
            api_version: 3,
            send_type: "NON_SEQUENTIALLY",
            email,
            interrupted,
            auth_token,
        };

        let endpoint = format!("{}/webhooks", self.base_url);
        // log the JSON payload at INFO level so it shows up even when debug
        // filtering is strict. this helps verify what is actually sent to Asaas.
        if let Ok(p) = serde_json::to_string(&payload) {
            tracing::info!("create_webhook payload: {}", p);
        }
        let res = self
            .auth(self.client.post(endpoint))
            .json(&payload)
            .send()
            .await
            .map_err(|e| Error::Message(format!("Asaas create_webhook erro: {}", e)))?;

        let status = res.status();
        let body = res.text().await.unwrap_or_default();

        if !status.is_success() {
            return Err(Error::Message(format!("Asaas create_webhook falhou: {}", body)));
        }

        serde_json::from_str(&body)
            .map_err(|e| Error::Message(format!("Asaas parse create_webhook: {}", e)))
    }

    pub async fn create_customer(
        &self,
        name: &str,
        email: &str,
        phone: Option<&str>,
        external_reference: Option<String>,
    ) -> Result<AsaasCustomer> {
        #[allow(non_snake_case)]
        #[derive(Serialize)]
        struct Payload<'a> {
            name: &'a str,
            email: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            phone: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            externalReference: Option<String>,
        }

        let payload = Payload {
            name,
            email,
            phone,
            externalReference: external_reference,
        };

        let url = format!("{}/customers", self.base_url);
        let res = self
            .auth(self.client.post(url))
            .json(&payload)
            .send()
            .await
            .map_err(|e| Error::Message(format!("Asaas create_customer erro: {}", e)))?;

        if !res.status().is_success() {
            let body: String = res.text().await.unwrap_or_default();
            return Err(Error::Message(format!("Asaas create_customer falhou: {}", body)));
        }

        res.json::<AsaasCustomer>()
            .await
            .map_err(|e| Error::Message(format!("Asaas parse customer: {}", e)))
    }

    pub async fn create_payment(
        &self,
        customer_id: &str,
        value_cents: i64,
        description: &str,
        external_reference: &str,
        billing_type: &str,
        due_date: Option<String>,
    ) -> Result<AsaasPayment> {
        #[allow(non_snake_case)]
        #[derive(Serialize)]
        struct Payload<'a> {
            customer: &'a str,
            billingType: &'a str,
            value: f64,
            dueDate: String,
            description: &'a str,
            externalReference: &'a str,
        }

        let value = (value_cents as f64) / 100.0;
        let due = if let Some(date) = due_date {
            date
        } else {
            (Utc::now() + Duration::days(2)).date_naive().to_string()
        };

        let payload = Payload {
            customer: customer_id,
            billingType: billing_type,
            value,
            dueDate: due,
            description,
            externalReference: external_reference,
        };

        let url = format!("{}/payments", self.base_url);
        let res = self
            .auth(self.client.post(url))
            .json(&payload)
            .send()
            .await
            .map_err(|e| Error::Message(format!("Asaas create_payment erro: {}", e)))?;

        if !res.status().is_success() {
            let body: String = res.text().await.unwrap_or_default();
            return Err(Error::Message(format!("Asaas create_payment falhou: {}", body)));
        }

        res.json::<AsaasPayment>()
            .await
            .map_err(|e| Error::Message(format!("Asaas parse payment: {}", e)))
    }

    pub fn map_status(&self, asaas_status: Option<&str>, event: Option<&str>) -> String {
        if let Some(ev) = event {
            if ev.eq_ignore_ascii_case("PAYMENT_CONFIRMED") {
                return "paid".to_string();
            }
        }

        match asaas_status.unwrap_or("") {
            "RECEIVED" | "CONFIRMED" | "RECEIVED_IN_CASH" | "RECEIVED_PIX" => "paid".to_string(),
            "PENDING" | "AWAITING_RISK_ANALYSIS" => "awaiting".to_string(),
            "OVERDUE" => "overdue".to_string(),
            "REFUNDED" | "CHARGEBACK" | "CHARGEBACK_DISPUTE" | "CHARGEBACK_REQUESTED" => {
                "refunded".to_string()
            }
            _ => "awaiting".to_string(),
        }
    }
}
