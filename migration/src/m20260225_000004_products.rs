use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Products::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Products::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Products::Pid).uuid().not_null().unique_key())
                    .col(ColumnDef::new(Products::StoreId).integer().not_null())
                    .col(ColumnDef::new(Products::Title).string_len(512).not_null())
                    .col(ColumnDef::new(Products::Slug).string_len(512).not_null())
                    .col(
                        ColumnDef::new(Products::Description)
                            .text()
                            .not_null()
                            .default(""),
                    )
                    .col(ColumnDef::new(Products::Handle).string_len(256).not_null())
                    .col(
                        ColumnDef::new(Products::Status)
                            .string_len(20)
                            .not_null()
                            .default("draft"),
                    )
                    .col(
                        ColumnDef::new(Products::ProductType)
                            .string_len(20)
                            .not_null()
                            .default("physical"),
                    )
                    .col(ColumnDef::new(Products::CategoryId).integer())
                    .col(
                        ColumnDef::new(Products::Tags)
                            .json_binary()
                            .not_null()
                            .default("[]"),
                    )
                    .col(
                        ColumnDef::new(Products::Metadata)
                            .json_binary()
                            .not_null()
                            .default("{}"),
                    )
                    .col(ColumnDef::new(Products::SeoTitle).string_len(256))
                    .col(ColumnDef::new(Products::SeoDescription).string_len(512))
                    .col(ColumnDef::new(Products::Weight).decimal())
                    .col(ColumnDef::new(Products::Dimensions).json_binary())
                    .col(
                        ColumnDef::new(Products::Featured)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(Products::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Products::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(Products::DeletedAt).timestamp_with_time_zone())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_products_store")
                            .from(Products::Table, Products::StoreId)
                            .to(Stores::Table, Stores::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_products_category")
                            .from(Products::Table, Products::CategoryId)
                            .to(Categories::Table, Categories::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_products_store_slug")
                    .table(Products::Table)
                    .col(Products::StoreId)
                    .col(Products::Slug)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_products_store_status")
                    .table(Products::Table)
                    .col(Products::StoreId)
                    .col(Products::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_products_featured")
                    .table(Products::Table)
                    .col(Products::StoreId)
                    .col(Products::Featured)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Products::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Products {
    Table,
    Id,
    Pid,
    StoreId,
    Title,
    Slug,
    Description,
    Handle,
    Status,
    ProductType,
    CategoryId,
    Tags,
    Metadata,
    SeoTitle,
    SeoDescription,
    Weight,
    Dimensions,
    Featured,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}

#[derive(Iden)]
enum Stores {
    Table,
    Id,
}

#[derive(Iden)]
enum Categories {
    Table,
    Id,
}
