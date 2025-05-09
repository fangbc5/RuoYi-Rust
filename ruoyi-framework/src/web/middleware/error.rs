// ruoyi-framework/src/middleware/error.rs
//! 错误处理中间件

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::StatusCode,
    web, Error, HttpResponse,
};
use futures::future::{ready, LocalBoxFuture, Ready};
use log::{error, warn};

use ruoyi_common::vo::R;

/// 错误处理中间件
pub struct ErrorHandling;

impl ErrorHandling {
    /// 创建一个新的错误处理中间件实例
    pub fn new() -> Self {
        Self {}
    }
}

impl<S, B> Transform<S, ServiceRequest> for ErrorHandling
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = ErrorHandlingMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ErrorHandlingMiddleware { service }))
    }
}

/// 错误处理中间件实现
pub struct ErrorHandlingMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for ErrorHandlingMiddleware<S>
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
        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await;

            match res {
                Ok(response) => {
                    // 处理成功响应
                    // 根据实际需求进行处理
                    Ok(response)
                }
                Err(err) => {
                    // 记录错误日志
                    error!("请求处理错误: {}", err);

                    // 根据错误类型构建特定响应
                    match err.as_response_error().status_code() {
                        StatusCode::NOT_FOUND => {
                            warn!("资源未找到: {}", err);
                            Err(err)
                        }
                        StatusCode::UNAUTHORIZED => {
                            warn!("未授权: {}", err);
                            Err(err)
                        }
                        StatusCode::FORBIDDEN => {
                            warn!("禁止访问: {}", err);
                            Err(err)
                        }
                        StatusCode::BAD_REQUEST => {
                            warn!("请求错误: {}", err);
                            Err(err)
                        }
                        _ => {
                            error!("服务器内部错误: {}", err);
                            Err(err)
                        }
                    }
                }
            }
        })
    }
}

/// 全局错误捕获配置
pub fn config_error_handlers(cfg: &mut web::ServiceConfig) {
    // 这里可以配置全局错误捕获处理器
    cfg.app_data(web::JsonConfig::default().error_handler(|err, _| {
        let message = format!("JSON解析错误: {}", err);
        actix_web::error::InternalError::from_response(
            err,
            HttpResponse::BadRequest().json(R::<()>::error_with_msg(&message)),
        )
        .into()
    }));

    cfg.app_data(web::FormConfig::default().error_handler(|err, _| {
        let message = format!("表单解析错误: {}", err);
        actix_web::error::InternalError::from_response(
            err,
            HttpResponse::BadRequest().json(R::<()>::error_with_msg(&message)),
        )
        .into()
    }));

    cfg.app_data(web::PathConfig::default().error_handler(|err, _| {
        let message = format!("路径参数错误: {}", err);
        actix_web::error::InternalError::from_response(
            err,
            HttpResponse::BadRequest().json(R::<()>::error_with_msg(&message)),
        )
        .into()
    }));

    cfg.app_data(web::QueryConfig::default().error_handler(|err, _| {
        let message = format!("查询参数错误: {}", err);
        actix_web::error::InternalError::from_response(
            err,
            HttpResponse::BadRequest().json(R::<()>::error_with_msg(&message)),
        )
        .into()
    }));
}
