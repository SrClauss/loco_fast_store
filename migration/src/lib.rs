#![allow(elided_lifetimes_in_paths)]
#![allow(clippy::wildcard_imports)]
pub use sea_orm_migration::prelude::*;
mod m20220101_000001_users;
mod m20260225_000002_stores;
mod m20260225_000003_categories;
mod m20260225_000004_products;
mod m20260225_000005_product_variants_prices_images;
mod m20260225_000006_collections;
mod m20260225_000007_customers_addresses;
mod m20260225_000008_carts;
mod m20260225_000009_orders;
mod m20260226_000010_store_collaborators_shippings;
mod m20260227_000011_remove_stores;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_users::Migration),
            Box::new(m20260225_000002_stores::Migration),
            Box::new(m20260225_000003_categories::Migration),
            Box::new(m20260225_000004_products::Migration),
            Box::new(m20260225_000005_product_variants_prices_images::Migration),
            Box::new(m20260225_000006_collections::Migration),
            Box::new(m20260225_000007_customers_addresses::Migration),
            Box::new(m20260225_000008_carts::Migration),
            Box::new(m20260225_000009_orders::Migration),
            Box::new(m20260226_000010_store_collaborators_shippings::Migration),
            Box::new(m20260227_000011_remove_stores::Migration),
            // inject-above (do not remove this comment)
        ]
    }
}
