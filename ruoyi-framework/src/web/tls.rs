use std::cell::RefCell;
use std::net::IpAddr;
use tokio::task_local;

// 定义用户上下文
#[derive(Debug, Clone)]
pub struct UserContext {
    pub user_id: i64,
    pub user_name: String,
    pub ip: IpAddr,
}

// 定义任务本地存储
task_local! {
    static ASYNC_USER_CONTEXT: Option<UserContext>;
}

pub async fn set_async_user_context(ctx: UserContext) {
    ASYNC_USER_CONTEXT
        .scope(Some(ctx), async {
            save_async_user_context().await;
        })
        .await;
}

async fn save_async_user_context() {
    if let Some(_ctx) = ASYNC_USER_CONTEXT.with(|ctx| ctx.clone()) {
        // 使用 ctx.user_id 操作数据库
    }
}

// 使用线程本地存储
thread_local! {
    static SYNC_USER_CONTEXT: RefCell<Option<UserContext>> = RefCell::new(None);
}

// 设置用户上下文
pub fn set_sync_user_context(ctx: UserContext) {
    SYNC_USER_CONTEXT.with(|cell| {
        *cell.borrow_mut() = Some(ctx);
    });
}

// 获取用户上下文
pub fn get_sync_user_context() -> Option<UserContext> {
    SYNC_USER_CONTEXT.with(|cell| cell.borrow().clone())
}
