use std::sync::Arc;

use axum::{async_trait, response::Redirect};
use sea_orm::{ActiveModelTrait, DatabaseConnection};
use tower_sessions::{
    session::{Id, Record},
    session_store, Session, SessionStore,
};

use crate::entities::session;
use sea_orm::*;

pub const USER_ID_KEY: &str = "USER_ID";

pub async fn user_id(session: Session) -> Result<i32, Redirect> {
    let user_id: Option<i32> = session
        .get(USER_ID_KEY)
        .await
        .map_err(|_| Redirect::to("/"))?;
    user_id.ok_or(Redirect::to("/"))
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
}

#[async_trait]
impl SessionStore for OrmStore {
    async fn save(&self, record: &Record) -> session_store::Result<()> {
        let row: session::ActiveModel = session::Model {
            id: record.id.to_string(),
            expiry_date: record.expiry_date,
            data: serde_json::to_value(&record)
                .map_err(|e| session_store::Error::Encode(e.to_string()))?
                .into(),
        }
        .into();

        row.insert(self.pool.as_ref())
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;
        Ok(())
    }

    async fn load(&self, session_id: &Id) -> session_store::Result<Option<Record>> {
        match session::Entity::find_by_id(session_id.to_string())
            .one(self.pool.as_ref())
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?
        {
            Some(r) => {
                let dec: Record = serde_json::from_value(r.data)
                    .map_err(|e| session_store::Error::Decode(e.to_string()))?;
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
}
