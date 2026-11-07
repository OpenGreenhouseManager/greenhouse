//! The alert API over both tables: one-time alerts (`alert`) and concurrent
//! alerts (`concurrent_alert`).

use super::aggrigated_alert::AggrigatedAlert;
use super::alert_models::{AggregateRow, Alert};
use super::concurrent_alert_models::ConcurrentAlert;
use super::{Error, Result};
use crate::Pool;
use chrono::Utc;
use diesel_async::AsyncPgConnection;
use greenhouse_core::data_storage_service_dto::alert_dto::{
    alert::{AlertDto, AlertKind},
    post_create_alert::{CreateAlertDto, DEFAULT_KEEP_ALIVE_SECS, MAX_KEEP_ALIVE_SECS},
    post_stop_alert::StopAlertDto,
    query::{AlertQuery, IntervalQuery},
};
use std::collections::HashMap;
use uuid::Uuid;

type Connection<'a> = bb8::PooledConnection<
    'a,
    diesel_async::pooled_connection::AsyncDieselConnectionManager<AsyncPgConnection>,
>;

async fn connection(pool: &Pool) -> Result<Connection<'_>> {
    pool.get().await.map_err(|e| {
        sentry::capture_error(&e);
        Error::DatabaseConnection
    })
}

fn parse_datasource_id(datasource_id: &str) -> Result<Uuid> {
    datasource_id.parse().map_err(|e| {
        tracing::debug!("invalid datasource_id {datasource_id:?}: {e}");
        Error::InvalidInput
    })
}

/// Fires an alert. See [`CreateAlertDto`].
pub(crate) async fn fire(alert: CreateAlertDto, pool: &Pool) -> Result<AlertDto> {
    let datasource_id = parse_datasource_id(&alert.datasource_id)?;
    match (alert.kind, alert.keep_alive_secs) {
        (AlertKind::OneTime, Some(_)) => {
            tracing::debug!("keep_alive_secs is only allowed for concurrent alerts");
            Err(Error::InvalidInput)
        }
        (AlertKind::OneTime, None) => {
            let mut conn = connection(pool).await?;
            Ok(Alert::create(alert, datasource_id, &mut conn).await?.into())
        }
        (AlertKind::Concurrent, keep_alive_secs) => {
            let keep_alive_secs = keep_alive_secs.unwrap_or(DEFAULT_KEEP_ALIVE_SECS);
            if !(1..=MAX_KEEP_ALIVE_SECS).contains(&keep_alive_secs) {
                tracing::debug!("keep_alive_secs {keep_alive_secs} out of range");
                return Err(Error::InvalidInput);
            }
            let mut conn = connection(pool).await?;
            let alert =
                ConcurrentAlert::fire(alert, datasource_id, keep_alive_secs, &mut conn).await?;
            Ok(alert.into_dto(Utc::now()))
        }
    }
}

/// Stops an open concurrent alert. `None` if none was open.
pub(crate) async fn stop(stop: StopAlertDto, pool: &Pool) -> Result<Option<AlertDto>> {
    let datasource_id = parse_datasource_id(&stop.datasource_id)?;
    let mut conn = connection(pool).await?;
    let stopped =
        ConcurrentAlert::stop(datasource_id, stop.identifier, stop.note, &mut conn).await?;
    Ok(stopped.map(|a| a.into_dto(Utc::now())))
}

/// Alerts of both kinds matching `query`, oldest first.
pub(crate) async fn query(query: AlertQuery, pool: &Pool) -> Result<Vec<AlertDto>> {
    let mut conn = connection(pool).await?;
    let mut alerts = Vec::new();
    // One-time alerts are never open.
    if query.kind != Some(AlertKind::Concurrent) && query.open != Some(true) {
        alerts.extend(
            Alert::query(&query, &mut conn)
                .await?
                .into_iter()
                .map(AlertDto::from),
        );
    }
    if query.kind != Some(AlertKind::OneTime) {
        let now = Utc::now();
        alerts.extend(
            ConcurrentAlert::query(&query, &mut conn)
                .await?
                .into_iter()
                .map(|a| a.into_dto(now)),
        );
    }
    alerts.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    Ok(alerts)
}

/// Alerts of both kinds active in the interval, grouped by datasource,
/// severity and identifier.
pub(crate) async fn aggregate(
    interval: IntervalQuery,
    pool: &Pool,
) -> Result<Vec<AggrigatedAlert>> {
    let mut conn = connection(pool).await?;
    let mut rows = Alert::aggregate(&interval, &mut conn).await?;
    rows.extend(ConcurrentAlert::aggregate(&interval, &mut conn).await?);

    let mut groups: HashMap<_, AggregateRow> = HashMap::new();
    for row in rows {
        let key = (row.0, row.1.clone(), row.2.clone());
        match groups.get_mut(&key) {
            None => {
                groups.insert(key, row);
            }
            Some(group) => {
                group.3 += row.3;
                group.4 += row.4;
                group.5 = [group.5, row.5].into_iter().flatten().min();
                group.6 = [group.6, row.6].into_iter().flatten().max();
            }
        }
    }
    Ok(groups
        .into_values()
        .map(|(ds, sev, ident, count, open, first, last)| {
            AggrigatedAlert::new(ds, sev, count, open, ident, first, last)
        })
        .collect())
}
