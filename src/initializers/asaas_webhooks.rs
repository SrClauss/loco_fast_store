use async_trait::async_trait;
use axum::Router as AxumRouter;
use loco_rs::{app::{AppContext, Initializer}, Result};
use std::{env, fs, path::PathBuf};

use crate::services::asaas::AsaasClient;

pub struct AsaasWebhooksInitializer;

#[async_trait]
impl Initializer for AsaasWebhooksInitializer {
    fn name(&self) -> String {
        "asaas-webhooks".to_string()
    }

    async fn after_routes(&self, router: AxumRouter, _ctx: &AppContext) -> Result<AxumRouter> {
        // load variables from .env so later env::var calls pick them up
        let _ = dotenvy::dotenv();
        // Ensure .env and .env.example exist (create from ASAAS_TOKEN if available)
        let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let env_path = cwd.join(".env");
        let example_path = cwd.join(".env.example");

        if !env_path.exists() {
            if let Ok(token) = env::var("ASAAS_TOKEN") {
                let base = env::var("ASAAS_BASE_URL")
                    .unwrap_or_else(|_| "https://api-sandbox.asaas.com/v3".to_string());
                // include webhook auth token if provided or use default from env var ASAAS_WEBHOOK_AUTH_TOKEN
                let webhook_auth = env::var("ASAAS_WEBHOOK_AUTH_TOKEN").unwrap_or_else(|_| "77aab49d-ed15-45d1-8633-a322441f99bd".to_string());
                let webhook_email = env::var("ASAAS_WEBHOOK_EMAIL").unwrap_or_else(|_| "clausemberg@yahoo.com.br".to_string());
                let content = format!("ASAAS_API_KEY='{}'\nASAAS_BASE_URL='{}'\nASAAS_WEBHOOK_AUTH_TOKEN='{}'\nASAAS_WEBHOOK_EMAIL='{}'\n", token, base, webhook_auth, webhook_email);
                let _ = fs::write(&env_path, content);
                println!("Wrote .env with ASAAS_API_KEY and ASAAS_WEBHOOK_AUTH_TOKEN (values hidden)");
            }
        }

        if !example_path.exists() {
            let example = "# Example Asaas env\nASAAS_API_KEY=your_api_key_here\nASAAS_BASE_URL=https://api-sandbox.asaas.com/v3\n# Token used to authorize incoming webhook requests\nASAAS_WEBHOOK_AUTH_TOKEN=77aab49d-ed15-45d1-8633-a322441f99bd\n# Email to receive errors/notifications for webhooks\nASAAS_WEBHOOK_EMAIL=clausemberg@yahoo.com.br\nASAAS_WEBHOOK_URL=https://yourdomain.com/api/payments/asaas/webhook\n";
            let _ = fs::write(&example_path, example);
        }

        // Register webhooks if ASAAS_WEBHOOK_URL is set
        let webhook_url = match env::var("ASAAS_WEBHOOK_URL") {
            Ok(v) if !v.is_empty() => v,
            _ => {
                println!("ASAAS_WEBHOOK_URL not set — skipping webhook auto-registration");
                return Ok(router);
            }
        };

        let client = match AsaasClient::from_env() {
            Ok(c) => c,
            Err(e) => {
                println!("AsaasClient init failed: {} — skipping webhook registration", e);
                return Ok(router);
            }
        };

        let existing = match client.list_webhooks().await {
            Ok(j) => j,
            Err(e) => {
                println!("Asaas list_webhooks failed: {} — skipping webhook registration", e);
                return Ok(router);
            }
        };

        let data = existing.get("data").and_then(|d| d.as_array()).cloned().unwrap_or_default();

        let required = vec!["PAYMENT_CONFIRMED", "CHECKOUT_CREATED"];

        let webhook_auth_token = env::var("ASAAS_WEBHOOK_AUTH_TOKEN").ok();

        for ev in required {
            let mut found = false;
            for w in &data {
                if let Some(u) = w.get("url").and_then(|s| s.as_str()) {
                    if u != webhook_url {
                        continue;
                    }
                } else {
                    continue;
                }

                if let Some(events) = w.get("events").and_then(|e| e.as_array()) {
                    for item in events {
                        if item.as_str() == Some(ev) {
                            found = true;
                            break;
                        }
                    }
                }

                if found {
                    break;
                }
            }

            if !found {
                let name = format!("auto_{}_{}", ev.to_lowercase(), chrono::Utc::now().timestamp());
                let email = env::var("ASAAS_WEBHOOK_EMAIL")
                    .unwrap_or_else(|_| "clausemberg@yahoo.com.br".to_string());
                match client
                    .create_webhook(&name, &webhook_url, &[ev], webhook_auth_token.as_deref(), &email, "false")
                    .await
                {
                    Ok(_) => println!("Created Asaas webhook for event {}", ev),
                    Err(e) => println!("Failed to create webhook {}: {}", ev, e),
                }
            } else {
                println!("Webhook for event {} already exists for URL", ev);
            }
        }

        Ok(router)
    }
}
