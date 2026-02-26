use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Stores::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Stores::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Stores::Pid).uuid().not_null().unique_key())
                    .col(
                        ColumnDef::new(Stores::Slug)
                            .string_len(128)
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Stores::Name).string_len(256).not_null())
                    .col(ColumnDef::new(Stores::Domain).string_len(256))
                    .col(
                        ColumnDef::new(Stores::DefaultCurrency)
                            .string_len(3)
                            .not_null()
                            .default("BRL"),
                    )
                    .col(
                        ColumnDef::new(Stores::Config)
                            .json_binary()
                            .not_null()
                            .default("{}"),
                    )
                    .col(
                        ColumnDef::new(Stores::Status)
                            .string_len(20)
                            .not_null()
                            .default("draft"),
                    )
                    .col(
                        ColumnDef::new(Stores::Metadata)
                            .json_binary()
                            .not_null()
                            .default("{}"),
                    )
                    .col(ColumnDef::new(Stores::OwnerId).integer().not_null())
                    .col(
                        ColumnDef::new(Stores::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Stores::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(Stores::DeletedAt).timestamp_with_time_zone())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_stores_owner")
                            .from(Stores::Table, Stores::OwnerId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_stores_slug")
                    .table(Stores::Table)
                    .col(Stores::Slug)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Stores::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Stores {
    Table,
    Id,
    Pid,
    Slug,
    Name,
    Domain,
    DefaultCurrency,
    Config,
    Status,
    Metadata,
    OwnerId,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}
