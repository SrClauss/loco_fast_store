use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─── Store ───────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct StoreResponse {
    pub pid: Uuid,
    pub slug: String,
    pub name: String,
    pub domain: Option<String>,
    pub default_currency: String,
    pub status: String,
    pub config: serde_json::Value,
    pub created_at: String,
}

impl From<crate::models::_entities::stores::Model> for StoreResponse {
    fn from(m: crate::models::_entities::stores::Model) -> Self {
        Self {
            pid: m.pid,
            slug: m.slug,
            name: m.name,
            domain: m.domain,
            default_currency: m.default_currency,
            status: m.status,
            config: m.config,
            created_at: m.created_at.to_string(),
        }
    }
}

// ─── Category ────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct CategoryResponse {
    pub pid: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub parent_id: Option<i32>,
    pub image_url: Option<String>,
    pub sort_order: i32,
}

impl From<crate::models::_entities::categories::Model> for CategoryResponse {
    fn from(m: crate::models::_entities::categories::Model) -> Self {
        Self {
            pid: m.pid,
            name: m.name,
            slug: m.slug,
            description: m.description,
            parent_id: m.parent_id,
            image_url: m.image_url,
            sort_order: m.sort_order,
        }
    }
}

// ─── Product ─────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductResponse {
    pub pid: Uuid,
    pub title: String,
    pub slug: String,
    pub description: String,
    pub handle: String,
    pub status: String,
    pub product_type: String,
    pub category_id: Option<i32>,
    pub tags: serde_json::Value,
    pub featured: bool,
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variants: Option<Vec<VariantResponse>>,
}

impl From<crate::models::_entities::products::Model> for ProductResponse {
    fn from(m: crate::models::_entities::products::Model) -> Self {
        Self {
            pid: m.pid,
            title: m.title,
            slug: m.slug,
            description: m.description,
            handle: m.handle,
            status: m.status,
            product_type: m.product_type,
            category_id: m.category_id,
            tags: m.tags,
            featured: m.featured,
            seo_title: m.seo_title,
            seo_description: m.seo_description,
            metadata: m.metadata,
            created_at: m.created_at.to_string(),
            variants: None,
        }
    }
}

// ─── Variant ─────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct VariantResponse {
    pub pid: Uuid,
    pub sku: String,
    pub title: String,
    pub option_values: serde_json::Value,
    pub inventory_quantity: i32,
    pub allow_backorder: bool,
    pub sort_order: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prices: Option<Vec<PriceResponse>>,
}

impl From<crate::models::_entities::product_variants::Model> for VariantResponse {
    fn from(m: crate::models::_entities::product_variants::Model) -> Self {
        Self {
            pid: m.pid,
            sku: m.sku,
            title: m.title,
            option_values: m.option_values,
            inventory_quantity: m.inventory_quantity,
            allow_backorder: m.allow_backorder,
            sort_order: m.sort_order,
            prices: None,
        }
    }
}

// ─── Price ───────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct PriceResponse {
    pub pid: Uuid,
    pub amount: i64,
    pub currency: String,
    pub region: Option<String>,
    pub min_quantity: i32,
    pub max_quantity: Option<i32>,
}

impl From<crate::models::_entities::prices::Model> for PriceResponse {
    fn from(m: crate::models::_entities::prices::Model) -> Self {
        Self {
            pid: m.pid,
            amount: m.amount,
            currency: m.currency,
            region: m.region,
            min_quantity: m.min_quantity,
            max_quantity: m.max_quantity,
        }
    }
}

// ─── Customer ────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomerResponse {
    pub pid: Uuid,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub phone: Option<String>,
    pub has_account: bool,
    pub marketing_consent: bool,
    pub created_at: String,
}

impl From<crate::models::_entities::customers::Model> for CustomerResponse {
    fn from(m: crate::models::_entities::customers::Model) -> Self {
        Self {
            pid: m.pid,
            email: m.email,
            first_name: m.first_name,
            last_name: m.last_name,
            phone: m.phone,
            has_account: m.has_account,
            marketing_consent: m.marketing_consent,
            created_at: m.created_at.to_string(),
        }
    }
}

