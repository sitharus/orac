use std::{str::FromStr, sync::Arc};

use axum::{async_trait, response::Redirect};
use sea_orm::{sea_query::OnConflict, DatabaseConnection};
use tower_sessions::{
    session::{Id, Record},
    session_store, Session, SessionStore,
};

use super::errors::Error;
use crate::entities::session;
use sea_orm::*;

pub const USER_ID_KEY: &str = "USER_ID";
pub const DISCORD_USER_ID_KEY: &str = "DISCORD_USER_ID";

pub async fn user_id_or_redirect(session: Session) -> Result<i32, Redirect> {
    let user_id: Option<i32> = session
        .get(USER_ID_KEY)
        .await
        .map_err(|_| Redirect::to("/"))?;
    user_id.ok_or(Redirect::to("/"))
}

pub async fn discord_user_id(session: &Session) -> Result<u64, Error> {
    session
        .get::<String>(DISCORD_USER_ID_KEY)
        .await?
        .ok_or(Error::LoggedOut)
        .and_then(|v| {
            v.parse::<u64>()
                .map_err(|_| Error::Anyhow(anyhow::anyhow!("Cannot convert ID")))
        })
        .map_err(|e| e.into())
}

pub async fn user_id(session: &Session) -> Result<i32, Error> {
    session
        .get::<i32>(USER_ID_KEY)
        .await?
        .ok_or(Error::LoggedOut)
}

/// A SQLite session store.
#[derive(Clone, Debug)]
pub struct OrmStore {
    pool: Arc<DatabaseConnection>,
}

impl OrmStore {
    pub fn new(pool: Arc<DatabaseConnection>) -> Self {
        Self { pool }
    }

    async fn id_exists(&self, id: &Id) -> session_store::Result<bool> {
        let existing = session::Entity::find_by_id(id.to_string())
            .one(self.pool.as_ref())
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;
        return Ok(existing.is_some());
    }

    async fn save_or_insert(&self, record: &Record) -> session_store::Result<()> {
        let row: session::ActiveModel = session::Model {
            id: record.id.to_string(),
            expiry_date: record.expiry_date,
            data: serde_json::to_value(&record.data)
                .map_err(|e| {
                    session_store::Error::Encode(format!("Serde error: {:?}", e).to_string())
                })?
                .into(),
        }
        .into();

        session::Entity::insert(row)
            .on_conflict(
                OnConflict::column(session::Column::Id)
                    .update_columns([session::Column::Data, session::Column::ExpiryDate])
                    .to_owned(),
            )
            .exec(self.pool.as_ref())
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;
        Ok(())
    }
}

#[async_trait]
impl SessionStore for OrmStore {
    async fn save(&self, record: &Record) -> session_store::Result<()> {
        self.save_or_insert(record).await
    }

    async fn load(&self, session_id: &Id) -> session_store::Result<Option<Record>> {
        match session::Entity::find_by_id(session_id.to_string())
            .one(self.pool.as_ref())
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?
        {
            Some(r) => {
                let dec = Record {
                    id: Id::from_str(r.id.as_ref())
                        .map_err(|_| session_store::Error::Encode("Could not decode ID".into()))?,
                    data: serde_json::from_value(r.data)
                        .map_err(|e| session_store::Error::Decode(e.to_string()))?,
                    expiry_date: r.expiry_date,
                };
                Ok(Some(dec))
            }

            None => Ok(None),
        }
    }

    async fn delete(&self, session_id: &Id) -> session_store::Result<()> {
        session::Entity::delete_by_id(session_id.to_string())
            .exec(self.pool.as_ref())
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;
        Ok(())
    }

    async fn create(&self, record: &mut Record) -> session_store::Result<()> {
        while self.id_exists(&record.id).await? {
            record.id = Id::default();
        }

        self.save_or_insert(record).await
    }
}
