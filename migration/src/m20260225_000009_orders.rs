use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    #[allow(clippy::too_many_lines)]
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Orders
        manager
            .create_table(
                Table::create()
                    .table(Orders::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Orders::Id).integer().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new(Orders::Pid).uuid().not_null().unique_key())
                    .col(ColumnDef::new(Orders::StoreId).integer().not_null())
                    .col(ColumnDef::new(Orders::CustomerId).integer().not_null())
                    .col(ColumnDef::new(Orders::CartId).integer())
                    .col(ColumnDef::new(Orders::OrderNumber).string_len(64).not_null().unique_key())
                    .col(ColumnDef::new(Orders::Status).string_len(20).not_null().default("pending"))
                    .col(ColumnDef::new(Orders::PaymentStatus).string_len(20).not_null().default("pending"))
                    .col(ColumnDef::new(Orders::FulfillmentStatus).string_len(30).not_null().default("pending"))
                    .col(ColumnDef::new(Orders::Currency).string_len(3).not_null().default("BRL"))
                    .col(ColumnDef::new(Orders::Subtotal).big_integer().not_null().default(0))
                    .col(ColumnDef::new(Orders::Tax).big_integer().not_null().default(0))
                    .col(ColumnDef::new(Orders::Shipping).big_integer().not_null().default(0))
                    .col(ColumnDef::new(Orders::Discount).big_integer().not_null().default(0))
                    .col(ColumnDef::new(Orders::Total).big_integer().not_null().default(0))
                    .col(ColumnDef::new(Orders::ShippingAddressId).integer())
                    .col(ColumnDef::new(Orders::BillingAddressId).integer())
                    .col(ColumnDef::new(Orders::PaymentMethod).string_len(32))
                    .col(ColumnDef::new(Orders::PaymentProvider).string_len(32))
                    .col(ColumnDef::new(Orders::PaymentData).json_binary().not_null().default("{}"))
                    .col(ColumnDef::new(Orders::Notes).text())
                    .col(ColumnDef::new(Orders::Metadata).json_binary().not_null().default("{}"))
                    .col(ColumnDef::new(Orders::CanceledAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Orders::PaidAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Orders::CreatedAt).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(Orders::UpdatedAt).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_orders_store")
                            .from(Orders::Table, Orders::StoreId)
                            .to(Stores::Table, Stores::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_orders_customer")
                            .from(Orders::Table, Orders::CustomerId)
                            .to(Customers::Table, Customers::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_orders_cart")
                            .from(Orders::Table, Orders::CartId)
                            .to(Carts::Table, Carts::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_orders_shipping_addr")
                            .from(Orders::Table, Orders::ShippingAddressId)
                            .to(Addresses::Table, Addresses::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_orders_billing_addr")
                            .from(Orders::Table, Orders::BillingAddressId)
                            .to(Addresses::Table, Addresses::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_orders_store_created")
                    .table(Orders::Table)
                    .col(Orders::StoreId)
                    .col(Orders::CreatedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_orders_customer")
                    .table(Orders::Table)
                    .col(Orders::CustomerId)
                    .to_owned(),
            )
            .await?;

        // Order Items
        manager
            .create_table(
                Table::create()
                    .table(OrderItems::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(OrderItems::Id).integer().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new(OrderItems::Pid).uuid().not_null().unique_key())
                    .col(ColumnDef::new(OrderItems::OrderId).integer().not_null())
                    .col(ColumnDef::new(OrderItems::VariantId).integer())
                    .col(ColumnDef::new(OrderItems::Title).string_len(512).not_null())
                    .col(ColumnDef::new(OrderItems::Sku).string_len(128).not_null())
                    .col(ColumnDef::new(OrderItems::Quantity).integer().not_null())
                    .col(ColumnDef::new(OrderItems::UnitPrice).big_integer().not_null())
                    .col(ColumnDef::new(OrderItems::Total).big_integer().not_null())
                    .col(ColumnDef::new(OrderItems::Metadata).json_binary().not_null().default("{}"))
                    .col(ColumnDef::new(OrderItems::CreatedAt).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(OrderItems::UpdatedAt).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_order_items_order")
                            .from(OrderItems::Table, OrderItems::OrderId)
                            .to(Orders::Table, Orders::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_order_items_variant")
                            .from(OrderItems::Table, OrderItems::VariantId)
                            .to(ProductVariants::Table, ProductVariants::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_order_items_order")
                    .table(OrderItems::Table)
                    .col(OrderItems::OrderId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(OrderItems::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(Orders::Table).to_owned()).await?;
        Ok(())
    }
}

#[derive(Iden)]
enum Orders {
    Table,
    Id,
    Pid,
    StoreId,
    CustomerId,
    CartId,
    OrderNumber,
    Status,
    PaymentStatus,
    FulfillmentStatus,
    Currency,
    Subtotal,
    Tax,
    Shipping,
    Discount,
    Total,
    ShippingAddressId,
    BillingAddressId,
    PaymentMethod,
    PaymentProvider,
    PaymentData,
    Notes,
    Metadata,
    CanceledAt,
    PaidAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum OrderItems {
    Table,
    Id,
    Pid,
    OrderId,
    VariantId,
    Title,
    Sku,
    Quantity,
    UnitPrice,
    Total,
    Metadata,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Stores {
    Table,
    Id,
}

#[derive(Iden)]
enum Customers {
    Table,
    Id,
}

#[derive(Iden)]
enum Carts {
    Table,
    Id,
}

#[derive(Iden)]
enum Addresses {
    Table,
    Id,
}

#[derive(Iden)]
enum ProductVariants {
    Table,
    Id,
}
