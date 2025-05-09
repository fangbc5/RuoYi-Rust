// ruoyi-common/src/vo.rs
//! VO (Value Object) 对象定义

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 通用响应结构
#[derive(Debug, Serialize, Deserialize)]
pub struct R<T> {
    /// 状态码
    pub code: i32,
    /// 消息
    pub msg: String,
    /// 数据
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub data: Option<T>,
}

impl<T> R<T> {
    /// 创建成功响应
    pub fn ok() -> R<T> {
        R {
            code: super::constants::status::SUCCESS,
            msg: super::constants::common::DEFAULT_SUCCESS_MESSAGE.to_string(),
            data: None,
        }
    }

    /// 创建带数据的成功响应
    pub fn ok_with_data(data: T) -> R<T> {
        R {
            code: super::constants::status::SUCCESS,
            msg: super::constants::common::DEFAULT_SUCCESS_MESSAGE.to_string(),
            data: Some(data),
        }
    }

    /// 创建带消息的成功响应
    pub fn ok_with_msg(msg: &str) -> R<T> {
        R {
            code: super::constants::status::SUCCESS,
            msg: msg.to_string(),
            data: None,
        }
    }

    /// 创建带数据和消息的成功响应
    pub fn ok_with_data_msg(data: T, msg: &str) -> R<T> {
        R {
            code: super::constants::status::SUCCESS,
            msg: msg.to_string(),
            data: Some(data),
        }
    }

    /// 创建错误响应
    pub fn error() -> R<T> {
        R {
            code: super::constants::status::ERROR,
            msg: super::constants::common::DEFAULT_ERROR_MESSAGE.to_string(),
            data: None,
        }
    }

    /// 创建带消息的错误响应
    pub fn error_with_msg(msg: &str) -> R<T> {
        R {
            code: super::constants::status::ERROR,
            msg: msg.to_string(),
            data: None,
        }
    }

    /// 创建带消息的错误响应（fail是error_with_msg的别名）
    pub fn fail(msg: &str) -> R<T> {
        Self::error_with_msg(msg)
    }

    /// 创建带消息的错误响应（err是error_with_msg的别名）
    pub fn err(msg: &str) -> R<T> {
        Self::error_with_msg(msg)
    }

    /// 创建带状态码和消息的错误响应
    pub fn error_with_code_msg(code: i32, msg: &str) -> R<T> {
        R {
            code,
            msg: msg.to_string(),
            data: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RList<T> {
    pub code: i32,
    pub msg: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub data: Vec<T>,
}

impl<T> RList<T> {
    pub fn ok_with_data(data: Vec<T>) -> RList<T> {
        RList {
            code: super::constants::status::SUCCESS,
            msg: super::constants::common::DEFAULT_SUCCESS_MESSAGE.to_string(),
            data,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RData<T> {
    pub code: i32,
    pub msg: String,
    pub data: T,
}

impl<T> RData<T> {
    pub fn ok(data: T) -> RData<T> {
        RData {
            code: super::constants::status::SUCCESS,
            msg: super::constants::common::DEFAULT_SUCCESS_MESSAGE.to_string(),
            data,
        }
    }
}

/// 分页查询参数
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PageParam {
    /// 当前页码
    #[serde(default = "default_page_num")]
    pub page_num: u64,
    /// 每页数量
    #[serde(default = "default_page_size")]
    pub page_size: u64,
    /// 排序字段
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_by_column: Option<String>,
    /// 排序方向 (ASC/DESC)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_asc: Option<String>,
}

/// 默认每页数量
fn default_page_size() -> u64 {
    10
}

/// 默认页码
fn default_page_num() -> u64 {
    1
}

/// 分页信息
#[derive(Debug, Serialize, Deserialize)]
pub struct PageInfo<T> {
    /// 总记录数
    pub total: u64,
    /// 列表数据
    pub list: Vec<T>,
    /// 当前页码
    pub page_num: u64,
    /// 每页数量
    pub page_size: u64,
}

impl<T> PageInfo<T> {
    /// 创建新的分页信息
    pub fn new(total: u64, list: Vec<T>, page_param: &PageParam) -> Self {
        PageInfo {
            total,
            list,
            page_num: page_param.page_num,
            page_size: page_param.page_size,
        }
    }
}

/// 日期查询参数
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DateParam {
    #[serde(rename = "beginTime")]
    pub begin_time: Option<DateTime<Utc>>,
    #[serde(rename = "endTime")]
    pub end_time: Option<DateTime<Utc>>,
}
