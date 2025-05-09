//! 控制器模块，处理各类 HTTP 请求

// 通用控制器相关函数
pub mod common {
    use actix_web::{get, web, HttpResponse};

    use log::error;

    use ruoyi_common::{enums::CaptchaType, vo::R};
    use serde::Serialize;
    use std::collections::HashMap;

    use crate::web::service::captcha::{CaptchaService, InMemoryCaptchaService};
    /// 健康检查
    #[get("/health")]
    pub async fn health_check() -> HttpResponse {
        HttpResponse::Ok().json(R::<()>::ok_with_msg("健康检查成功"))
    }
    /// 验证码响应
    #[derive(Debug, Serialize)]
    pub struct CaptchaResponse {
        /// 验证码图片 Base64
        pub img: String,
        /// 验证码唯一标识
        pub uuid: String,
        /// 验证码开关
        #[serde(rename = "captchaEnabled")]
        pub captcha_enabled: bool,
        /// 验证码图片类型
        #[serde(rename = "captchaType")]
        pub captcha_type: String,
    }

    /// 验证码接口
    #[get("/captchaImage")]
    pub async fn captcha_image(
        query: web::Query<HashMap<String, String>>,
        captcha_service: web::Data<InMemoryCaptchaService>,
    ) -> HttpResponse {
        // 默认为字母数字混合验证码
        let captcha_type_str = match query.get("type").map(|s| s.as_str()) {
            Some("math") => "math",
            _ => "char",
        };

        let captcha_type = match captcha_type_str {
            "math" => CaptchaType::Math,
            _ => CaptchaType::AlphaNumeric,
        };

        let (img, uuid) = match captcha_service.generate_captcha(captcha_type) {
            Ok((img, uuid)) => (img, uuid),
            Err(e) => {
                error!("生成验证码失败: {}", e);
                return HttpResponse::InternalServerError()
                    .json(R::<String>::fail("生成验证码失败"));
            }
        };
        // 不使用中间结构，直接返回JSON响应
        HttpResponse::Ok().json(R::<CaptchaResponse>::ok_with_data(CaptchaResponse {
            img,
            uuid,
            captcha_enabled: true,
            captcha_type: captcha_type_str.to_string(),
        }))
    }

    pub async fn upload() -> HttpResponse {
        // TODO: 实现文件上传逻辑
        HttpResponse::Ok().json(R::<()>::ok_with_msg("文件上传功能待实现"))
    }
}