use sea_orm_migration::prelude::*;

use super::entities::Channel;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Channel::Table)
                    .drop_column(Channel::ResetDay)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Channel::Table)
                    .add_column(ColumnDef::new(Channel::ResetSchedule).text())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Channel::Table)
                    .add_column(
                        ColumnDef::new(Channel::AllowReset)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Migration("Cannot downgrade from this!".into()))
    }
}
