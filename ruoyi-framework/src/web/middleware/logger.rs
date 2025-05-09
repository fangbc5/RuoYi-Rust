// ruoyi-framework/src/middleware/logger.rs
//! 日志中间件

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures::future::{ready, LocalBoxFuture, Ready};
use log::info;
use ruoyi_common::utils::ip;
use std::time::Instant;

/// 请求日志中间件
pub struct RequestLogger;

impl RequestLogger {
    /// 创建一个新的请求日志中间件实例
    pub fn new() -> Self {
        Self {}
    }
}

impl<S, B> Transform<S, ServiceRequest> for RequestLogger
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RequestLoggerMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestLoggerMiddleware { service }))
    }
}

/// 请求日志中间件实现
pub struct RequestLoggerMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for RequestLoggerMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let start_time = Instant::now();
        let method = req.method().to_string();
        let path = req.path().to_string();
        let ip = ip::get_real_ip_by_middleware(&req);
        let query_string = req.query_string().to_string();

        // 记录请求开始
        info!(
            "收到请求: {} {} 来自 IP: {} 查询参数: {}",
            method, path, ip, query_string
        );

        let fut = self.service.call(req);

        Box::pin(async move {
            // 等待请求处理完成
            let res = fut.await?;

            // 计算请求处理时间
            let duration = start_time.elapsed();
            let status = res.status().as_u16();

            // 记录请求结束
            info!(
                "请求完成: {} {} 状态码: {} 耗时: {:?} 来自 IP: {}",
                method, path, status, duration, ip
            );

            Ok(res)
        })
    }
}
