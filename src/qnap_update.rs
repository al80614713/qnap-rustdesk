//! QNAP RustDesk 6 个月强制更新双保险校验模块。
//!
//! 策略：
//! 1. 在线版本清单 JSON 校验：请求编译期注入的 `QNAP_UPDATE_MANIFEST_URL`，
//!    对比当前版本与清单中的 `min_version`，低于最低版本即阻断。
//! 2. 编译满 180 天兜底到期：无论在线清单是否可达，只要当前系统时间超过
//!    `QNAP_BUILD_EPOCH + QNAP_EXPIRE_DAYS * 86400`，立即阻断。
//! 3. 阻断效果：界面弹出不可关闭的强制升级框，同时 Rust 底层拦截主控/被控连接。

use serde_derive::{Deserialize, Serialize};
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const DEFAULT_EXPIRE_DAYS: u64 = 180;
const DEFAULT_DOWNLOAD_URL: &str = "https://github.com/al80614713/qnap-rustdesk/releases/latest";
const MANIFEST_CACHE_TTL_SECS: u64 = 1800;
const MANIFEST_HTTP_TIMEOUT_SECS: u64 = 5;

#[derive(Debug, Clone, Deserialize)]
struct UpdateManifest {
    #[serde(default)]
    min_version: String,
    #[serde(default)]
    latest_version: String,
    #[serde(default)]
    download_url: String,
    #[serde(default)]
    message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForceUpdateStatus {
    pub blocked: bool,
    pub reason: String,
    pub message: String,
    pub current_version: String,
    pub min_version: String,
    pub latest_version: String,
    pub download_url: String,
    pub expires_at_unix: u64,
    pub source: String,
}

struct CachedCheck {
    checked_at_unix: u64,
    status: ForceUpdateStatus,
}

static CACHED_STATUS: Mutex<Option<CachedCheck>> = Mutex::new(None);

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn build_epoch() -> u64 {
    option_env!("QNAP_BUILD_EPOCH")
        .and_then(|v| v.trim().parse::<u64>().ok())
        .unwrap_or(0)
}

fn expire_days() -> u64 {
    option_env!("QNAP_EXPIRE_DAYS")
        .and_then(|v| v.trim().parse::<u64>().ok())
        .filter(|&d| d > 0)
        .unwrap_or(DEFAULT_EXPIRE_DAYS)
}

fn manifest_url() -> &'static str {
    option_env!("QNAP_UPDATE_MANIFEST_URL").unwrap_or("").trim()
}

fn parse_version_tuple(v: &str) -> Vec<u64> {
    let core = v.trim().trim_start_matches(['v', 'V']);
    let core = core.split(['-', '+']).next().unwrap_or(core);
    core.split('.')
        .map(|part| part.parse::<u64>().unwrap_or(0))
        .collect()
}

/// 比较版本号，若 `current < min_version` 返回 `true`。
pub fn is_version_lower(current: &str, min_version: &str) -> bool {
    if min_version.trim().is_empty() {
        return false;
    }
    let a = parse_version_tuple(current);
    let b = parse_version_tuple(min_version);
    let max_len = a.len().max(b.len());
    for i in 0..max_len {
        let av = *a.get(i).unwrap_or(&0);
        let bv = *b.get(i).unwrap_or(&0);
        if av < bv {
            return true;
        }
        if av > bv {
            return false;
        }
    }
    false
}

fn evaluate_status(
    now: u64,
    build_ts: u64,
    days: u64,
    current_ver: &str,
    manifest: Option<&UpdateManifest>,
) -> ForceUpdateStatus {
    let expires_at_unix = if build_ts > 0 {
        build_ts.saturating_add(days.saturating_mul(86_400))
    } else {
        0
    };
    let download_url = manifest
        .map(|m| m.download_url.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or(DEFAULT_DOWNLOAD_URL)
        .to_owned();
    let min_version = manifest
        .map(|m| m.min_version.trim().to_owned())
        .unwrap_or_default();
    let latest_version = manifest
        .map(|m| m.latest_version.trim().to_owned())
        .unwrap_or_default();
    let custom_msg = manifest
        .map(|m| m.message.trim().to_owned())
        .unwrap_or_default();

    // 1. 优先判定在线清单 min_version
    if !min_version.is_empty() && is_version_lower(current_ver, &min_version) {
        let message = if custom_msg.is_empty() {
            format!(
                "当前 QNAP RustDesk 版本（{}）低于最低要求版本（{}），已停止远控服务，请下载安装最新版本。",
                current_ver, min_version
            )
        } else {
            custom_msg
        };
        return ForceUpdateStatus {
            blocked: true,
            reason: "min_version".to_owned(),
            message,
            current_version: current_ver.to_owned(),
            min_version,
            latest_version,
            download_url,
            expires_at_unix,
            source: "manifest".to_owned(),
        };
    }

    // 2. 180 天编译兜底到期判定
    if expires_at_unix > 0 && now >= expires_at_unix {
        let message = if custom_msg.is_empty() {
            format!(
                "当前 QNAP RustDesk（{}）已超过 {} 天有效期，已自动停用远控功能，请下载安装最新版本。",
                current_ver, days
            )
        } else {
            custom_msg
        };
        return ForceUpdateStatus {
            blocked: true,
            reason: "expired".to_owned(),
            message,
            current_version: current_ver.to_owned(),
            min_version,
            latest_version,
            download_url,
            expires_at_unix,
            source: "build_ttl".to_owned(),
        };
    }

    ForceUpdateStatus {
        blocked: false,
        reason: String::new(),
        message: String::new(),
        current_version: current_ver.to_owned(),
        min_version,
        latest_version,
        download_url,
        expires_at_unix,
        source: if manifest.is_some() {
            "manifest".to_owned()
        } else {
            "build_ttl".to_owned()
        },
    }
}

fn fetch_manifest_blocking(url: &str) -> Option<UpdateManifest> {
    if url.is_empty() {
        return None;
    }
    let client = match reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(MANIFEST_HTTP_TIMEOUT_SECS))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            log::warn!("qnap_update: failed to build http client: {}", e);
            return None;
        }
    };
    match client.get(url).send() {
        Ok(resp) if resp.status().is_success() => match resp.json::<UpdateManifest>() {
            Ok(m) => Some(m),
            Err(e) => {
                log::warn!("qnap_update: invalid manifest json: {}", e);
                None
            }
        },
        Ok(resp) => {
            log::warn!("qnap_update: manifest http status {}", resp.status());
            None
        }
        Err(e) => {
            log::warn!("qnap_update: failed to fetch manifest: {}", e);
            None
        }
    }
}

