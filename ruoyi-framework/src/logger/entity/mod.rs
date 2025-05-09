pub mod login_info;
pub mod oper_log;

pub use oper_log::{
    ActiveModel as OperLogActiveModel, Column as OperLogColumn, Entity as OperLogEntity,
    Model as OperLogModel,
};

pub use login_info::{
    ActiveModel as LoginInfoActiveModel, Column as LoginInfoColumn, Entity as LoginInfoEntity,
    Model as LoginInfoModel,
};

