use anyhow::{bail, Result};
use aws_config::BehaviorVersion;
use aws_sdk_cloudwatchlogs::Client;

use crate::app::{LogEvent, LogGroup, LogStream};

pub async fn build_client() -> Result<Client> {
    let config = aws_config::defaults(BehaviorVersion::latest()).load().await;

    // Verify region is set
    if config.region().is_none() {
        bail!(
            "AWS region is not configured. Set AWS_DEFAULT_REGION or configure ~/.aws/config."
        );
    }

    Ok(Client::new(&config))
}

pub async fn fetch_log_groups(
    client: &Client,
    next_token: Option<String>,
) -> Result<(Vec<LogGroup>, Option<String>)> {
    let mut req = client.describe_log_groups();
    if let Some(token) = next_token {
        req = req.next_token(token);
    }
    let resp = req.send().await.map_err(|e| {
        anyhow::anyhow!("Failed to fetch log groups: {}. Run `aws sso login` if credentials expired.", e)
    })?;

    let groups = resp
        .log_groups()
        .iter()
        .filter_map(|g| g.log_group_name().map(|n| LogGroup { name: n.to_string() }))
        .collect();

    Ok((groups, resp.next_token().map(String::from)))
}

pub async fn fetch_log_streams(
    client: &Client,
    group_name: &str,
    next_token: Option<String>,
) -> Result<(Vec<LogStream>, Option<String>)> {
    let mut req = client
        .describe_log_streams()
        .log_group_name(group_name)
        .order_by(aws_sdk_cloudwatchlogs::types::OrderBy::LastEventTime)
        .descending(true);
    if let Some(token) = next_token {
        req = req.next_token(token);
    }
    let resp = req.send().await.map_err(|e| {
        anyhow::anyhow!("Failed to fetch log streams: {}", e)
    })?;

    let streams = resp
        .log_streams()
        .iter()
        .filter_map(|s| {
            s.log_stream_name().map(|n| LogStream {
                name: n.to_string(),
                last_event_time: s.last_event_timestamp(),
            })
        })
        .collect();

    Ok((streams, resp.next_token().map(String::from)))
}

pub async fn fetch_log_events(
    client: &Client,
    group_name: &str,
    stream_name: Option<&str>,
    start_time_ms: Option<i64>,
    end_time_ms: Option<i64>,
    filter_pattern: Option<String>,
    next_token: Option<String>,
) -> Result<(Vec<LogEvent>, Option<String>)> {
    let mut req = client
        .filter_log_events()
        .log_group_name(group_name);
    if let Some(name) = stream_name {
        req = req.log_stream_names(name);
    }
    if let Some(start_time) = start_time_ms {
        req = req.start_time(start_time);
    }
    if let Some(end_time) = end_time_ms {
        req = req.end_time(end_time);
    }

    if let Some(pattern) = filter_pattern {
        if !pattern.is_empty() {
            req = req.filter_pattern(pattern);
        }
    }
    if let Some(token) = next_token {
        req = req.next_token(token);
    }

    let resp = req.send().await.map_err(|e| {
        anyhow::anyhow!("Failed to fetch log events: {}", e)
    })?;

    let events = resp
        .events()
        .iter()
        .map(|e| LogEvent {
            timestamp: e.timestamp().unwrap_or(0),
            message: e.message().unwrap_or("").to_string(),
        })
        .collect();

    Ok((events, resp.next_token().map(String::from)))
}
