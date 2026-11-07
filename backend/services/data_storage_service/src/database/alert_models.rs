//! One-time alerts: single events, one row each, in the `alert` table.

use super::severity_models::Severity;
use super::{Error, Result, schema::alert};
use chrono::{DateTime, Utc};
use diesel::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use greenhouse_core::data_storage_service_dto::alert_dto::{
    alert::{AlertDto, AlertKind},
    post_create_alert::CreateAlertDto,
    query::{AlertQuery, IntervalQuery},
};
use uuid::Uuid;

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
}

/// One group of the aggregated view: datasource, severity, identifier, count,
/// open count, first start, last fire.
pub(crate) type AggregateRow = (
    Uuid,
    Severity,
    String,
    i64,
    i64,
    Option<DateTime<Utc>>,
    Option<DateTime<Utc>>,
);

impl Alert {
    pub(crate) async fn create(
        alert: CreateAlertDto,
        datasource_id: Uuid,
        conn: &mut AsyncPgConnection,
    ) -> Result<Self> {
        let alert = Self {
            id: Uuid::new_v4(),
            severity: alert.severity.into(),
            identifier: alert.identifier,
            value: alert.value.unwrap_or_default(),
            note: alert.note,
            created_at: Utc::now(),
            datasource_id,
        };
        diesel::insert_into(alert::table)
            .values(&alert)
            .execute(conn)
            .await
            .map_err(|e| {
                sentry::capture_error(&e);
                Error::Creation
            })?;
        Ok(alert)
    }

    pub(crate) async fn query(
        alert_query: &AlertQuery,
        conn: &mut AsyncPgConnection,
    ) -> Result<Vec<Self>> {
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
        if let Some(identifier) = &alert_query.identifier {
            query = query.filter(alert::identifier.eq(identifier));
        }
        query.load(conn).await.map_err(|e| {
            sentry::capture_error(&e);
            Error::Find
        })
    }

    /// One-time alerts that happened in the interval, grouped. They are never open.
    pub(crate) async fn aggregate(
        interval: &IntervalQuery,
        conn: &mut AsyncPgConnection,
    ) -> Result<Vec<AggregateRow>> {
        let mut query = alert::table
            .group_by((alert::datasource_id, alert::severity, alert::identifier))
            .select((
                alert::datasource_id,
                alert::severity,
                alert::identifier,
                diesel::dsl::count(alert::id),
                diesel::dsl::min(alert::created_at),
                diesel::dsl::max(alert::created_at),
            ))
            .into_boxed();
        if let Some(start) = interval.start {
            query = query.filter(alert::created_at.ge(start));
        }
        if let Some(end) = interval.end {
            query = query.filter(alert::created_at.le(end));
        }
        let rows = query
            .load::<(
                Uuid,
                Severity,
                String,
                i64,
                Option<DateTime<Utc>>,
                Option<DateTime<Utc>>,
            )>(conn)
            .await
            .map_err(|e| {
                sentry::capture_error(&e);
                Error::Find
            })?;
        Ok(rows
            .into_iter()
            .map(|(ds, sev, ident, count, first, last)| (ds, sev, ident, count, 0, first, last))
            .collect())
    }
}

impl From<Alert> for AlertDto {
    fn from(alert: Alert) -> Self {
        let created_at = alert.created_at.to_rfc3339();
        Self {
            id: alert.id.to_string(),
            severity: alert.severity.into(),
            identifier: alert.identifier,
            value: alert.value,
            note: alert.note,
            last_seen_at: created_at.clone(),
            expires_at: created_at.clone(),
            created_at,
            datasource_id: alert.datasource_id.to_string(),
            kind: AlertKind::OneTime,
            open: false,
            keep_alive_secs: None,
            end_reason: None,
            occurrences: 1,
        }
    }
}