// ─── Cart ────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct CartResponse {
    pub pid: Uuid,
    pub session_id: String,
    pub status: String,
    pub email: Option<String>,
    pub currency: String,
    pub subtotal: i64,
    pub tax: i64,
    pub shipping: i64,
    pub total: i64,
    pub last_activity_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<CartItemResponse>>,
}

impl From<crate::models::_entities::carts::Model> for CartResponse {
    fn from(m: crate::models::_entities::carts::Model) -> Self {
        Self {
            pid: m.pid,
            session_id: m.session_id,
            status: m.status,
            email: m.email,
            currency: m.currency,
            subtotal: m.subtotal,
            tax: m.tax,
            shipping: m.shipping,
            total: m.total,
            last_activity_at: m.last_activity_at.to_string(),
            items: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CartItemResponse {
    pub pid: Uuid,
    pub variant_id: i32,
    pub quantity: i32,
    pub unit_price: i64,
    pub total: i64,
}

impl From<crate::models::_entities::cart_items::Model> for CartItemResponse {
    fn from(m: crate::models::_entities::cart_items::Model) -> Self {
        Self {
            pid: m.pid,
            variant_id: m.variant_id,
            quantity: m.quantity,
            unit_price: m.unit_price,
            total: m.total,
        }
    }
}

// ─── Order ───────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderResponse {
    pub pid: Uuid,
    pub order_number: String,
    pub status: String,
    pub payment_status: String,
    pub fulfillment_status: String,
    pub currency: String,
    pub subtotal: i64,
    pub tax: i64,
    pub shipping: i64,
    pub discount: i64,
    pub total: i64,
    pub payment_method: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub paid_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<OrderItemResponse>>,
}

impl From<crate::models::_entities::orders::Model> for OrderResponse {
    fn from(m: crate::models::_entities::orders::Model) -> Self {
        Self {
            pid: m.pid,
            order_number: m.order_number,
            status: m.status,
            payment_status: m.payment_status,
            fulfillment_status: m.fulfillment_status,
            currency: m.currency,
            subtotal: m.subtotal,
            tax: m.tax,
            shipping: m.shipping,
            discount: m.discount,
            total: m.total,
            payment_method: m.payment_method,
            notes: m.notes,
            created_at: m.created_at.to_string(),
            paid_at: m.paid_at.map(|t| t.to_string()),
            items: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderItemResponse {
    pub pid: Uuid,
    pub title: String,
    pub sku: String,
    pub quantity: i32,
    pub unit_price: i64,
    pub total: i64,
}

impl From<crate::models::_entities::order_items::Model> for OrderItemResponse {
    fn from(m: crate::models::_entities::order_items::Model) -> Self {
        Self {
            pid: m.pid,
            title: m.title,
            sku: m.sku,
            quantity: m.quantity,
            unit_price: m.unit_price,
            total: m.total,
        }
    }
}

// ─── Collection ──────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct CollectionResponse {
    pub pid: Uuid,
    pub title: String,
    pub slug: String,
    pub description: String,
}

impl From<crate::models::_entities::collections::Model> for CollectionResponse {
    fn from(m: crate::models::_entities::collections::Model) -> Self {
        Self {
            pid: m.pid,
            title: m.title,
            slug: m.slug,
            description: m.description,
        }
    }
}

// ─── Address ─────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct AddressResponse {
    pub pid: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub company: Option<String>,
    pub address_line_1: String,
    pub address_line_2: Option<String>,
    pub city: String,
    pub state: String,
    pub postal_code: String,
    pub country: String,
    pub phone: Option<String>,
    pub is_default_shipping: bool,
    pub is_default_billing: bool,
}

impl From<crate::models::_entities::addresses::Model> for AddressResponse {
    fn from(m: crate::models::_entities::addresses::Model) -> Self {
        Self {
            pid: m.pid,
            first_name: m.first_name,
            last_name: m.last_name,
            company: m.company,
            address_line_1: m.address_line_1,
            address_line_2: m.address_line_2,
            city: m.city,
            state: m.state,
            postal_code: m.postal_code,
            country: m.country,
            phone: m.phone,
            is_default_shipping: m.is_default_shipping,
            is_default_billing: m.is_default_billing,
        }
    }
}
