use super::aggrigated_alert::AggrigatedAlert;
use super::severity_models::Severity;
use super::{Error, Result, schema::alert};
use crate::Pool;
use chrono::{DateTime, TimeDelta, Utc};
use diesel::upsert::{DecoratableTarget, excluded};
use diesel::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use greenhouse_core::data_storage_service_dto::alert_dto::{
    alert::{AlertDto, AlertKind, EndReason},
    post_create_alert::{CreateAlertDto, DEFAULT_KEEP_ALIVE_SECS, MAX_KEEP_ALIVE_SECS},
    post_stop_alert::StopAlertDto,
    query::{AlertQuery, IntervalQuery},
};
use uuid::Uuid;

const KIND_ONE_TIME: &str = "one_time";
const KIND_CONCURRENT: &str = "concurrent";
const END_STOPPED: &str = "stopped";
const END_EXPIRED: &str = "expired";

fn kind_to_db(kind: AlertKind) -> &'static str {
    match kind {
        AlertKind::OneTime => KIND_ONE_TIME,
        AlertKind::Concurrent => KIND_CONCURRENT,
    }
}

#[derive(Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::database::schema::alert)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct Alert {
    pub(crate) id: Uuid,
    pub(crate) severity: Severity,
    pub(crate) identifier: String,
    pub(crate) value: String,
    pub(crate) note: Option<String>,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) datasource_id: Uuid,
    pub(crate) kind: String,
    pub(crate) last_seen_at: DateTime<Utc>,
    pub(crate) expires_at: DateTime<Utc>,
    pub(crate) ended_at: Option<DateTime<Utc>>,
    pub(crate) end_reason: Option<String>,
    pub(crate) occurrences: i64,
}

fn parse_datasource_id(datasource_id: &str) -> Result<Uuid> {
    datasource_id.parse().map_err(|e| {
        tracing::debug!("invalid datasource_id {datasource_id:?}: {e}");
        Error::InvalidInput
    })
}

/// Validates the keep-alive for `kind`. `None` for one-time alerts.
fn keep_alive(kind: AlertKind, keep_alive_secs: Option<u32>) -> Result<Option<TimeDelta>> {
    match (kind, keep_alive_secs) {
        (AlertKind::OneTime, None) => Ok(None),
        (AlertKind::OneTime, Some(_)) => {
            tracing::debug!("keep_alive_secs is only allowed for concurrent alerts");
            Err(Error::InvalidInput)
        }
        (AlertKind::Concurrent, secs) => {
            let secs = secs.unwrap_or(DEFAULT_KEEP_ALIVE_SECS);
            if !(1..=MAX_KEEP_ALIVE_SECS).contains(&secs) {
                tracing::debug!("keep_alive_secs {secs} out of range");
                return Err(Error::InvalidInput);
            }
            Ok(Some(TimeDelta::seconds(secs.into())))
        }
    }
}

async fn connection(
    pool: &Pool,
) -> Result<
    bb8::PooledConnection<
        '_,
        diesel_async::pooled_connection::AsyncDieselConnectionManager<AsyncPgConnection>,
    >,
> {
    pool.get().await.map_err(|e| {
        sentry::capture_error(&e);
        Error::DatabaseConnection
    })
}

/// Ends every open alert whose keep-alive ran out, at its `expires_at`.
///
/// Runs before each read and write, so expiry needs no background job and
/// responses never show an expired alert as open.
async fn end_expired(conn: &mut AsyncPgConnection, now: DateTime<Utc>) -> Result<()> {
    diesel::update(
        alert::table
            .filter(alert::ended_at.is_null())
            .filter(alert::expires_at.le(now)),
    )
    .set((
        alert::ended_at.eq(alert::expires_at.nullable()),
        alert::end_reason.eq(END_EXPIRED),
    ))
    .execute(conn)
    .await
    .map_err(|e| {
        sentry::capture_error(&e);
        Error::Update
    })?;
    Ok(())
}

impl Alert {
    /// Fires an alert. See [`CreateAlertDto`].
    ///
    /// A concurrent fire relies on the partial unique index `idx_alert_open`, so
    /// simultaneous fires for the same key cannot open two alerts.
    pub(crate) async fn fire(alert: CreateAlertDto, pool: &Pool) -> Result<Self> {
        let keep_alive = keep_alive(alert.kind, alert.keep_alive_secs)?;
        let now = Utc::now();
        let expires_at = keep_alive.map_or(now, |k| now + k);
        let row = Self {
            id: Uuid::new_v4(),
            severity: alert.severity.into(),
            identifier: alert.identifier,
            value: alert.value.unwrap_or_default(),
            note: alert.note,
            created_at: now,
            datasource_id: parse_datasource_id(&alert.datasource_id)?,
            kind: kind_to_db(alert.kind).to_string(),
            last_seen_at: now,
            expires_at,
            ended_at: keep_alive.is_none().then_some(now),
            end_reason: None,
            occurrences: 1,
        };

        let mut conn = connection(pool).await?;
        let inserted = if keep_alive.is_none() {
            diesel::insert_into(alert::table)
                .values(&row)
                .returning(Self::as_returning())
                .get_result(&mut conn)
                .await
        } else {
            // An expired alert must not absorb this fire, so end it first.
            end_expired(&mut conn, now).await?;
            diesel::insert_into(alert::table)
                .values(&row)
                .on_conflict((alert::datasource_id, alert::identifier))
                .filter_target(alert::ended_at.is_null())
                .do_update()
                .set((
                    alert::severity.eq(excluded(alert::severity)),
                    alert::value.eq(excluded(alert::value)),
                    alert::note.eq(excluded(alert::note)),
                    alert::last_seen_at.eq(excluded(alert::last_seen_at)),
                    alert::expires_at.eq(excluded(alert::expires_at)),
                    alert::occurrences.eq(alert::occurrences + 1),
                ))
                .returning(Self::as_returning())
                .get_result(&mut conn)
                .await
        };
        inserted.map_err(|e| {
            sentry::capture_error(&e);
            Error::Creation
        })
    }

