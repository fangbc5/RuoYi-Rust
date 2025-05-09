use std::{sync::Arc, time::Duration};

use anyhow::Result;
use log::{error, info, Level, Record};
use sea_orm::{DatabaseConnection, EntityTrait, IntoActiveModel, Set};
use tokio::{
    sync::{mpsc, oneshot},
    time::interval,
};

use super::entity::*;

#[derive(Debug)]
pub enum LogMessage {
    LoginInfo(LoginInfoActiveModel),
    OperLog(OperLogActiveModel),
    Flush(oneshot::Sender<()>), // 用于手动刷新并获取完成通知
    Shutdown,
}

#[derive(Debug)]
pub struct DatabaseAppender {
    modules: Vec<String>,
    tx: mpsc::Sender<LogMessage>,
}

impl DatabaseAppender {
    pub fn new(db: Arc<DatabaseConnection>, batch_size: usize, interval: Duration) -> Self {
        let modules = vec![
            "system::login_info".to_string(),
            "system::oper_log".to_string(),
        ];
        let (tx, rx) = mpsc::channel::<LogMessage>(1024);

        // 启动单一后台任务处理所有日志事件
        Self::spawn_log_processor(db.clone(), rx, batch_size, interval);

        Self { modules, tx }
    }

    // 启动日志处理器
    fn spawn_log_processor(
        db: Arc<DatabaseConnection>,
        mut rx: mpsc::Receiver<LogMessage>,
        batch_size: usize,
        flush_interval: Duration,
    ) {
        tokio::spawn(async move {
            let mut login_buffer = Vec::with_capacity(batch_size);
            let mut oper_buffer = Vec::with_capacity(batch_size);
            let mut interval_timer = interval(flush_interval);
            interval_timer.tick().await; // 消费第一个立即触发的tick

            loop {
                tokio::select! {
                    // 处理新的日志消息
                    Some(message) = rx.recv() => {
                        match message {
                            LogMessage::LoginInfo(log) => {
                                login_buffer.push(log);
                                // 如果达到批处理阈值，立即刷新
                                info!("登录日志缓冲区长度: {}", login_buffer.len());
                                if login_buffer.len() >= batch_size {
                                    Self::flush_login_info(&db, &mut login_buffer).await;
                                }
                            }
                            LogMessage::OperLog(log) => {
                                oper_buffer.push(log);
                                // 如果达到批处理阈值，立即刷新
                                info!("操作日志缓冲区长度: {}", oper_buffer.len());
                                if oper_buffer.len() >= batch_size {
                                    Self::flush_oper_log(&db, &mut oper_buffer).await;
                                }
                            }
                            LogMessage::Flush(notify) => {
                                // 手动刷新所有缓冲区
                                Self::flush_login_info(&db, &mut login_buffer).await;
                                Self::flush_oper_log(&db, &mut oper_buffer).await;
                                let _ = notify.send(());  // 通知刷新完成
                            }
                            LogMessage::Shutdown => {
                                // 清空缓冲区并退出
                                Self::flush_login_info(&db, &mut login_buffer).await;
                                Self::flush_oper_log(&db, &mut oper_buffer).await;
                                log::info!("日志处理器正常关闭");
                                break;
                            }
                        }
                    }
                    // 定时刷新
                    _ = interval_timer.tick() => {
                        // 执行定时刷新
                        if !login_buffer.is_empty() {
                            Self::flush_login_info(&db, &mut login_buffer).await;
                        }
                        if !oper_buffer.is_empty() {
                            Self::flush_oper_log(&db, &mut oper_buffer).await;
                        }
                    }
                    // 如果所有通道都关闭，则退出
                    else => break,
                }
            }
        });
    }

    // 刷新登录日志到数据库
    async fn flush_login_info(db: &DatabaseConnection, buffer: &mut Vec<LoginInfoActiveModel>) {
        if buffer.is_empty() {
            return;
        }

        let logs_to_insert = std::mem::take(buffer);
        let count = logs_to_insert.len();
        match LoginInfoEntity::insert_many(logs_to_insert).exec(db).await {
            Ok(res) => {
                log::info!("成功批量插入{}条登录日志, 最后插入ID: {}", count, res.last_insert_id);
            }
            Err(e) => {
                log::error!("批量插入登录日志失败: {}", e);
            }
        }
    }

