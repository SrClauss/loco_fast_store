use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Customers
        manager
            .create_table(
                Table::create()
                    .table(Customers::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Customers::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Customers::Pid)
                            .uuid()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Customers::StoreId).integer().not_null())
                    .col(ColumnDef::new(Customers::Email).string_len(256).not_null())
                    .col(
                        ColumnDef::new(Customers::FirstName)
                            .string_len(128)
                            .not_null()
                            .default(""),
                    )
                    .col(
                        ColumnDef::new(Customers::LastName)
                            .string_len(128)
                            .not_null()
                            .default(""),
                    )
                    .col(ColumnDef::new(Customers::Phone).string_len(32))
                    .col(
                        ColumnDef::new(Customers::HasAccount)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(Customers::UserId).integer())
                    .col(
                        ColumnDef::new(Customers::Metadata)
                            .json_binary()
                            .not_null()
                            .default("{}"),
                    )
                    .col(
                        ColumnDef::new(Customers::MarketingConsent)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(Customers::AnalyticsSessionId).string_len(128))
                    .col(ColumnDef::new(Customers::LastSeenAt).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(Customers::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Customers::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(Customers::DeletedAt).timestamp_with_time_zone())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_customers_store")
                            .from(Customers::Table, Customers::StoreId)
                            .to(Stores::Table, Stores::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_customers_user")
                            .from(Customers::Table, Customers::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_customers_store_email")
                    .table(Customers::Table)
                    .col(Customers::StoreId)
                    .col(Customers::Email)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Addresses
        manager
            .create_table(
                Table::create()
                    .table(Addresses::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Addresses::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Addresses::Pid)
                            .uuid()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Addresses::CustomerId).integer().not_null())
                    .col(
                        ColumnDef::new(Addresses::FirstName)
                            .string_len(128)
                            .not_null()
                            .default(""),
                    )
                    .col(
                        ColumnDef::new(Addresses::LastName)
                            .string_len(128)
                            .not_null()
                            .default(""),
                    )
                    .col(ColumnDef::new(Addresses::Company).string_len(256))
                    .col(
                        ColumnDef::new(Addresses::AddressLine1)
                            .string_len(512)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Addresses::AddressLine2).string_len(512))
                    .col(ColumnDef::new(Addresses::City).string_len(128).not_null())
                    .col(ColumnDef::new(Addresses::State).string_len(64).not_null())
                    .col(
                        ColumnDef::new(Addresses::PostalCode)
                            .string_len(20)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Addresses::Country)
                            .string_len(2)
                            .not_null()
                            .default("BR"),
                    )
                    .col(ColumnDef::new(Addresses::Phone).string_len(32))
                    .col(
                        ColumnDef::new(Addresses::IsDefaultShipping)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(Addresses::IsDefaultBilling)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(Addresses::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Addresses::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_addresses_customer")
                            .from(Addresses::Table, Addresses::CustomerId)
                            .to(Customers::Table, Customers::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Addresses::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Customers::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(Iden)]
enum Customers {
    Table,
    Id,
    Pid,
    StoreId,
    Email,
    FirstName,
    LastName,
    Phone,
    HasAccount,
    UserId,
    Metadata,
    MarketingConsent,
    AnalyticsSessionId,
    LastSeenAt,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}

#[derive(Iden)]
enum Addresses {
    Table,
    Id,
    Pid,
    CustomerId,
    FirstName,
    LastName,
    Company,
    AddressLine1,
    AddressLine2,
    City,
    State,
    PostalCode,
    Country,
    Phone,
    IsDefaultShipping,
    IsDefaultBilling,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Stores {
    Table,
    Id,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}
