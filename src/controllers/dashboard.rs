use loco_rs::prelude::*;

/// Renders the admin dashboard page
#[debug_handler]
pub async fn dashboard_page(ViewEngine(v): ViewEngine<TeraView>) -> Result<Response> {
    format::render().view(
        &v,
        "admin/dashboard.html",
        serde_json::json!({"current_page": "dashboard"}),
    )
}

#[debug_handler]
pub async fn products_page(ViewEngine(v): ViewEngine<TeraView>) -> Result<Response> {
    format::render().view(
        &v,
        "admin/products/list.html",
        serde_json::json!({"current_page": "products"}),
    )
}

#[debug_handler]
pub async fn categories_page(ViewEngine(v): ViewEngine<TeraView>) -> Result<Response> {
    format::render().view(
        &v,
        "admin/categories/list.html",
        serde_json::json!({"current_page": "categories"}),
    )
}

#[debug_handler]
pub async fn collections_page(ViewEngine(v): ViewEngine<TeraView>) -> Result<Response> {
    format::render().view(
        &v,
        "admin/collections/list.html",
        serde_json::json!({"current_page": "collections"}),
    )
}

#[debug_handler]
pub async fn orders_page(ViewEngine(v): ViewEngine<TeraView>) -> Result<Response> {
    format::render().view(
        &v,
        "admin/orders/list.html",
        serde_json::json!({"current_page": "orders"}),
    )
}

#[debug_handler]
pub async fn carts_page(ViewEngine(v): ViewEngine<TeraView>) -> Result<Response> {
    format::render().view(
        &v,
        "admin/carts/list.html",
        serde_json::json!({"current_page": "carts"}),
    )
}

#[debug_handler]
pub async fn customers_page(ViewEngine(v): ViewEngine<TeraView>) -> Result<Response> {
    format::render().view(
        &v,
        "admin/customers/list.html",
        serde_json::json!({"current_page": "customers"}),
    )
}

#[debug_handler]
pub async fn stores_page(ViewEngine(v): ViewEngine<TeraView>) -> Result<Response> {
    format::render().view(
        &v,
        "admin/stores/list.html",
        serde_json::json!({"current_page": "stores"}),
    )
}

#[debug_handler]
pub async fn users_page(ViewEngine(v): ViewEngine<TeraView>) -> Result<Response> {
    format::render().view(
        &v,
        "admin/users/list.html",
        serde_json::json!({"current_page": "users"}),
    )
}

#[debug_handler]
pub async fn analytics_page(ViewEngine(v): ViewEngine<TeraView>) -> Result<Response> {
    format::render().view(
        &v,
        "admin/analytics/index.html",
        serde_json::json!({"current_page": "analytics"}),
    )
}

#[debug_handler]
pub async fn profile_page(ViewEngine(v): ViewEngine<TeraView>) -> Result<Response> {
    format::render().view(
        &v,
        "admin/profile/index.html",
        serde_json::json!({"current_page": "profile"}),
    )
}

#[debug_handler]
pub async fn settings_page(ViewEngine(v): ViewEngine<TeraView>) -> Result<Response> {
    format::render().view(
        &v,
        "admin/settings/index.html",
        serde_json::json!({"current_page": "settings"}),
    )
}

#[debug_handler]
pub async fn product_new_page(ViewEngine(v): ViewEngine<TeraView>) -> Result<Response> {
    format::render().view(
        &v,
        "admin/products/form.html",
        serde_json::json!({"current_page": "products", "product": null}),
    )
}

#[debug_handler]
pub async fn product_edit_page(ViewEngine(v): ViewEngine<TeraView>) -> Result<Response> {
    format::render().view(
        &v,
        "admin/products/form.html",
        serde_json::json!({"current_page": "products", "product": null}),
    )
}

#[debug_handler]
pub async fn product_import_page(ViewEngine(v): ViewEngine<TeraView>) -> Result<Response> {
    format::render().view(
        &v,
        "admin/products/import.html",
        serde_json::json!({"current_page": "products"}),
    )
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/admin")
        .add("/dashboard", get(dashboard_page))
        .add("/products", get(products_page))
        .add("/products/new", get(product_new_page))
        .add("/products/import", get(product_import_page))
        .add("/products/:id/edit", get(product_edit_page))
        .add("/categories", get(categories_page))
        .add("/collections", get(collections_page))
        .add("/orders", get(orders_page))
        .add("/carts", get(carts_page))
        .add("/customers", get(customers_page))
        .add("/stores", get(stores_page))
        .add("/users", get(users_page))
        .add("/analytics", get(analytics_page))
        .add("/profile", get(profile_page))
        .add("/settings", get(settings_page))
}
