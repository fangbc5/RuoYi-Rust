//! 请求过滤器模块，用于在请求处理前后执行特定逻辑

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures::future::{ready, LocalBoxFuture, Ready};
use log::info;
use std::time::Instant;

/// 请求耗时统计过滤器
pub struct RequestTimeFilter;

impl<S, B> Transform<S, ServiceRequest> for RequestTimeFilter
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RequestTimeMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestTimeMiddleware { service }))
    }
}

/// 请求耗时中间件
pub struct RequestTimeMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for RequestTimeMiddleware<S>
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

        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;
            let duration = start_time.elapsed();

            info!("请求 {} {} 处理耗时: {:?}", method, path, duration);

            Ok(res)
        })
    }
}

/// 操作日志过滤器（记录用户操作）
pub struct OperationLogFilter;

// TODO: 实现操作日志过滤器

/// XSS 防御过滤器
pub struct XssFilter;

// TODO: 实现 XSS 防御过滤器

/// SQL 注入防御过滤器
pub struct SqlInjectionFilter;

// TODO: 实现 SQL 注入防御过滤器
