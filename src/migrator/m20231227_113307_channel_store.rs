use sea_orm_migration::prelude::*;

use super::entities::{Channel, Guild};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Guild::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Guild::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Guild::Name).string())
                    .col(ColumnDef::new(Guild::DiscordId).string().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Channel::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Channel::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Channel::Name).string())
                    .col(ColumnDef::new(Channel::DiscordId).string().not_null())
                    .col(ColumnDef::new(Channel::GuildId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("FK_channel_guild")
                            .from(Channel::Table, Channel::GuildId)
                            .to(Guild::Table, Guild::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Guild::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Channel::Table).to_owned())
            .await
    }
}
