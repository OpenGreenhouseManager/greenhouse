use super::{Error, Result, schema::device_key};
use crate::Pool;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

/// A scripting key handed to a device on activation. The device sends it as
/// its `auth-token` when it calls `scripting_api`.
#[derive(Debug, Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::database::schema::device_key)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct DeviceKey {
    pub(crate) key: String,
    pub(crate) device_id: Uuid,
}

impl DeviceKey {
    /// Creates and stores a new key for `device_id`.
    pub(crate) async fn issue(device_id: Uuid, pool: &Pool) -> Result<Self> {
        let mut conn = pool.get().await.map_err(|e| {
            sentry::capture_error(&e);
            Error::DatabaseConnection
        })?;
        let entry = Self {
            key: Uuid::new_v4().to_string(),
            device_id,
        };
        diesel::insert_into(device_key::table)
            .values(&entry)
            .execute(&mut conn)
            .await
            .map_err(|e| {
                sentry::capture_error(&e);
                Error::Creation
            })?;
        Ok(entry)
    }

    /// Whether `key` belongs to a device.
    pub(crate) async fn exists(key: &str, pool: &Pool) -> Result<bool> {
        let mut conn = pool.get().await.map_err(|e| {
            sentry::capture_error(&e);
            Error::DatabaseConnection
        })?;
        diesel::select(diesel::dsl::exists(
            device_key::table.filter(device_key::key.eq(key)),
        ))
        .get_result(&mut conn)
        .await
        .map_err(|e| {
            sentry::capture_error(&e);
            Error::Find
        })
    }

    /// Removes every other key of this key's device, so only this one stays
    /// valid.
    pub(crate) async fn revoke_others(&self, pool: &Pool) -> Result<()> {
        let mut conn = pool.get().await.map_err(|e| {
            sentry::capture_error(&e);
            Error::DatabaseConnection
        })?;
        diesel::delete(
            device_key::table
                .filter(device_key::device_id.eq(self.device_id))
                .filter(device_key::key.ne(&self.key)),
        )
        .execute(&mut conn)
        .await
        .map_err(|e| {
            sentry::capture_error(&e);
            Error::Delete
        })?;
        Ok(())
    }
}
