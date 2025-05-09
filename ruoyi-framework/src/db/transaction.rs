// ruoyi-framework/src/db/transaction.rs
//! 事务管理模块

use sea_orm::{DatabaseConnection, DbErr, TransactionTrait};
use std::future::Future;
use std::pin::Pin;

/// 事务管理器
pub struct TransactionManager<'a> {
    /// 数据库连接
    conn: &'a DatabaseConnection,
}

impl<'a> TransactionManager<'a> {
    /// 创建新的事务管理器
    pub fn new(conn: &'a DatabaseConnection) -> Self {
        Self { conn }
    }

    /// 在事务中执行操作
    pub async fn transaction<F, T, E>(&self, f: F) -> Result<T, E>
    where
        F: for<'b> FnOnce(
            &'b sea_orm::DatabaseTransaction,
        ) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'b>>,
        E: From<DbErr>,
    {
        let txn = self.conn.begin().await.map_err(E::from)?;

        match f(&txn).await {
            Ok(result) => {
                txn.commit().await.map_err(E::from)?;
                Ok(result)
            }
            Err(e) => {
                // 回滚事务（忽略可能的回滚错误）
                let _ = txn.rollback().await;
                Err(e)
            }
        }
    }
}
