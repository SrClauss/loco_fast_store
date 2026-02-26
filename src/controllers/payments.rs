use axum::extract::Path;
use loco_rs::prelude::*;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    dto::response::ApiResponse,
    models::{
        _entities::{carts, customers},
        orders::Model as OrderModel,
    },
    services::{
        analytics::{AnalyticsEvent, AnalyticsService},
        asaas::{AsaasClient, AsaasWebhookPayload},
    },
};

#[derive(Debug, Deserialize)]
pub struct CreateAsaasPaymentParams {
    pub billing_type: Option<String>,
    pub due_date: Option<String>,
    pub description: Option<String>,
}

#[debug_handler]
async fn create_payment(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(order_pid): Path<Uuid>,
    Json(params): Json<CreateAsaasPaymentParams>,
) -> Result<Response> {
    let _user = crate::models::_entities::users::Model::find_by_pid(&ctx.db, &auth.claims.pid)
        .await?;
    let order = OrderModel::find_by_pid(&ctx.db, &order_pid).await?;

    let customer = customers::Entity::find_by_id(order.customer_id)
        .one(&ctx.db)
        .await?
        .ok_or(loco_rs::Error::NotFound)?;

    let client = AsaasClient::from_env()?;

    // Garante que temos um customer no Asaas e persiste o ID em metadata
    let mut metadata = customer.metadata.clone();
    let asaas_customer_id = if let Some(existing) = metadata
        .get("asaas_customer_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
    {
        existing
    } else {
        let full_name = format!("{} {}", customer.first_name, customer.last_name).trim().to_string();
        let created = client
            .create_customer(
                &full_name,
                &customer.email,
                customer.phone.as_deref(),
                Some(customer.pid.to_string()),
            )
            .await?;
        metadata["asaas_customer_id"] = serde_json::json!(created.id.clone());

        let mut active: customers::ActiveModel = customer.clone().into();
        active.metadata = ActiveValue::set(metadata.clone());
        let _ = active.update(&ctx.db).await?;
        created.id
    };

    let billing_type = params
        .billing_type
        .as_deref()
        .unwrap_or("PIX")
        .to_string();
    let description = params
        .description
        .as_deref()
        .unwrap_or("Pagamento de pedido")
        .to_string();

    let payment = client
        .create_payment(
            &asaas_customer_id,
            order.total,
            &description,
            &order.pid.to_string(),
            &billing_type,
            params.due_date,
        )
        .await?;

    let status = client.map_status(payment.status.as_deref(), None);
    let payment_data = serde_json::json!({
        "provider": "asaas",
        "payment_id": payment.id,
        "status": payment.status,
        "invoice_url": payment.invoiceUrl,
        "bank_slip_url": payment.bankSlipUrl,
        "pix_qr_code": payment.pixQrCode,
        "pix_qr_code_id": payment.pixQrCodeId,
        "checkout_url": payment.checkoutUrl,
    });

    let updated = OrderModel::update_payment_status(&ctx.db, order.id, &status, Some(payment_data))
        .await?;

    format::json(ApiResponse::success(serde_json::json!({
        "order_pid": updated.pid,
        "payment_status": updated.payment_status,
        "asaas_payment_id": payment.id,
        "invoice_url": payment.invoiceUrl,
        "bank_slip_url": payment.bankSlipUrl,
        "pix_qr_code": payment.pixQrCode,
        "checkout_url": payment.checkoutUrl,
    })))
}

#[debug_handler]
async fn webhook(
    State(ctx): State<AppContext>,
    Json(payload): Json<AsaasWebhookPayload>,
) -> Result<Response> {
    let Some(payment) = payload.payment else {
        return format::json(ApiResponse::<()>::error("NO_PAYMENT", "Payload sem payment"));
    };

    let Some(external_reference) = payment.externalReference.clone() else {
        return format::json(ApiResponse::<()>::error("NO_REFERENCE", "Sem externalReference"));
    };

    let Ok(order_pid) = Uuid::parse_str(&external_reference) else {
        return format::json(ApiResponse::<()>::error("BAD_REFERENCE", "externalReference inválido"));
    };

    let order = OrderModel::find_by_pid(&ctx.db, &order_pid).await?;
    let client = AsaasClient::from_env()?;
    let status = client.map_status(payment.status.as_deref(), payload.event.as_deref());

    let payment_data = serde_json::json!({
        "provider": "asaas",
        "event": payload.event,
        "payment": payment,
    });

    let updated = OrderModel::update_payment_status(&ctx.db, order.id, &status, Some(payment_data))
        .await?;

    if status == "paid" {
        if let Some(cart_id) = updated.cart_id {
            if let Some(cart) = carts::Entity::find_by_id(cart_id).one(&ctx.db).await? {
                let session_id = cart.session_id.clone();
                crate::env::load();
                let redis_url = std::env::var("REDIS_URL")
                    .unwrap_or_else(|_| "redis://127.0.0.1".to_string());
                let sled_path = std::env::var("SLED_PATH")
                    .unwrap_or_else(|_| "./data/analytics_sled".to_string());
                if let Ok(analytics) = AnalyticsService::new(&redis_url, &sled_path) {
                    let _ = analytics
                        .track_event(&AnalyticsEvent {
                            session_id,
                            customer_id: Some(updated.customer_id),
                            event_type: "checkout_complete".to_string(),
                            entity_type: Some("order".to_string()),
                            entity_id: Some(updated.pid.to_string()),
                            metadata: serde_json::json!({
                                "provider": "asaas",
                                "status": status,
                            }),
                            timestamp: chrono::Utc::now().timestamp(),
                        })
                        .await;
                }
            }
        }
    }

    format::json(ApiResponse::success(serde_json::json!({
        "order_pid": updated.pid,
        "payment_status": updated.payment_status,
    })))
}

#[debug_handler]
async fn list_asaas_webhooks(State(_ctx): State<AppContext>) -> Result<Response> {
    let client = AsaasClient::from_env()?;
    let webhooks = client.list_webhooks().await?;

    format::json(ApiResponse::success(webhooks))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/v1/orders/{order_pid}/payments/asaas", post(create_payment))
        .add("/payments/asaas/webhook", post(webhook))
        .add("/payments/asaas/webhooks", get(list_asaas_webhooks))
}
