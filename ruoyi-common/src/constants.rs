// ruoyi-common/src/constants.rs
//! 常量定义

/// 状态常量
pub mod status {
    /// 成功状态码
    pub const SUCCESS: i32 = 200;
    /// 失败状态码
    pub const ERROR: i32 = 500;
}

/// 通用常量
pub mod common {
    /// 默认成功消息
    pub const DEFAULT_SUCCESS_MESSAGE: &str = "操作成功";
    /// 默认失败消息
    pub const DEFAULT_ERROR_MESSAGE: &str = "操作失败";
}

/// 用户常量
pub mod user {
    /// 正常状态
    pub const NORMAL: &str = "0";
    /// 停用状态
    pub const DISABLE: &str = "1";
    /// 删除标记
    pub const DELETED: &str = "2";
    /// 管理员ID
    pub const ADMIN_ID: i64 = 1;
}

/// 菜单常量
pub mod menu {
    /// 菜单类型（目录）
    pub const TYPE_DIR: &str = "M";
    /// 菜单类型（菜单）
    pub const TYPE_MENU: &str = "C";
    /// 菜单类型（按钮）
    pub const TYPE_BUTTON: &str = "F";
    /// 显示状态（显示）
    pub const VISIBLE: &str = "0";
    /// 显示状态（隐藏）
    pub const HIDDEN: &str = "1";
}

/// 缓存常量
pub mod cache {
    /// 用户信息    
    pub const USER_INFO_KEY: &str = "user_info_key";
    /// 配置信息
    pub const SYS_CONFIG_KEY: &str = "sys_config_key";
    /// 字典信息
    pub const SYS_DICT_KEY: &str = "sys_dict_key";
    /// 验证码
    pub const CAPTCHA_KEY: &str = "captcha_key";
    /// 防重提交
    pub const REPEAT_SUBMIT_KEY: &str = "repeat_submit_key";
    /// 限流处理
    pub const RATE_LIMIT_KEY: &str = "rate_limit_key";
    /// 密码错误次数
    pub const PWD_ERR_CNT_KEY: &str = "pwd_err_cnt_key";

    /// Token 前缀
    pub const TOKEN_PREFIX: &str = "login_tokens:";
    /// 验证码前缀
    pub const CAPTCHA_PREFIX: &str = "captcha_codes:";
    /// 参数缓存前缀
    pub const SYS_CONFIG_PREFIX: &str = "sys_config:";
    /// 部门缓存前缀
    pub const SYS_DEPT_PREFIX: &str = "sys_dept:";
    /// 字典缓存前缀
    pub const SYS_DICT_PREFIX: &str = "sys_dict:";
}

/// 字典类型常量
pub mod dict_type {
    /// 系统状态
    pub const SYS_NORMAL_DISABLE: &str = "sys_normal_disable";
    /// 用户性别
    pub const SYS_USER_SEX: &str = "sys_user_sex";
    /// 菜单状态
    pub const SYS_SHOW_HIDE: &str = "sys_show_hide";
    /// 系统开关
    pub const SYS_YES_NO: &str = "sys_yes_no";
    /// 任务状态
    pub const SYS_JOB_STATUS: &str = "sys_job_status";
    /// 任务分组
    pub const SYS_JOB_GROUP: &str = "sys_job_group";
    /// 系统是否
    pub const SYS_WHETHER: &str = "sys_whether";
    /// 操作类型
    pub const SYS_OPER_TYPE: &str = "sys_oper_type";
    /// 系统状态
    pub const SYS_COMMON_STATUS: &str = "sys_common_status";
    /// 通知类型
    pub const SYS_NOTICE_TYPE: &str = "sys_notice_type";
    /// 通知状态
    pub const SYS_NOTICE_STATUS: &str = "sys_notice_status";
    /// 系统是否
    pub const SYS_YESNO: &str = "sys_yes_no";
}

/// 权限常量
pub mod permission {
    /// 超级管理员角色ID
    pub const ADMIN_ROLE_ID: i64 = 1;
    /// 管理员角色唯一标识
    pub const ADMIN_ROLE_KEY: &str = "admin";
}
