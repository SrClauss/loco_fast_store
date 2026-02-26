use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    #[allow(clippy::too_many_lines)]
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Carts
        manager
            .create_table(
                Table::create()
                    .table(Carts::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Carts::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Carts::Pid).uuid().not_null().unique_key())
                    .col(ColumnDef::new(Carts::StoreId).integer().not_null())
                    .col(ColumnDef::new(Carts::CustomerId).integer())
                    .col(ColumnDef::new(Carts::SessionId).string_len(256).not_null())
                    .col(
                        ColumnDef::new(Carts::Status)
                            .string_len(20)
                            .not_null()
                            .default("active"),
                    )
                    .col(ColumnDef::new(Carts::Email).string_len(256))
                    .col(
                        ColumnDef::new(Carts::Currency)
                            .string_len(3)
                            .not_null()
                            .default("BRL"),
                    )
                    .col(
                        ColumnDef::new(Carts::Subtotal)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Carts::Tax)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Carts::Shipping)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Carts::Total)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Carts::Metadata)
                            .json_binary()
                            .not_null()
                            .default("{}"),
                    )
                    .col(ColumnDef::new(Carts::ExpiresAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Carts::CompletedAt).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(Carts::LastActivityAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(Carts::RecoveryToken).string_len(128))
                    .col(
                        ColumnDef::new(Carts::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Carts::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_carts_store")
                            .from(Carts::Table, Carts::StoreId)
                            .to(Stores::Table, Stores::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_carts_customer")
                            .from(Carts::Table, Carts::CustomerId)
                            .to(Customers::Table, Customers::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_carts_session")
                    .table(Carts::Table)
                    .col(Carts::StoreId)
                    .col(Carts::SessionId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_carts_abandoned")
                    .table(Carts::Table)
                    .col(Carts::Status)
                    .col(Carts::LastActivityAt)
                    .to_owned(),
            )
            .await?;

        // Cart Items
        manager
            .create_table(
                Table::create()
                    .table(CartItems::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(CartItems::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(CartItems::Pid)
                            .uuid()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(CartItems::CartId).integer().not_null())
                    .col(ColumnDef::new(CartItems::VariantId).integer().not_null())
                    .col(
                        ColumnDef::new(CartItems::Quantity)
                            .integer()
                            .not_null()
                            .default(1),
                    )
                    .col(
                        ColumnDef::new(CartItems::UnitPrice)
                            .big_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(CartItems::Total).big_integer().not_null())
                    .col(
                        ColumnDef::new(CartItems::Metadata)
                            .json_binary()
                            .not_null()
                            .default("{}"),
                    )
                    .col(
                        ColumnDef::new(CartItems::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(CartItems::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_cart_items_cart")
                            .from(CartItems::Table, CartItems::CartId)
                            .to(Carts::Table, Carts::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_cart_items_variant")
                            .from(CartItems::Table, CartItems::VariantId)
                            .to(ProductVariants::Table, ProductVariants::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_cart_items_cart")
                    .table(CartItems::Table)
                    .col(CartItems::CartId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CartItems::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Carts::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(Iden)]
enum Carts {
    Table,
    Id,
    Pid,
    StoreId,
    CustomerId,
    SessionId,
    Status,
    Email,
    Currency,
    Subtotal,
    Tax,
    Shipping,
    Total,
    Metadata,
    ExpiresAt,
    CompletedAt,
    LastActivityAt,
    RecoveryToken,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum CartItems {
    Table,
    Id,
    Pid,
    CartId,
    VariantId,
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
enum ProductVariants {
    Table,
    Id,
}
