use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GenConfig {
    /// 作者
    pub author: Option<String>,

    /// 生成包路径
    pub package_name: Option<String>,

    /// 自动去除表前缀
    pub auto_remove_pre: bool,

    /// 表前缀
    pub table_prefix: Option<String>,

    /// 是否允许生成文件覆盖到本地（自定义路径）
    pub allow_overwrite: bool,
}
