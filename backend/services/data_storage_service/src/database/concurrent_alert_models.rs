//! Concurrent alerts: lasting conditions, in the `concurrent_alert` table.
//!
//! An alert is open while `expires_at` is in the future. Firing moves
//! `expires_at` to now + keep-alive, stopping sets it to now. Nothing else
//! records the end, so expiry needs no background job.

use super::alert_models::AggregateRow;
use super::severity_models::Severity;
use super::{Error, Result, schema::concurrent_alert};
use chrono::{DateTime, TimeDelta, Utc};
use diesel::sql_types::Text;
use diesel::*;
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use greenhouse_core::data_storage_service_dto::alert_dto::{
    alert::{AlertDto, AlertKind, EndReason},
    post_create_alert::CreateAlertDto,
    query::{AlertQuery, IntervalQuery},
};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::database::schema::concurrent_alert)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct ConcurrentAlert {
    pub(crate) id: Uuid,
    pub(crate) severity: Severity,
    pub(crate) identifier: String,
    pub(crate) value: String,
    pub(crate) note: Option<String>,
    pub(crate) datasource_id: Uuid,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) last_seen_at: DateTime<Utc>,
    pub(crate) expires_at: DateTime<Utc>,
    pub(crate) keep_alive_secs: i32,
    pub(crate) occurrences: i64,
}

impl ConcurrentAlert {
    /// Opens an alert, or, if one with the same `datasource_id` and
    /// `identifier` is open, updates it and extends its `expires_at`.
    ///
    /// Runs in a transaction holding an advisory lock on the key, so
    /// simultaneous fires cannot open two alerts for the same key.
    pub(crate) async fn fire(
        alert: CreateAlertDto,
        datasource_id: Uuid,
        keep_alive_secs: u32,
        conn: &mut AsyncPgConnection,
    ) -> Result<Self> {
        let now = Utc::now();
        let expires_at = now + TimeDelta::seconds(keep_alive_secs.into());
        let keep_alive_secs = i32::try_from(keep_alive_secs).map_err(|_| Error::InvalidInput)?;
        let severity: Severity = alert.severity.into();
        let value = alert.value.unwrap_or_default();
        let identifier = alert.identifier;
        let note = alert.note;

        let result = conn
            .transaction(async move |conn| {
                diesel::sql_query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
                    .bind::<Text, _>(format!("concurrent_alert:{datasource_id}:{identifier}"))
                    .execute(conn)
                    .await?;

                let updated = diesel::update(
                    concurrent_alert::table
                        .filter(concurrent_alert::datasource_id.eq(datasource_id))
                        .filter(concurrent_alert::identifier.eq(&identifier))
                        .filter(concurrent_alert::expires_at.gt(now)),
                )
                .set((
                    concurrent_alert::severity.eq(severity.clone()),
                    concurrent_alert::value.eq(&value),
                    concurrent_alert::note.eq(&note),
                    concurrent_alert::last_seen_at.eq(now),
                    concurrent_alert::expires_at.eq(expires_at),
                    concurrent_alert::keep_alive_secs.eq(keep_alive_secs),
                    concurrent_alert::occurrences.eq(concurrent_alert::occurrences + 1),
                ))
                .returning(Self::as_returning())
                .get_result(conn)
                .await
                .optional()?;
                if let Some(updated) = updated {
                    return Ok(updated);
                }

                diesel::insert_into(concurrent_alert::table)
                    .values(Self {
                        id: Uuid::new_v4(),
                        severity,
                        identifier,
                        value,
                        note,
                        datasource_id,
                        created_at: now,
                        last_seen_at: now,
                        expires_at,
                        keep_alive_secs,
                        occurrences: 1,
                    })
                    .returning(Self::as_returning())
                    .get_result(conn)
                    .await
            })
            .await;
        result.map_err(|e: diesel::result::Error| {
            sentry::capture_error(&e);
            Error::Creation
        })
    }

    /// Stops the open alert for `datasource_id` and `identifier` by setting its
    /// `expires_at` to now. Returns `None` if none was open; that is not an error.
    pub(crate) async fn stop(
        datasource_id: Uuid,
        identifier: String,
        note: Option<String>,
        conn: &mut AsyncPgConnection,
    ) -> Result<Option<Self>> {
        let now = Utc::now();
        let target = concurrent_alert::table
            .filter(concurrent_alert::datasource_id.eq(datasource_id))
            .filter(concurrent_alert::identifier.eq(identifier))
            .filter(concurrent_alert::expires_at.gt(now));
        let result = match note {
            Some(note) => {
                diesel::update(target)
                    .set((
                        concurrent_alert::expires_at.eq(now),
                        concurrent_alert::note.eq(note),
                    ))
                    .returning(Self::as_returning())
                    .get_result(conn)
                    .await
            }
            None => {
                diesel::update(target)
                    .set(concurrent_alert::expires_at.eq(now))
                    .returning(Self::as_returning())
                    .get_result(conn)
                    .await
            }
        };
        result.optional().map_err(|e| {
            sentry::capture_error(&e);
            Error::Update
        })
    }

