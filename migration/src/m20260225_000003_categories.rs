use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Categories::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Categories::Id).integer().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new(Categories::Pid).uuid().not_null().unique_key())
                    .col(ColumnDef::new(Categories::StoreId).integer().not_null())
                    .col(ColumnDef::new(Categories::Name).string_len(256).not_null())
                    .col(ColumnDef::new(Categories::Slug).string_len(256).not_null())
                    .col(ColumnDef::new(Categories::Description).text())
                    .col(ColumnDef::new(Categories::ParentId).integer())
                    .col(ColumnDef::new(Categories::ImageUrl).string_len(512))
                    .col(ColumnDef::new(Categories::SortOrder).integer().not_null().default(0))
                    .col(ColumnDef::new(Categories::CreatedAt).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(Categories::UpdatedAt).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(Categories::DeletedAt).timestamp_with_time_zone())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_categories_store")
                            .from(Categories::Table, Categories::StoreId)
                            .to(Stores::Table, Stores::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_categories_parent")
                            .from(Categories::Table, Categories::ParentId)
                            .to(Categories::Table, Categories::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_categories_store_slug")
                    .table(Categories::Table)
                    .col(Categories::StoreId)
                    .col(Categories::Slug)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(Categories::Table).to_owned()).await
    }
}

#[derive(Iden)]
enum Categories {
    Table,
    Id,
    Pid,
    StoreId,
    Name,
    Slug,
    Description,
    ParentId,
    ImageUrl,
    SortOrder,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}

#[derive(Iden)]
enum Stores {
    Table,
    Id,
}
