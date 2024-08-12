use sea_orm_migration::prelude::*;

use super::entities::GuildAccess;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(GuildAccess::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(GuildAccess::GuildId).integer().not_null())
                    .col(ColumnDef::new(GuildAccess::UserId).integer().not_null())
                    .col(ColumnDef::new(GuildAccess::IsOwner).boolean().not_null())
                    .col(
                        ColumnDef::new(GuildAccess::IsAdministrator)
                            .boolean()
                            .not_null(),
                    )
                    .col(ColumnDef::new(GuildAccess::Roles).json().not_null())
                    .primary_key(
                        Index::create()
                            .col(GuildAccess::GuildId)
                            .col(GuildAccess::UserId),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(GuildAccess::Table).to_owned())
            .await
    }
}
