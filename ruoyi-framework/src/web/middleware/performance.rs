//! 自定义中间件示例

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures::future::{ready, LocalBoxFuture, Ready};
use log::info;
use std::time::Instant;

/// 性能监控中间件
///
/// 用于监控请求处理性能，记录处理时间超过阈值的请求
pub struct PerformanceMonitor {
    /// 性能阈值（毫秒）
    threshold_ms: u64,
}

impl PerformanceMonitor {
    /// 创建新的性能监控中间件
    pub fn new(threshold_ms: u64) -> Self {
        Self { threshold_ms }
    }
}

impl<S, B> Transform<S, ServiceRequest> for PerformanceMonitor
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = PerformanceMonitorMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(PerformanceMonitorMiddleware {
            service,
            threshold_ms: self.threshold_ms,
        }))
    }
}

/// 性能监控中间件实现
pub struct PerformanceMonitorMiddleware<S> {
    service: S,
    threshold_ms: u64,
}

impl<S, B> Service<ServiceRequest> for PerformanceMonitorMiddleware<S>
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
        let threshold = self.threshold_ms;

        Box::pin(async move {
            // 执行请求处理
            let res = fut.await?;

            // 计算请求处理时间
            let duration = start_time.elapsed();
            let duration_ms = duration.as_millis() as u64;

            // 如果处理时间超过阈值，记录警告日志
            if duration_ms > threshold {
                info!(
                    "性能警告: {} {} 处理耗时 {}ms，超过阈值 {}ms",
                    method, path, duration_ms, threshold
                );
            }

            Ok(res)
        })
    }
}
