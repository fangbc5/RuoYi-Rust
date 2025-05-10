// ruoyi-common/src/utils/time.rs
//! 时间处理工具模块

use chrono::{DateTime, Local, NaiveDateTime, Utc};
use serde::Deserialize;

/// 获取当前时间戳
pub fn current_timestamp() -> i64 {
    Utc::now().timestamp()
}

/// 获取当前时间
pub fn current_datetime() -> DateTime<Utc> {
    Utc::now()
}

/// 格式化时间
pub fn format_datetime(dt: &DateTime<Utc>, format: &str) -> String {
    dt.format(format).to_string()
}

/// 解析时间字符串
pub fn parse_datetime(
    datetime_str: &str,
    format: &str,
) -> Result<DateTime<Utc>, chrono::ParseError> {
    let native_dt = NaiveDateTime::parse_from_str(datetime_str, format)?;
    Ok(DateTime::<Utc>::from_naive_utc_and_offset(native_dt, Utc))
}

/// 转换为本地时间
pub fn to_local(dt: &DateTime<Utc>) -> DateTime<Local> {
    dt.with_timezone(&Local)
}

/// 序列化响应的时间格式
pub fn serialize_optional_datetime<S>(
    value: &Option<DateTime<Utc>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    if let Some(dt) = value {
        serializer.serialize_str(&format_datetime(dt, "%Y-%m-%d %H:%M:%S"))
    } else {
        serializer.serialize_none()
    }
}

// 添加自定义反序列化函数，处理可选日期时间
pub fn deserialize_optional_datetime<'de, D>(
    deserializer: D,
) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    if let Some(s) = s {
        if s.is_empty() {
            return Ok(None);
        }

        // 尝试解析日期字符串
        match chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d") {
            Ok(date) => {
                // 转换为DateTime<Utc>
                let datetime = chrono::DateTime::<Utc>::from_naive_utc_and_offset(
                    date.and_hms_opt(0, 0, 0).unwrap(),
                    Utc,
                );
                Ok(Some(datetime))
            }
            Err(_) => {
                //解析失败时尝试使用带时分秒的格式解析
                match chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S") {
                    Ok(datetime) => {
                        // 转换为DateTime<Utc>
                        let datetime =
                            chrono::DateTime::<Utc>::from_naive_utc_and_offset(datetime, Utc);
                        Ok(Some(datetime))
                    }
                    Err(_) => Err(serde::de::Error::custom("无法解析日期时间字符串")),
                }
            }
        }
    } else {
        Ok(None)
    }
}

/// 格式化时间间隔
pub fn format_duration(duration: std::time::Duration) -> String {
    let years = duration.as_secs() / 3600 / 24 / 365;
    let months = (duration.as_secs() % (3600 * 24 * 365)) / (3600 * 24 * 30);
    let days = (duration.as_secs() % (3600 * 24 * 365)) / (3600 * 24);
    let hours = (duration.as_secs() % (3600 * 24)) / 3600;
    let minutes = (duration.as_secs() % 3600) / 60;
    let seconds = duration.as_secs() % 60;
    if years > 0 {
        format!(
            "{}年{}月{}日 {}小时{}分钟{}秒",
            years, months, days, hours, minutes, seconds
        )
    } else if months > 0 {
        format!(
            "{}月{}日 {}小时{}分钟{}秒",
            months, days, hours, minutes, seconds
        )
    } else if days > 0 {
        format!("{}日 {}小时{}分钟{}秒", days, hours, minutes, seconds)
    } else {
        format!("{}小时{}分钟{}秒", hours, minutes, seconds)
    }
}
