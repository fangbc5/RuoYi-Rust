//! Web 相关功能模块

use actix_web::web;

pub mod controller;
pub mod filter;
pub mod middleware;
pub mod service;
pub mod tls;

pub fn register_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(controller::common::health_check);
    cfg.service(controller::common::captcha_image);
}
