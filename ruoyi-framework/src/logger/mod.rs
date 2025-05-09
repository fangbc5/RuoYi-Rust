pub mod database_appender;
pub mod entity;
use std::{str::FromStr, sync::Arc, time::Duration};

use anyhow::Result;
use database_appender::DatabaseAppender;
use log::info;
use log4rs::{
    append::{
        console::ConsoleAppender,
        rolling_file::{
            policy::compound::{roll::fixed_window::FixedWindowRoller, trigger::size::SizeTrigger},
            RollingFileAppender,
        },
    },
    config::{Appender, Logger, Root},
    encode::pattern::PatternEncoder,
    Config,
};
use sea_orm::DatabaseConnection;

pub fn init_logger() -> Result<()> {
    let config = log4rs::config::load_config_file("config/log4rs.yaml", Default::default())?;
    log4rs::init_config(config)?;

    info!("日志初始化成功");
    Ok(())
}

pub fn init_logger_with_db(db: Arc<DatabaseConnection>) -> Result<()> {
    // 创建控制台输出器
    let console_appender = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "[{d(%Y-%m-%d %H:%M:%S%.3f)}] [{l}] [{M}:{L}] {m}\n",
        )))
        .build();

    // 创建文件输出器
    let rolling_file_appender = RollingFileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "[{d(%Y-%m-%d %H:%M:%S%.3f)}] [{l}] [{M}:{L}] {m}\n",
        )))
        .build(
            "logs/app.log",
            Box::new(
                log4rs::append::rolling_file::policy::compound::CompoundPolicy::new(
                    Box::new(SizeTrigger::new(10 * 1024 * 1024)), // 10 MB
                    Box::new(
                        FixedWindowRoller::builder()
                            .base(1)
                            .build("logs/app.{}.log", 5)
                            .unwrap(),
                    ),
                ),
            ),
        )
        .unwrap();
    let database_appender = DatabaseAppender::new(db, 100, Duration::from_secs(1));

    // 环境变量中获取日志级别
    let log_level = std::env::var("RUST_LOG").unwrap_or_default();
    let log_level = log::LevelFilter::from_str(&log_level).unwrap_or(log::LevelFilter::Info);

    // 从配置文件初始化（只需调用一次）
    let config = Config::builder()
        .appender(Appender::builder().build("console", Box::new(console_appender)))
        .appender(Appender::builder().build("rolling_file", Box::new(rolling_file_appender)))
        .appender(Appender::builder().build("database", Box::new(database_appender)))
        .logger(
            Logger::builder()
                .additive(false)
                .build("actix_server", log::LevelFilter::Off),
        )
        .logger(
            Logger::builder()
                .additive(false)
                .build("sqlx", log::LevelFilter::Off),
        )
        .logger(
            Logger::builder()
                .additive(false)
                .build("login_info", log::LevelFilter::Info),
        )
        .logger(
            Logger::builder()
                .additive(false)
                .build("oper_log", log::LevelFilter::Info),
        )
        .build(
            Root::builder()
                .appender("console")
                .appender("rolling_file")
                .appender("database")
                .build(log_level),
        )?;
    // 初始化日志系统
    log4rs::init_config(config)?;

    info!("日志初始化成功");
    Ok(())
}
