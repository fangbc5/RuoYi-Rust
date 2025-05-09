// ruoyi-common/src/utils/string.rs
//! 字符串处理工具模块
use rand::{distributions::Alphanumeric, Rng};
use regex::Regex;
use serde::Deserialize;
use serde_json;
use std::{borrow::Cow, collections::HashMap};
use uuid::Uuid;

/// 生成UUID
pub fn uuid() -> String {
    Uuid::new_v4().to_string()
}

/// 生成随机字符串
pub fn random_string(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

/// 首字母大写
pub fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

/// 下划线转驼峰
pub fn to_camel_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;

    for c in s.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}

/// 驼峰转下划线
pub fn to_snake_case(s: &str) -> String {
    let mut result = String::new();

    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }

    result
}

/// 截取字符串
pub fn substring(s: &str, start: usize, end: Option<usize>) -> Cow<str> {
    let len = s.len();
    let end = end.unwrap_or(len);

    if start >= len || end <= start {
        return Cow::Borrowed("");
    }

    let s_bytes = s.as_bytes();
    let mut start_idx = start;
    let mut end_idx = end.min(len);

    // 确保不会在UTF-8字符的中间截断
    while start_idx < len && !is_char_boundary(s, start_idx) {
        start_idx += 1;
    }
    while end_idx > 0 && !is_char_boundary(s, end_idx) {
        end_idx -= 1;
    }

    if start_idx >= end_idx {
        return Cow::Borrowed("");
    }

    Cow::Borrowed(unsafe { std::str::from_utf8_unchecked(&s_bytes[start_idx..end_idx]) })
}

/// 检查字符边界
fn is_char_boundary(s: &str, idx: usize) -> bool {
    if idx == 0 || idx == s.len() {
        return true;
    }
    let b = s.as_bytes()[idx];
    b & 0xc0 != 0x80
}

/// 缓存正则表达式
pub fn regex_match(s: &str, pattern: &Regex) -> bool {
    pattern.is_match(s)
}

pub fn regex_from_pattern(pattern: &str) -> Regex {
    Regex::new(pattern).unwrap()
}

pub fn option_is_empty(option: &Option<String>) -> bool {
    option.is_none() || option.as_ref().unwrap().trim().is_empty()
}

pub fn string_to_vec_u8(s: &str) -> Vec<u8> {
    s.as_bytes().to_vec()
}

pub fn deserialize_str_to_i32<'de, D>(deserializer: D) -> std::result::Result<Option<i32>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    // 使用 serde_json::Value 接收反序列化结果，它可以处理各种类型
    let value = serde_json::Value::deserialize(deserializer)?;

    if value.is_null() {
        return Ok(None);
    } else if let Some(i) = value.as_i64() {
        // 处理整数
        return Ok(Some(i as i32));
    } else if let Some(s) = value.as_str() {
        // 处理字符串，尝试解析为整数
        if let Ok(i) = s.parse::<i32>() {
            return Ok(Some(i));
        }
    }
    // 处理其他情况
    Ok(None)
}

pub fn serialize_i32_to_string<S>(value: &Option<i32>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match value {
        Some(i) => serializer.serialize_str(&i.to_string()),
        None => serializer.serialize_none(),
    }
}

pub fn serialize_vec_u8_to_string<S>(
    value: &Option<Vec<u8>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match value {
        Some(v) => serializer.serialize_str(String::from_utf8_lossy(v).as_ref()),
        None => serializer.serialize_none(),
    }
}

pub fn redis_info_to_map(info: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in info.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut parts = line.splitn(2, ':');
        if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
            map.insert(key.to_string(), value.to_string());
        }
    }
    map
}

pub fn redis_command_stats_to_map(info: &str) -> Vec<HashMap<String, String>> {
    let mut list = Vec::new();
    for line in info.lines() {
        let mut parts = line.splitn(2, ':');
        if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
            let mut map = HashMap::new();
            // 将key中的前缀去掉
            let key = key.replace("cmdstat_", "");
            // 截取出value中的calls部分
            let value = value
                .split(',')
                .find(|part| part.trim().starts_with("calls="))
                .map(|part| part.trim().replace("calls=", ""))
                .unwrap_or_else(|| "0".to_string());
            map.insert("name".to_string(), key.to_string());
            map.insert("value".to_string(), value);
            list.push(map);
        }
    }
    list
}
