use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Cria tabela de backup antes de remover
        manager
            .get_connection()
            .execute_unprepared("CREATE TABLE IF NOT EXISTS backup_stores AS SELECT * FROM stores")
            .await?;

        // Remove FK constraints e coluna store_id de store_collaborators
        #[cfg(not(feature = "sqlite"))]
        manager
            .get_connection()
            .execute_unprepared("ALTER TABLE store_collaborators DROP CONSTRAINT fk_collab_store")
            .await
            .ok();

        manager
            .alter_table(
                Table::alter()
                    .table(StoreCollaborators::Table)
                    .drop_column(StoreCollaborators::StoreId)
                    .to_owned(),
            )
            .await?;

        // Remove FK e coluna store_id de order_shippings
        #[cfg(not(feature = "sqlite"))]
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE order_shippings DROP CONSTRAINT IF EXISTS fk_shipping_store",
            )
            .await
            .ok();

        manager
            .alter_table(
                Table::alter()
                    .table(OrderShippings::Table)
                    .drop_column(OrderShippings::StoreId)
                    .to_owned(),
            )
            .await?;

        // Remove FK e coluna store_id de orders
        #[cfg(not(feature = "sqlite"))]
        manager
            .get_connection()
            .execute_unprepared("ALTER TABLE orders DROP CONSTRAINT IF EXISTS fk_orders_store")
            .await
            .ok();

        manager
            .alter_table(
                Table::alter()
                    .table(Orders::Table)
                    .drop_column(Orders::StoreId)
                    .to_owned(),
            )
            .await?;

        // Remove FK e coluna store_id de carts
        #[cfg(not(feature = "sqlite"))]
        manager
            .get_connection()
            .execute_unprepared("ALTER TABLE carts DROP CONSTRAINT IF EXISTS fk_carts_store")
            .await
            .ok();

        manager
            .alter_table(
                Table::alter()
                    .table(Carts::Table)
                    .drop_column(Carts::StoreId)
                    .to_owned(),
            )
            .await?;

        // Remove FK e coluna store_id de customers
        #[cfg(not(feature = "sqlite"))]
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE customers DROP CONSTRAINT IF EXISTS fk_customers_store",
            )
            .await
            .ok();

        manager
            .alter_table(
                Table::alter()
                    .table(Customers::Table)
                    .drop_column(Customers::StoreId)
                    .to_owned(),
            )
            .await?;

        // Remove FK e coluna store_id de collections
        #[cfg(not(feature = "sqlite"))]
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE collections DROP CONSTRAINT IF EXISTS fk_collections_store",
            )
            .await
            .ok();

        manager
            .alter_table(
                Table::alter()
                    .table(Collections::Table)
                    .drop_column(Collections::StoreId)
                    .to_owned(),
            )
            .await?;

        // Remove FK e coluna store_id de products
        #[cfg(not(feature = "sqlite"))]
        manager
            .get_connection()
            .execute_unprepared("ALTER TABLE products DROP CONSTRAINT IF EXISTS fk_products_store")
            .await
            .ok();

        manager
            .alter_table(
                Table::alter()
                    .table(Products::Table)
                    .drop_column(Products::StoreId)
                    .to_owned(),
            )
            .await?;

        // Remove FK e coluna store_id de categories
        #[cfg(not(feature = "sqlite"))]
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE categories DROP CONSTRAINT IF EXISTS fk_categories_store",
            )
            .await
            .ok();

        manager
            .alter_table(
                Table::alter()
                    .table(Categories::Table)
                    .drop_column(Categories::StoreId)
                    .to_owned(),
            )
            .await?;

        // Remove FK da tabela stores para users (owner)
        #[cfg(not(feature = "sqlite"))]
        manager
            .get_connection()
            .execute_unprepared("ALTER TABLE stores DROP CONSTRAINT IF EXISTS fk_stores_owner")
            .await
            .ok();

        // Drop tabela stores
        manager
            .drop_table(Table::drop().table(Stores::Table).if_exists().to_owned())
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Recria tabela stores
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
                    .to_owned(),
            )
            .await?;

        // Restaura coluna store_id em categories (nullable para compatibilidade)
        manager
            .alter_table(
                Table::alter()
                    .table(Categories::Table)
                    .add_column(ColumnDef::new(Categories::StoreId).integer().null())
                    .to_owned(),
            )
            .await?;

        // Restaura coluna store_id em products
        manager
            .alter_table(
                Table::alter()
                    .table(Products::Table)
                    .add_column(ColumnDef::new(Products::StoreId).integer().null())
                    .to_owned(),
            )
            .await?;

        // Restaura coluna store_id em collections
        manager
            .alter_table(
                Table::alter()
                    .table(Collections::Table)
                    .add_column(ColumnDef::new(Collections::StoreId).integer().null())
                    .to_owned(),
            )
            .await?;

        // Restaura coluna store_id em carts
        manager
            .alter_table(
                Table::alter()
                    .table(Carts::Table)
                    .add_column(ColumnDef::new(Carts::StoreId).integer().null())
                    .to_owned(),
            )
            .await?;

        // Restaura coluna store_id em customers
        manager
            .alter_table(
                Table::alter()
                    .table(Customers::Table)
                    .add_column(ColumnDef::new(Customers::StoreId).integer().null())
                    .to_owned(),
            )
            .await?;

        // Restaura coluna store_id em orders
        manager
            .alter_table(
                Table::alter()
                    .table(Orders::Table)
                    .add_column(ColumnDef::new(Orders::StoreId).integer().null())
                    .to_owned(),
            )
            .await?;

        // Restaura coluna store_id em order_shippings
        manager
            .alter_table(
                Table::alter()
                    .table(OrderShippings::Table)
                    .add_column(ColumnDef::new(OrderShippings::StoreId).integer().null())
                    .to_owned(),
            )
            .await?;

        // Restaura coluna store_id em store_collaborators
        manager
            .alter_table(
                Table::alter()
                    .table(StoreCollaborators::Table)
                    .add_column(ColumnDef::new(StoreCollaborators::StoreId).integer().null())
                    .to_owned(),
            )
            .await?;

        Ok(())
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
enum Categories {
    Table,
    StoreId,
}

#[derive(Iden)]
enum Products {
    Table,
    StoreId,
}

#[derive(Iden)]
enum Collections {
    Table,
    StoreId,
}

#[derive(Iden)]
enum Carts {
    Table,
    StoreId,
}

#[derive(Iden)]
enum Customers {
    Table,
    StoreId,
}

#[derive(Iden)]
enum Orders {
    Table,
    StoreId,
}

#[derive(Iden)]
enum OrderShippings {
    Table,
    StoreId,
}

#[derive(Iden)]
enum StoreCollaborators {
    Table,
    StoreId,
}