    /// Stops the open alert for `datasource_id` and `identifier`.
    ///
    /// Returns `None` if no alert was open; that is not an error.
    pub(crate) async fn stop(stop: StopAlertDto, pool: &Pool) -> Result<Option<Self>> {
        let datasource_id = parse_datasource_id(&stop.datasource_id)?;
        let now = Utc::now();
        let mut conn = connection(pool).await?;
        end_expired(&mut conn, now).await?;
        let target = alert::table
            .filter(alert::datasource_id.eq(datasource_id))
            .filter(alert::identifier.eq(stop.identifier))
            .filter(alert::ended_at.is_null());
        let ended = (alert::ended_at.eq(now), alert::end_reason.eq(END_STOPPED));
        let result = match stop.note {
            Some(note) => {
                diesel::update(target)
                    .set((ended, alert::note.eq(note)))
                    .returning(Self::as_returning())
                    .get_result(&mut conn)
                    .await
            }
            None => {
                diesel::update(target)
                    .set(ended)
                    .returning(Self::as_returning())
                    .get_result(&mut conn)
                    .await
            }
        };
        result.optional().map_err(|e| {
            sentry::capture_error(&e);
            Error::Update
        })
    }

    pub(crate) async fn query(alert_query: AlertQuery, pool: &Pool) -> Result<Vec<Self>> {
        let mut conn = connection(pool).await?;
        end_expired(&mut conn, Utc::now()).await?;
        let mut query = alert::table.into_boxed();
        if let Some(start) = alert_query.created_at {
            query = query.filter(alert::created_at.ge(start));
        }
        if let Some(datasource_id) = alert_query.datasource_id {
            query = query.filter(alert::datasource_id.eq(datasource_id));
        }
        if let Some(severity) = alert_query.severity {
            let s: Severity = severity.into();
            query = query.filter(alert::severity.eq(s));
        }
        if let Some(identifier) = alert_query.identifier {
            query = query.filter(alert::identifier.eq(identifier));
        }
        if let Some(kind) = alert_query.kind {
            query = query.filter(alert::kind.eq(kind_to_db(kind)));
        }
        match alert_query.open {
            Some(true) => query = query.filter(alert::ended_at.is_null()),
            Some(false) => query = query.filter(alert::ended_at.is_not_null()),
            None => {}
        }
        query.load(&mut conn).await.map_err(|e| {
            sentry::capture_error(&e);
            Error::Find
        })
    }

    pub(crate) async fn aggrigate(
        interval_query: IntervalQuery,
        pool: &Pool,
    ) -> Result<Vec<AggrigatedAlert>> {
        let mut conn = connection(pool).await?;
        end_expired(&mut conn, Utc::now()).await?;

        let mut query = alert::table
            .group_by((alert::datasource_id, alert::severity, alert::identifier))
            .select((
                alert::datasource_id,
                alert::severity,
                alert::identifier,
                diesel::dsl::count(alert::id),
                diesel::dsl::count(alert::ended_at),
                diesel::dsl::min(alert::created_at),
                diesel::dsl::max(alert::last_seen_at),
            ))
            .into_boxed();

        // Include every alert that was active at some point in the interval,
        // not only the ones that started in it.
        if let Some(start) = interval_query.start {
            query = query.filter(alert::ended_at.is_null().or(alert::ended_at.ge(start)));
        }
        if let Some(end) = interval_query.end {
            query = query.filter(alert::created_at.le(end));
        }

        let query = query
            .load::<(
                Uuid,
                Severity,
                String,
                i64,
                i64,
                Option<DateTime<Utc>>,
                Option<DateTime<Utc>>,
            )>(&mut conn)
            .await
            .map_err(|e| {
                sentry::capture_error(&e);
                Error::Find
            })?;

        Ok(query
            .into_iter()
            .map(
                |(datasource_id, severity, identifier, count, ended, first_date, last_date)| {
                    AggrigatedAlert::new(
                        datasource_id,
                        severity,
                        count,
                        count - ended,
                        identifier,
                        first_date,
                        last_date,
                    )
                },
            )
            .collect())
    }
}

impl From<Alert> for AlertDto {
    fn from(alert: Alert) -> Self {
        Self {
            id: alert.id.to_string(),
            severity: alert.severity.into(),
            identifier: alert.identifier.to_string(),
            value: alert.value,
            note: alert.note,
            created_at: alert.created_at.to_rfc3339(),
            datasource_id: alert.datasource_id.to_string(),
            kind: match alert.kind.as_str() {
                KIND_CONCURRENT => AlertKind::Concurrent,
                _ => AlertKind::OneTime,
            },
            last_seen_at: alert.last_seen_at.to_rfc3339(),
            expires_at: alert.expires_at.to_rfc3339(),
            ended_at: alert.ended_at.map(|t| t.to_rfc3339()),
            end_reason: match alert.end_reason.as_deref() {
                Some(END_STOPPED) => Some(EndReason::Stopped),
                Some(END_EXPIRED) => Some(EndReason::Expired),
                _ => None,
            },
            occurrences: alert.occurrences,
        }
    }
}
