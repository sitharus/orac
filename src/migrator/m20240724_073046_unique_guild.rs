use sea_orm_migration::prelude::*;

use super::entities::Guild;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let constraint = Index::create()
            .name("uq-guild-discordid")
            .table(Guild::Table)
            .col(Guild::DiscordId)
            .unique()
            .to_owned();
        manager.create_index(constraint).await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Guild::Table)
                    .add_column_if_not_exists(ColumnDef::new(Guild::LogoUrl).text())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("uq-guild-discordid")
                    .table(Guild::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Guild::Table)
                    .drop_column(Guild::LogoUrl)
                    .to_owned(),
            )
            .await
    }
}
