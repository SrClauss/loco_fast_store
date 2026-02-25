use chrono::{Duration, Utc};
use loco_rs::prelude::*;
use sea_orm::{
    ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};
use serde::Serialize;
use std::collections::HashMap;

use crate::models::_entities::{
    customers, orders, products,
};

/// Resumo de um pedido recente para o dashboard
#[derive(Debug, Serialize)]
pub struct RecentOrderItem {
    pub id: i32,
    pub order_number: String,
    pub status: String,
    pub payment_status: String,
    pub total: i64,
    pub currency: String,
    pub customer_email: Option<String>,
    pub created_at: String,
}

/// Ponto de dado para o gráfico de receita diária
#[derive(Debug, Serialize)]
pub struct RevenuePoint {
    pub date: String,
    pub revenue: i64,
    pub orders: i64,
}

/// Payload completo do stats do dashboard
#[derive(Debug, Serialize)]
pub struct DashboardStats {
    pub total_revenue: i64,
    pub total_orders: u64,
    pub total_customers: u64,
    pub total_products: u64,
    pub pending_orders: u64,
    pub processing_orders: u64,
    pub delivered_orders: u64,
    pub cancelled_orders: u64,
    pub recent_orders: Vec<RecentOrderItem>,
    pub revenue_chart: Vec<RevenuePoint>,
}

/// GET /api/admin/dashboard/stats
/// Retorna dados reais do banco para o painel administrativo.
/// Não exige autenticação JWT (painel interno).
#[debug_handler]
pub async fn stats(State(ctx): State<AppContext>) -> Result<Response> {
    // ── Contagens básicas ──────────────────────────────────────────────────
    let total_orders = orders::Entity::find().count(&ctx.db).await?;

    let total_customers = customers::Entity::find()
        .filter(customers::Column::DeletedAt.is_null())
        .count(&ctx.db)
        .await?;

    let total_products = products::Entity::find()
        .filter(products::Column::DeletedAt.is_null())
        .filter(products::Column::Status.eq("active"))
        .count(&ctx.db)
        .await?;

    // ── Contagem por status de pedido ──────────────────────────────────────
    let pending_orders = orders::Entity::find()
        .filter(orders::Column::Status.eq("pending"))
        .count(&ctx.db)
        .await?;

    let processing_orders = orders::Entity::find()
        .filter(orders::Column::Status.is_in(["confirmed", "processing"]))
        .count(&ctx.db)
        .await?;

    let delivered_orders = orders::Entity::find()
        .filter(orders::Column::Status.eq("delivered"))
        .count(&ctx.db)
        .await?;

    let cancelled_orders = orders::Entity::find()
        .filter(orders::Column::Status.eq("cancelled"))
        .count(&ctx.db)
        .await?;

    // ── Receita total (todos pedidos não cancelados) ────────────────────────
    let all_orders = orders::Entity::find()
        .filter(orders::Column::Status.ne("cancelled"))
        .all(&ctx.db)
        .await?;

    let total_revenue: i64 = all_orders.iter().map(|o| o.total).sum();

    // ── Gráfico de receita dos últimos 30 dias ─────────────────────────────
    let thirty_days_ago = Utc::now() - Duration::days(30);
    let thirty_days_ago_dt: sea_orm::prelude::DateTimeWithTimeZone = thirty_days_ago.into();

    let recent_all_orders = orders::Entity::find()
        .filter(orders::Column::CreatedAt.gte(thirty_days_ago_dt))
        .filter(orders::Column::Status.ne("cancelled"))
        .all(&ctx.db)
        .await?;

    // Agrupa receita e contagem por data (YYYY-MM-DD)
    let mut daily: HashMap<String, (i64, i64)> = HashMap::new();
    for o in &recent_all_orders {
        let date = o.created_at.format("%Y-%m-%d").to_string();
        let entry = daily.entry(date).or_insert((0, 0));
        entry.0 += o.total;
        entry.1 += 1;
    }

    // Gera todos os 30 dias (inclusive dias sem pedidos)
    let mut revenue_chart: Vec<RevenuePoint> = (0..30)
        .rev()
        .map(|i| {
            let date = (Utc::now() - Duration::days(i)).format("%Y-%m-%d").to_string();
            let (rev, cnt) = daily.get(&date).copied().unwrap_or((0, 0));
            RevenuePoint {
                date,
                revenue: rev,
                orders: cnt,
            }
        })
        .collect();
    revenue_chart.sort_by(|a, b| a.date.cmp(&b.date));

    // ── Pedidos recentes (últimos 10) ──────────────────────────────────────
    let last_orders = orders::Entity::find()
        .order_by_desc(orders::Column::CreatedAt)
        .limit(10)
        .all(&ctx.db)
        .await?;

    // Coleta customer_ids para buscar e-mails
    let customer_ids: Vec<i32> = last_orders.iter().map(|o| o.customer_id).collect();
    let customers_list = customers::Entity::find()
        .filter(customers::Column::Id.is_in(customer_ids))
        .all(&ctx.db)
        .await?;
    let customer_map: HashMap<i32, String> = customers_list
        .into_iter()
        .map(|c| {
            let full = format!("{} {}", c.first_name, c.last_name);
            let name = full.trim().to_string();
            let label = if name.is_empty() { c.email } else { name };
            (c.id, label)
        })
        .collect();

    let recent_orders: Vec<RecentOrderItem> = last_orders
        .into_iter()
        .map(|o| {
            let customer_email = customer_map.get(&o.customer_id).cloned();
            RecentOrderItem {
                id: o.id,
                order_number: o.order_number,
                status: o.status,
                payment_status: o.payment_status,
                total: o.total,
                currency: o.currency,
                customer_email,
                created_at: o.created_at.to_rfc3339(),
            }
        })
        .collect();

    let stats = DashboardStats {
        total_revenue,
        total_orders,
        total_customers,
        total_products,
        pending_orders,
        processing_orders,
        delivered_orders,
        cancelled_orders,
        recent_orders,
        revenue_chart,
    };

    format::json(stats)
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/admin/dashboard")
        .add("/stats", get(stats))
}
