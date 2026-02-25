use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Collections
        manager
            .create_table(
                Table::create()
                    .table(Collections::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Collections::Id).integer().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new(Collections::Pid).uuid().not_null().unique_key())
                    .col(ColumnDef::new(Collections::StoreId).integer().not_null())
                    .col(ColumnDef::new(Collections::Title).string_len(256).not_null())
                    .col(ColumnDef::new(Collections::Slug).string_len(256).not_null())
                    .col(ColumnDef::new(Collections::Description).text().not_null().default(""))
                    .col(ColumnDef::new(Collections::ImageUrl).string_len(1024))
                    .col(ColumnDef::new(Collections::Published).boolean().not_null().default(false))
                    .col(ColumnDef::new(Collections::SortOrder).integer().not_null().default(0))
                    .col(ColumnDef::new(Collections::Metadata).json_binary().not_null().default("{}"))
                    .col(ColumnDef::new(Collections::CreatedAt).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(Collections::UpdatedAt).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(Collections::DeletedAt).timestamp_with_time_zone())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_collections_store")
                            .from(Collections::Table, Collections::StoreId)
                            .to(Stores::Table, Stores::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_collections_store_slug")
                    .table(Collections::Table)
                    .col(Collections::StoreId)
                    .col(Collections::Slug)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Collection-products join table
        manager
            .create_table(
                Table::create()
                    .table(CollectionProducts::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(CollectionProducts::Id).integer().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new(CollectionProducts::CollectionId).integer().not_null())
                    .col(ColumnDef::new(CollectionProducts::ProductId).integer().not_null())
                    .col(ColumnDef::new(CollectionProducts::SortOrder).integer().not_null().default(0))
                    .col(ColumnDef::new(CollectionProducts::CreatedAt).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_colprod_collection")
                            .from(CollectionProducts::Table, CollectionProducts::CollectionId)
                            .to(Collections::Table, Collections::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_colprod_product")
                            .from(CollectionProducts::Table, CollectionProducts::ProductId)
                            .to(Products::Table, Products::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_colprod_unique")
                    .table(CollectionProducts::Table)
                    .col(CollectionProducts::CollectionId)
                    .col(CollectionProducts::ProductId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(CollectionProducts::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(Collections::Table).to_owned()).await?;
        Ok(())
    }
}

#[derive(Iden)]
enum Collections {
    Table,
    Id,
    Pid,
    StoreId,
    Title,
    Slug,
    Description,
    ImageUrl,
    Published,
    SortOrder,
    Metadata,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}

#[derive(Iden)]
enum CollectionProducts {
    Table,
    Id,
    CollectionId,
    ProductId,
    SortOrder,
    CreatedAt,
}

#[derive(Iden)]
enum Stores {
    Table,
    Id,
}

#[derive(Iden)]
enum Products {
    Table,
    Id,
}
