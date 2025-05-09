use std::fmt;

use num_enum::IntoPrimitive;
use serde::Deserialize;

/// 验证码类型
#[derive(Debug, Deserialize)]
pub enum CaptchaType {
    /// 计算题
    Math,
    /// 字母数字混合
    AlphaNumeric,
}

/// 菜单类型枚举
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuType {
    /// 目录
    Directory,
    /// 菜单
    Menu,
    /// 按钮
    Button,
}

impl MenuType {
    /// 从字符串创建菜单类型
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "M" => Some(MenuType::Directory),
            "C" => Some(MenuType::Menu),
            "F" => Some(MenuType::Button),
            _ => None,
        }
    }

    /// 转换为字符串
    pub fn to_string(&self) -> String {
        match self {
            MenuType::Directory => "M".to_string(),
            MenuType::Menu => "C".to_string(),
            MenuType::Button => "F".to_string(),
        }
    }
}

impl fmt::Display for MenuType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// 操作日志类型枚举
#[derive(Debug, Deserialize, IntoPrimitive)]
#[repr(i32)]
pub enum OperLogBusinessType {
    /// 其它
    Other = 0,
    /// 新增
    Insert = 1,
    /// 修改
    Update = 2,
    /// 删除
    Delete = 3,
    /// 授权
    Auth = 4,
    /// 导出
    Export = 5,
    /// 导入
    Import = 6,
    /// 强退
    ForceLogout = 7,
    /// 清空数据
    ClearData = 8,
}

#[derive(Debug, Deserialize, IntoPrimitive)]
#[repr(i32)]
pub enum OperLogOperatorType {
    /// 其它
    Other = 0,
    /// 后台用户
    Web = 1,
    /// 手机端用户
    Mobile = 2,
}
