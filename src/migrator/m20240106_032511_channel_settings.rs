use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Channel::Table)
                    .add_column_if_not_exists(ColumnDef::new(Channel::ResetDay).small_integer())
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Channel::Table)
                    .add_column_if_not_exists(ColumnDef::new(Channel::ResetMessage).text())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .alter_table(
                Table::alter()
                    .table(Channel::Table)
                    .drop_column(Channel::ResetDay)
                    .drop_column(Channel::ResetMessage)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Channel {
    Table,
    ResetDay,
    ResetMessage,
}