    // 刷新操作日志到数据库
    async fn flush_oper_log(db: &DatabaseConnection, buffer: &mut Vec<OperLogActiveModel>) {
        if buffer.is_empty() {
            return;
        }

        let logs_to_insert = std::mem::take(buffer);
        let count = logs_to_insert.len();
        match OperLogEntity::insert_many(logs_to_insert).exec(db).await {
            Ok(res) => {
                log::info!("成功批量插入{}条操作日志, 最后插入ID: {}", count, res.last_insert_id);
            }
            Err(e) => {
                log::error!("批量插入操作日志失败: {}", e);
            }
        }
    }

    pub fn from_record_to_login_log(&self, record: &Record) -> Option<LoginInfoActiveModel> {
        let args = record.args().to_string();
        match serde_json::from_str::<LoginInfoModel>(&args) {
            Ok(login_info) => {
                let mut login_info = login_info.into_active_model();
                // 根据日志级别设置状态
                if record.level() == Level::Error || record.level() == Level::Warn {
                    login_info.status = Set(Some("1".to_string())); // 失败
                } else {
                    login_info.status = Set(Some("0".to_string())); // 成功
                }
                Some(login_info)
            }
            Err(e) => {
                log::error!("解析登录日志失败: {}, 内容: {}", e, args);
                None
            }
        }
    }

    pub fn from_record_to_oper_log(&self, record: &Record) -> Option<OperLogActiveModel> {
        let args = record.args().to_string();
        match serde_json::from_str::<OperLogModel>(&args) {
            Ok(oper_log) => {
                let mut oper_log = oper_log.into_active_model();
                // 根据日志级别设置状态
                if record.level() == Level::Error || record.level() == Level::Warn {
                    oper_log.status = Set(Some(1)); // 失败
                } else {
                    oper_log.status = Set(Some(0)); // 成功
                }
                Some(oper_log)
            }
            Err(e) => {
                log::error!("解析操作日志失败: {}, 内容: {}", e, args);
                None
            }
        }
    }

    // 手动刷新所有缓冲区
    pub fn manual_flush(&self) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        if let Err(e) = self.tx.try_send(LogMessage::Flush(tx)) {
            log::error!("发送刷新命令失败: {}", e);
            return Err(anyhow::anyhow!("发送刷新命令失败: {}", e));
        }

        // 转换为同步等待完成通知
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                match tokio::time::timeout(Duration::from_secs(5), rx).await {
                    Ok(Ok(())) => Ok(()),
                    Ok(Err(_)) => Err(anyhow::anyhow!("刷新操作被取消")),
                    Err(_) => Err(anyhow::anyhow!("刷新操作超时")),
                }
            })
        })
    }
}

impl log4rs::append::Append for DatabaseAppender {
    fn append(&self, record: &Record) -> Result<()> {
        if let Some(module) = self
            .modules
            .iter()
            .find(|m| record.target().starts_with(m.as_str()))
        {
            // 根据模块类型处理不同的日志
            if module.contains("login_info") {
                // 处理登录日志
                if let Some(login_info) = self.from_record_to_login_log(record) {
                    // 发送到处理队列
                    if let Err(e) = self.tx.try_send(LogMessage::LoginInfo(login_info)) {
                        error!("发送登录日志失败: {}", e);
                    }
                }
            } else if module.contains("oper_log") {
                // 处理操作日志
                if let Some(oper_log) = self.from_record_to_oper_log(record) {
                    // 发送到处理队列
                    if let Err(e) = self.tx.try_send(LogMessage::OperLog(oper_log)) {
                        error!("发送操作日志失败: {}", e);
                    }
                }
            }
        }

        Ok(())
    }

    fn flush(&self) {
        // 调用手动刷新
        if let Err(e) = self.manual_flush() {
            log::error!("刷新日志失败: {}", e);
        }
    }
}

impl Drop for DatabaseAppender {
    fn drop(&mut self) {
        // 在实例被销毁时发送关闭信号
        let _ = self.tx.try_send(LogMessage::Shutdown);
    }
}