    pub(crate) async fn query(
        alert_query: &AlertQuery,
        conn: &mut AsyncPgConnection,
    ) -> Result<Vec<Self>> {
        let now = Utc::now();
        let mut query = concurrent_alert::table.into_boxed();
        if let Some(start) = alert_query.created_at {
            query = query.filter(concurrent_alert::created_at.ge(start));
        }
        if let Some(datasource_id) = alert_query.datasource_id {
            query = query.filter(concurrent_alert::datasource_id.eq(datasource_id));
        }
        if let Some(severity) = alert_query.severity {
            let s: Severity = severity.into();
            query = query.filter(concurrent_alert::severity.eq(s));
        }
        if let Some(identifier) = &alert_query.identifier {
            query = query.filter(concurrent_alert::identifier.eq(identifier));
        }
        match alert_query.open {
            Some(true) => query = query.filter(concurrent_alert::expires_at.gt(now)),
            Some(false) => query = query.filter(concurrent_alert::expires_at.le(now)),
            None => {}
        }
        query.load(conn).await.map_err(|e| {
            sentry::capture_error(&e);
            Error::Find
        })
    }

    /// Concurrent alerts that were open at some point in the interval, grouped,
    /// with how many are open now.
    pub(crate) async fn aggregate(
        interval: &IntervalQuery,
        conn: &mut AsyncPgConnection,
    ) -> Result<Vec<AggregateRow>> {
        let now = Utc::now();
        let grouped = || {
            let mut query = concurrent_alert::table
                .group_by((
                    concurrent_alert::datasource_id,
                    concurrent_alert::severity,
                    concurrent_alert::identifier,
                ))
                .select((
                    concurrent_alert::datasource_id,
                    concurrent_alert::severity,
                    concurrent_alert::identifier,
                    diesel::dsl::count(concurrent_alert::id),
                    diesel::dsl::min(concurrent_alert::created_at),
                    diesel::dsl::max(concurrent_alert::last_seen_at),
                ))
                .into_boxed();
            if let Some(start) = interval.start {
                query = query.filter(concurrent_alert::expires_at.ge(start));
            }
            if let Some(end) = interval.end {
                query = query.filter(concurrent_alert::created_at.le(end));
            }
            query
        };
        type Row = (
            Uuid,
            Severity,
            String,
            i64,
            Option<DateTime<Utc>>,
            Option<DateTime<Utc>>,
        );
        let map_err = |e: diesel::result::Error| {
            sentry::capture_error(&e);
            Error::Find
        };
        let all: Vec<Row> = grouped().load(conn).await.map_err(map_err)?;
        let open: HashMap<(Uuid, Severity, String), i64> = grouped()
            .filter(concurrent_alert::expires_at.gt(now))
            .load::<Row>(conn)
            .await
            .map_err(map_err)?
            .into_iter()
            .map(|(ds, sev, ident, count, _, _)| ((ds, sev, ident), count))
            .collect();
        Ok(all
            .into_iter()
            .map(|(ds, sev, ident, count, first, last)| {
                let open = open
                    .get(&(ds, sev.clone(), ident.clone()))
                    .copied()
                    .unwrap_or(0);
                (ds, sev, ident, count, open, first, last)
            })
            .collect())
    }

    /// Converts to the API representation, as of `now`.
    pub(crate) fn into_dto(self, now: DateTime<Utc>) -> AlertDto {
        let open = self.expires_at > now;
        // Firing sets `expires_at = last_seen_at + keep-alive`; stopping sets
        // it earlier. So an ended alert expired iff that equation still holds.
        let end_reason = match open {
            true => None,
            false
                if self.expires_at
                    == self.last_seen_at + TimeDelta::seconds(self.keep_alive_secs.into()) =>
            {
                Some(EndReason::Expired)
            }
            false => Some(EndReason::Stopped),
        };
        AlertDto {
            id: self.id.to_string(),
            severity: self.severity.into(),
            identifier: self.identifier,
            value: self.value,
            note: self.note,
            created_at: self.created_at.to_rfc3339(),
            datasource_id: self.datasource_id.to_string(),
            kind: AlertKind::Concurrent,
            last_seen_at: self.last_seen_at.to_rfc3339(),
            expires_at: self.expires_at.to_rfc3339(),
            open,
            keep_alive_secs: u32::try_from(self.keep_alive_secs).ok(),
            end_reason,
            occurrences: self.occurrences,
        }
    }
}
