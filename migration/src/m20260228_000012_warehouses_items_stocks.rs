use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // warehouses table
        manager
            .create_table(
                Table::create()
                    .table(Warehouses::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Warehouses::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Warehouses::Pid)
                            .uuid()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Warehouses::Name)
                            .string_len(256)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Warehouses::Latitude)
                            .double()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Warehouses::Longitude)
                            .double()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // variants already exist; create items
        manager
            .create_table(
                Table::create()
                    .table(Items::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Items::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Items::Pid)
                            .uuid()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Items::VariantId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Items::Batch)
                            .string_len(128),
                    )
                    .col(
                        ColumnDef::new(Items::Expiration)
                            .date(),
                    )
                    .to_owned(),
            )
            .await?;

        // stocks linking warehouses and items
        manager
            .create_table(
                Table::create()
                    .table(Stocks::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Stocks::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Stocks::WarehouseId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Stocks::ItemId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Stocks::Quantity)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_stock_warehouse")
                            .from(Stocks::Table, Stocks::WarehouseId)
                            .to(Warehouses::Table, Warehouses::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_stock_item")
                            .from(Stocks::Table, Stocks::ItemId)
                            .to(Items::Table, Items::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Stocks::Table).if_exists().to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Items::Table).if_exists().to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Warehouses::Table).if_exists().to_owned())
            .await?;
        Ok(())
    }
}

/// Generated entity definitions used by the migration only
#[derive(Iden)]
enum Warehouses {
    Table,
    Id,
    Pid,
    Name,
    Latitude,
    Longitude,
}

#[derive(Iden)]
enum Items {
    Table,
    Id,
    Pid,
    VariantId,
    Batch,
    Expiration,
}

#[derive(Iden)]
enum Stocks {
    Table,
    Id,
    WarehouseId,
    ItemId,
    Quantity,
}