/// 获取强制更新状态；当 `allow_network` 为 true 且缓存过期时拉取在线版本清单。
pub fn get_status(allow_network: bool) -> ForceUpdateStatus {
    let now = now_unix();
    if let Ok(guard) = CACHED_STATUS.lock() {
        if let Some(cached) = guard.as_ref() {
            if !allow_network || now.saturating_sub(cached.checked_at_unix) < MANIFEST_CACHE_TTL_SECS
            {
                return cached.status.clone();
            }
        }
    }

    let manifest = if allow_network {
        fetch_manifest_blocking(manifest_url())
    } else {
        None
    };

    let status = evaluate_status(now, build_epoch(), expire_days(), crate::VERSION, manifest.as_ref());

    if allow_network {
        if let Ok(mut guard) = CACHED_STATUS.lock() {
            *guard = Some(CachedCheck {
                checked_at_unix: now,
                status: status.clone(),
            });
        }
    }
    status
}

/// 供异步 FFI（`main_get_common`）调用：允许联网检查并返回 JSON。
pub fn check_and_get_status_json() -> String {
    let status = get_status(true);
    serde_json::to_string(&status).unwrap_or_default()
}

/// 供同步 FFI（`main_get_common_sync`）调用：仅读缓存 + 180 天兜底，不阻塞网络。
pub fn cached_status_json() -> String {
    let status = get_status(false);
    serde_json::to_string(&status).unwrap_or_default()
}

/// 供 Rust 底层主控/被控连接入口调用：若被阻断，返回友好提示文本。
/// 若尚未联网检查过，仅基于缓存与 180 天兜底做即时判定，避免阻塞异步运行时；
/// 同时后台线程会由前端启动检查刷新缓存。
pub fn block_reason() -> Option<String> {
    let status = get_status(false);
    if status.blocked {
        Some(status.message)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_compare() {
        assert!(is_version_lower("1.5.3", "1.5.4"));
        assert!(is_version_lower("1.5.3", "1.6.0"));
        assert!(is_version_lower("1.5.3", "2.0.0"));
        assert!(!is_version_lower("1.5.3", "1.5.3"));
        assert!(!is_version_lower("1.5.4", "1.5.3"));
        assert!(!is_version_lower("1.5.3", ""));
        assert!(is_version_lower("v1.5.3-beta", "1.5.4"));
    }

    #[test]
    fn test_manifest_blocks_old_version() {
        let manifest = UpdateManifest {
            min_version: "1.6.0".to_owned(),
            latest_version: "1.6.1".to_owned(),
            download_url: "https://example.com/dl".to_owned(),
            message: String::new(),
        };
        let status = evaluate_status(1_700_000_000, 1_699_000_000, 180, "1.5.3", Some(&manifest));
        assert!(status.blocked);
        assert_eq!(status.reason, "min_version");
        assert_eq!(status.download_url, "https://example.com/dl");
    }

    #[test]
    fn test_build_ttl_blocks_after_180_days() {
        let build_ts = 1_700_000_000u64;
        let after_181_days = build_ts + 181 * 86_400;
        let status = evaluate_status(after_181_days, build_ts, 180, "1.5.3", None);
        assert!(status.blocked);
        assert_eq!(status.reason, "expired");
    }

    #[test]
    fn test_valid_within_180_days_and_version_ok() {
        let build_ts = 1_700_000_000u64;
        let after_10_days = build_ts + 10 * 86_400;
        let manifest = UpdateManifest {
            min_version: "1.5.0".to_owned(),
            latest_version: "1.5.3".to_owned(),
            download_url: "https://example.com/dl".to_owned(),
            message: String::new(),
        };
        let status = evaluate_status(after_10_days, build_ts, 180, "1.5.3", Some(&manifest));
        assert!(!status.blocked);
    }
}
