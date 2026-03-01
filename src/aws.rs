//! AWS CloudWatch Logs との通信を担うモジュール。
//!
//! SDK クライアントの構築と、ロググループ・ログストリーム・ログイベントの
//! フェッチ関数を提供します。

use anyhow::{Result, bail};
use aws_config::BehaviorVersion;
use aws_sdk_cloudwatchlogs::Client;

use crate::app::{LogEvent, LogGroup, LogStream};

/// AWS CloudWatch Logs SDK クライアントを構築して返します。
///
/// `~/.aws/config` または環境変数から AWS 設定を読み込みます。
/// リージョンが設定されていない場合はエラーを返します。
///
/// # Errors
///
/// - AWS リージョンが未設定の場合
pub async fn build_client() -> Result<Client> {
    let config = aws_config::defaults(BehaviorVersion::latest()).load().await;

    // Verify region is set
    if config.region().is_none() {
        bail!("AWS region is not configured. Set AWS_DEFAULT_REGION or configure ~/.aws/config.");
    }

    Ok(Client::new(&config))
}

/// ロググループの一覧を取得します。
///
/// ページネーションに対応しており、`next_token` を渡すことで続きのページを取得できます。
///
/// # Returns
///
/// `(ロググループ一覧, 次ページトークン)` のタプルを返します。
///
/// # Errors
///
/// - AWS API 呼び出しに失敗した場合（認証エラー・ネットワークエラーなど）
pub async fn fetch_log_groups(
    client: &Client,
    next_token: Option<String>,
) -> Result<(Vec<LogGroup>, Option<String>)> {
    let mut req = client.describe_log_groups();
    if let Some(token) = next_token {
        req = req.next_token(token);
    }
    let resp = req.send().await.map_err(|e| {
        anyhow::anyhow!(
            "Failed to fetch log groups: {}. Run `aws sso login` if credentials expired.",
            e
        )
    })?;

    let groups = resp
        .log_groups()
        .iter()
        .filter_map(|g| {
            g.log_group_name().map(|n| LogGroup {
                name: n.to_string(),
            })
        })
        .collect();

    Ok((groups, resp.next_token().map(String::from)))
}

/// 指定ロググループのログストリーム一覧を取得します。
///
/// ストリームは最終イベント時刻の降順でソートされます。
///
/// # Arguments
///
/// * `group_name` - 対象のロググループ名
/// * `next_token` - ページネーショントークン（初回は `None`）
///
/// # Returns
///
/// `(ログストリーム一覧, 次ページトークン)` のタプルを返します。
///
/// # Errors
///
/// - AWS API 呼び出しに失敗した場合
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
    let resp = req
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch log streams: {}", e))?;

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

/// ログイベントを取得します（`FilterLogEvents` API を使用）。
///
/// ストリーム名・時間範囲・フィルタパターンを任意で指定できます。
/// `stream_name` が `None` の場合はロググループ全体を検索します。
///
/// # Arguments
///
/// * `group_name` - 対象のロググループ名
/// * `stream_name` - 対象のログストリーム名（`None` でグループ全体）
/// * `start_time_ms` - 検索開始時刻（Unix ミリ秒、`None` で無制限）
/// * `end_time_ms` - 検索終了時刻（Unix ミリ秒、`None` で無制限）
/// * `filter_pattern` - CloudWatch Logs フィルタパターン（`None` または空文字で全件）
/// * `next_token` - ページネーショントークン（初回は `None`）
///
/// # Returns
///
/// `(ログイベント一覧, 次ページトークン)` のタプルを返します。
///
/// # Errors
///
/// - AWS API 呼び出しに失敗した場合
pub async fn fetch_log_events(
    client: &Client,
    group_name: &str,
    stream_name: Option<&str>,
    start_time_ms: Option<i64>,
    end_time_ms: Option<i64>,
    filter_pattern: Option<String>,
    next_token: Option<String>,
) -> Result<(Vec<LogEvent>, Option<String>)> {
    let mut req = client.filter_log_events().log_group_name(group_name);
    if let Some(name) = stream_name {
        req = req.log_stream_names(name);
    }
    if let Some(start_time) = start_time_ms {
        req = req.start_time(start_time);
    }
    if let Some(end_time) = end_time_ms {
        req = req.end_time(end_time);
    }

    if let Some(pattern) = filter_pattern
        && !pattern.is_empty()
    {
        req = req.filter_pattern(pattern);
    }
    if let Some(token) = next_token {
        req = req.next_token(token);
    }

    let resp = req
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch log events: {}", e))?;

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
