use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GenConfig {
    /// 作者
    #[serde(default = "default_author")]
    pub author: Option<String>,

    /// 生成包路径
    #[serde(default = "default_package_name")]
    pub package_name: Option<String>,

    /// 自动去除表前缀
    #[serde(default = "default_auto_remove_pre")]
    pub auto_remove_pre: bool,

    /// 表前缀
    #[serde(default = "default_table_prefix")]
    pub table_prefix: Option<String>,

    /// 是否允许生成文件覆盖到本地（自定义路径）
    #[serde(default = "default_allow_overwrite")]
    pub allow_overwrite: bool,
}

fn default_author() -> Option<String> {
    Some("ruoyi".to_string())
}

fn default_package_name() -> Option<String> {
    Some("com.ruoyi".to_string())
}

fn default_auto_remove_pre() -> bool {
    true
}

fn default_table_prefix() -> Option<String> {
    None
}

fn default_allow_overwrite() -> bool {
    false
}
