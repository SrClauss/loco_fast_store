use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    #[allow(clippy::too_many_lines)]
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // ── store_collaborators ──────────────────────────────────────
        // Vincula usuários do sistema como colaboradores de uma loja,
        // com papel (role) que define o que cada um pode fazer.
        manager
            .create_table(
                Table::create()
                    .table(StoreCollaborators::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(StoreCollaborators::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(StoreCollaborators::Pid)
                            .uuid()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(StoreCollaborators::StoreId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(StoreCollaborators::UserId)
                            .integer()
                            .not_null(),
                    )
                    // Roles: 'owner' | 'admin' | 'shipping' | 'viewer'
                    .col(
                        ColumnDef::new(StoreCollaborators::Role)
                            .string_len(20)
                            .not_null()
                            .default("shipping"),
                    )
                    .col(
                        ColumnDef::new(StoreCollaborators::Active)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(StoreCollaborators::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(StoreCollaborators::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_collab_store")
                            .from(StoreCollaborators::Table, StoreCollaborators::StoreId)
                            .to(Stores::Table, Stores::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_collab_user")
                            .from(StoreCollaborators::Table, StoreCollaborators::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Par (store_id, user_id) deve ser único
        manager
            .create_index(
                Index::create()
                    .name("idx_collab_store_user")
                    .table(StoreCollaborators::Table)
                    .col(StoreCollaborators::StoreId)
                    .col(StoreCollaborators::UserId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // ── order_shippings ─────────────────────────────────────────
        // Representa o envio de um pedido. Suporta registro manual e
        // integração futura com provedores como MelhorEnvio, CorreiosAPI, etc.
        manager
            .create_table(
                Table::create()
                    .table(OrderShippings::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(OrderShippings::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(OrderShippings::Pid)
                            .uuid()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(OrderShippings::OrderId).integer().not_null())
                    .col(ColumnDef::new(OrderShippings::StoreId).integer().not_null())
                    // Transportadora: 'correios', 'jadlog', 'total_express', 'melhor_envio', 'manual', ...
                    .col(
                        ColumnDef::new(OrderShippings::Carrier)
                            .string_len(64)
                            .not_null()
                            .default("manual"),
                    )
                    // Serviço: 'pac', 'sedex', 'expresso', etc.
                    .col(ColumnDef::new(OrderShippings::Service).string_len(64))
                    .col(ColumnDef::new(OrderShippings::TrackingCode).string_len(128))
                    .col(ColumnDef::new(OrderShippings::TrackingUrl).string_len(512))
                    // Status: 'pending' | 'posted' | 'in_transit' | 'out_for_delivery' | 'delivered' | 'failed' | 'returned'
                    .col(
                        ColumnDef::new(OrderShippings::Status)
                            .string_len(30)
                            .not_null()
                            .default("pending"),
                    )
                    .col(ColumnDef::new(OrderShippings::ShippedAt).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(OrderShippings::EstimatedDeliveryAt)
                            .timestamp_with_time_zone(),
                    )
                    .col(ColumnDef::new(OrderShippings::DeliveredAt).timestamp_with_time_zone())
                    // Provider externo: 'melhor_envio' | 'correios_api' | null = manual
                    .col(ColumnDef::new(OrderShippings::Provider).string_len(32))
                    // Dados brutos retornados pelo provider (ex.: shipment_id no MelhorEnvio)
                    .col(
                        ColumnDef::new(OrderShippings::ProviderData)
                            .json_binary()
                            .not_null()
                            .default("{}"),
                    )
                    .col(ColumnDef::new(OrderShippings::Notes).text())
                    .col(
                        ColumnDef::new(OrderShippings::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(OrderShippings::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_shipping_order")
                            .from(OrderShippings::Table, OrderShippings::OrderId)
                            .to(Orders::Table, Orders::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_shipping_store")
                            .from(OrderShippings::Table, OrderShippings::StoreId)
                            .to(Stores::Table, Stores::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_shipping_order")
                    .table(OrderShippings::Table)
                    .col(OrderShippings::OrderId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_shipping_store_status")
                    .table(OrderShippings::Table)
                    .col(OrderShippings::StoreId)
                    .col(OrderShippings::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_shipping_tracking")
                    .table(OrderShippings::Table)
                    .col(OrderShippings::TrackingCode)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(OrderShippings::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(StoreCollaborators::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(Iden)]
enum StoreCollaborators {
    Table,
    Id,
    Pid,
    StoreId,
    UserId,
    Role,
    Active,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum OrderShippings {
    Table,
    Id,
    Pid,
    OrderId,
    StoreId,
    Carrier,
    Service,
    TrackingCode,
    TrackingUrl,
    Status,
    ShippedAt,
    EstimatedDeliveryAt,
    DeliveredAt,
    Provider,
    ProviderData,
    Notes,
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

#[derive(Iden)]
enum Orders {
    Table,
    Id,
}
