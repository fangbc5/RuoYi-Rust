/** 数据库字符串类型 */
pub const COLUMNTYPE_STR: [&str; 4] = ["char", "varchar", "nvarchar", "varchar2"];

/** 数据库文本类型 */
pub const COLUMNTYPE_TEXT: [&str; 4] = ["tinytext", "text", "mediumtext", "longtext"];

/** 数据库时间类型 */
pub const COLUMNTYPE_TIME: [&str; 4] = ["datetime", "time", "date", "timestamp"];

/** 数据库数字类型 */
pub const COLUMNTYPE_NUMBER: [&str; 11] = [
    "tinyint",
    "smallint",
    "mediumint",
    "int",
    "number",
    "integer",
    "bit",
    "bigint",
    "float",
    "double",
    "decimal",
];

/** 页面不需要编辑字段 */
pub const COLUMNNAME_NOT_EDIT: [&str; 4] = ["id", "create_by", "create_time", "del_flag"];

/** 页面不需要显示的列表字段 */
pub const COLUMNNAME_NOT_LIST: [&str; 6] = [
    "id",
    "create_by",
    "create_time",
    "del_flag",
    "update_by",
    "update_time",
];

/** 页面不需要查询字段 */
pub const COLUMNNAME_NOT_QUERY: [&str; 7] = [
    "id",
    "create_by",
    "create_time",
    "del_flag",
    "update_by",
    "update_time",
    "remark",
];

pub const HTML_TEXTAREA: &str = "textarea";
pub const HTML_INPUT: &str = "input";

/** 下拉框 */
pub const HTML_SELECT: &str = "select";

/** 单选框 */
pub const HTML_RADIO: &str = "radio";

/** 复选框 */
pub const HTML_CHECKBOX: &str = "checkbox";

/** 日期控件 */
pub const HTML_DATETIME: &str = "datetime";

/** 图片上传控件 */
pub const HTML_IMAGE_UPLOAD: &str = "imageUpload";

/** 文件上传控件 */
pub const HTML_FILE_UPLOAD: &str = "fileUpload";

/** 富文本控件 */
pub const HTML_EDITOR: &str = "editor";

/** 字符串类型 */
pub const TYPE_STRING: &str = "String";

/** 整型 */
pub const TYPE_INTEGER: &str = "Integer";

/** 长整型 */
pub const TYPE_LONG: &str = "Long";

/** 浮点型 */
pub const TYPE_DOUBLE: &str = "Double";

/** 高精度计算类型 */
pub const TYPE_BIGDECIMAL: &str = "BigDecimal";

/** 时间类型 */
pub const TYPE_DATE: &str = "Date";

/** 模糊查询 */
pub const QUERY_LIKE: &str = "LIKE";

/** 相等查询 */
pub const QUERY_EQ: &str = "EQ";

/** 需要 */
pub const REQUIRE: &str = "1";
