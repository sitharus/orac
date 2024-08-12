use sea_orm::DeriveIden;

#[derive(DeriveIden)]
pub enum Guild {
    Table,
    Id,
    DiscordId,
    Name,
    LogoUrl,
}

#[derive(DeriveIden)]
pub enum Channel {
    Table,
    Id,
    GuildId,
    DiscordId,
    Name,
    ResetDay,
    ResetMessage,
    ResetSchedule,
    AllowReset,
}

#[derive(DeriveIden)]
pub enum User {
    Table,
    Id,
    Name,
    Password,
    Email,
    DiscordId,
}

#[derive(DeriveIden)]
pub enum Session {
    Table,
    Id,
    Data,
    ExpiryDate,
}

#[derive(DeriveIden)]
pub enum GuildAccess {
    Table,
    GuildId,
    UserId,
    IsOwner,
    IsAdministrator,
    Roles,
}
