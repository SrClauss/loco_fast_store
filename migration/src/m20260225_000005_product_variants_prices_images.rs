use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    #[allow(clippy::too_many_lines)]
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ProductVariants::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ProductVariants::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ProductVariants::Pid)
                            .uuid()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(ProductVariants::ProductId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ProductVariants::Sku)
                            .string_len(128)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ProductVariants::Title)
                            .string_len(256)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ProductVariants::OptionValues)
                            .json_binary()
                            .not_null()
                            .default("{}"),
                    )
                    .col(
                        ColumnDef::new(ProductVariants::InventoryQuantity)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(ProductVariants::AllowBackorder)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(ProductVariants::Weight).decimal())
                    .col(ColumnDef::new(ProductVariants::Dimensions).json_binary())
                    .col(
                        ColumnDef::new(ProductVariants::Metadata)
                            .json_binary()
                            .not_null()
                            .default("{}"),
                    )
                    .col(
                        ColumnDef::new(ProductVariants::SortOrder)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(ProductVariants::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(ProductVariants::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(ProductVariants::DeletedAt).timestamp_with_time_zone())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_variants_product")
                            .from(ProductVariants::Table, ProductVariants::ProductId)
                            .to(Products::Table, Products::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_variants_product")
                    .table(ProductVariants::Table)
                    .col(ProductVariants::ProductId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_variants_sku")
                    .table(ProductVariants::Table)
                    .col(ProductVariants::Sku)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Prices table
        manager
            .create_table(
                Table::create()
                    .table(Prices::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Prices::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Prices::Pid).uuid().not_null().unique_key())
                    .col(ColumnDef::new(Prices::VariantId).integer().not_null())
                    .col(ColumnDef::new(Prices::Amount).big_integer().not_null()) // centavos
                    .col(
                        ColumnDef::new(Prices::Currency)
                            .string_len(3)
                            .not_null()
                            .default("BRL"),
                    )
                    .col(ColumnDef::new(Prices::Region).string_len(10))
                    .col(
                        ColumnDef::new(Prices::MinQuantity)
                            .integer()
                            .not_null()
                            .default(1),
                    )
                    .col(ColumnDef::new(Prices::MaxQuantity).integer())
                    .col(ColumnDef::new(Prices::StartsAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Prices::EndsAt).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(Prices::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Prices::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_prices_variant")
                            .from(Prices::Table, Prices::VariantId)
                            .to(ProductVariants::Table, ProductVariants::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_prices_variant")
                    .table(Prices::Table)
                    .col(Prices::VariantId)
                    .to_owned(),
            )
            .await?;

        // Product Images table
        manager
            .create_table(
                Table::create()
                    .table(ProductImages::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ProductImages::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ProductImages::Pid)
                            .uuid()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(ProductImages::ProductId)
                            .integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ProductImages::VariantId).integer())
                    .col(
                        ColumnDef::new(ProductImages::Url)
                            .string_len(1024)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ProductImages::AltText)
                            .string_len(256)
                            .not_null()
                            .default(""),
                    )
                    .col(
                        ColumnDef::new(ProductImages::SortOrder)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(ProductImages::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(ProductImages::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_images_product")
                            .from(ProductImages::Table, ProductImages::ProductId)
                            .to(Products::Table, Products::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_images_variant")
                            .from(ProductImages::Table, ProductImages::VariantId)
                            .to(ProductVariants::Table, ProductVariants::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ProductImages::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Prices::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ProductVariants::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(Iden)]
enum ProductVariants {
    Table,
    Id,
    Pid,
    ProductId,
    Sku,
    Title,
    OptionValues,
    InventoryQuantity,
    AllowBackorder,
    Weight,
    Dimensions,
    Metadata,
    SortOrder,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}

#[derive(Iden)]
enum Prices {
    Table,
    Id,
    Pid,
    VariantId,
    Amount,
    Currency,
    Region,
    MinQuantity,
    MaxQuantity,
    StartsAt,
    EndsAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum ProductImages {
    Table,
    Id,
    Pid,
    ProductId,
    VariantId,
    Url,
    AltText,
    SortOrder,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Products {
    Table,
    Id,
}
