// ruoyi-framework/src/middleware/response.rs
//! 响应处理中间件

use actix_web::body::MessageBody;
use actix_web::{
    body::BoxBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse,
};
use futures::future::{ready, LocalBoxFuture, Ready};
use serde::Serialize;
use std::rc::Rc;

use ruoyi_common::vo::R;

/// 响应包装中间件
pub struct ResponseWrapper;

impl ResponseWrapper {
    /// 创建新的响应包装中间件实例
    pub fn new() -> Self {
        Self {}
    }
}

impl<S, B> Transform<S, ServiceRequest> for ResponseWrapper
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Transform = ResponseWrapperMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ResponseWrapperMiddleware {
            service: Rc::new(service),
        }))
    }
}

/// 响应包装中间件实现
pub struct ResponseWrapperMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for ResponseWrapperMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();

        Box::pin(async move {
            let res = svc.call(req).await?;

            // 获取响应状态
            let status = res.status();

            // 如果是 Actix-Web 的内部错误处理，不包装
            if status.is_client_error() || status.is_server_error() {
                return Ok(res.map_into_boxed_body());
            }

            // 如果是流响应，不包装
            if res.headers().contains_key("content-disposition") {
                return Ok(res.map_into_boxed_body());
            }

            // 处理正常响应 - 简化处理，保持原样
            Ok(res.map_into_boxed_body())
        })
    }
}

/// 封装响应数据
pub fn wrap_response<T: Serialize>(data: T) -> HttpResponse {
    HttpResponse::Ok().json(R::ok_with_data(data))
}

/// 封装成功响应
pub fn success_response() -> HttpResponse {
    HttpResponse::Ok().json(R::<()>::ok())
}

/// 封装成功消息响应
pub fn success_msg_response(msg: &str) -> HttpResponse {
    HttpResponse::Ok().json(R::<()>::ok_with_msg(msg))
}

/// 封装错误响应
pub fn error_response(code: i32, msg: &str) -> HttpResponse {
    HttpResponse::Ok().json(R::<()>::error_with_code_msg(code, msg))
}
