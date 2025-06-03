//! 通用服务模块

/// 文件服务
pub mod file {
    use actix_web::HttpResponse;

    // 通用文件服务实现
    pub async fn upload_file() -> Result<String, anyhow::Error> {
        // TODO: 实现文件上传逻辑
        Ok("文件上传成功".to_string())
    }

    pub async fn download_file(file_name: &str, data: Vec<u8>) -> HttpResponse {
        // TODO: 实现文件下载逻辑
        HttpResponse::Ok()
            .content_type("application/octet-stream; charset=UTF-8")
            .append_header(("Access-Control-Allow-Origin", "*"))
            .append_header(("Access-Control-Expose-Headers", "Content-Disposition"))
            .append_header(("Content-Disposition", format!("attachment; filename=\"{}\"", file_name)))
            .append_header(("Content-Length", data.len().to_string()))
            .body(data)
    }
}

/// 验证码服务
pub mod captcha {
    use std::sync::Arc;

    use captcha::{filters::Noise, Captcha};
    use dashmap::DashMap;
    use rand::{thread_rng, Rng};

    use ruoyi_common::enums::CaptchaType;
    use uuid::Uuid;

    pub trait CaptchaService: Send + Sync + 'static {
        fn generate_captcha(
            &self,
            captcha_type: CaptchaType,
        ) -> Result<(String, String), anyhow::Error>;
        fn verify_captcha(&self, uuid: &str, code: &str) -> bool;
    }

    pub struct InMemoryCaptchaService {
        cache: Arc<DashMap<String, String>>,
    }

    impl InMemoryCaptchaService {
        pub fn new(cache: Arc<DashMap<String, String>>) -> Self {
            Self { cache }
        }
        /// 生成计算验证码
        fn gen_math_captcha(&self) -> (String, String) {
            let mut rng = thread_rng();
            let num1: u8 = rng.gen_range(1..=10);
            let num2: u8 = rng.gen_range(1..=10);
            let operators = ["+", "-", "*"];
            let op_idx = rng.gen_range(0..2); // 0: +, 1: -, 2: *
            let operator = operators[op_idx];

            let result = match operator {
                "+" => num1 + num2,
                "-" => {
                    // 确保结果为正数
                    if num1 >= num2 {
                        num1 - num2
                    } else {
                        num2 - num1
                    }
                }
                "*" => num1 * num2,
                _ => unreachable!(),
            };

            // 构建问题
            let question = if operator == "-" && num1 < num2 {
                format!("{} {} {} =", num2, operator, num1)
            } else {
                format!("{} {} {} =", num1, operator, num2)
            };
            let mut captcha = Captcha::new();
            captcha
                .add_chars(4)
                .set_chars(&question.chars().collect::<Vec<_>>())
                .apply_filter(Noise::new(0.1))
                .view(160, 60);
            let base64 = captcha.as_base64().unwrap_or_default();
            (base64, result.to_string())
        }

        /// 生成字母数字验证码
        fn gen_alphanum_captcha(&self) -> (String, String) {
            let mut captcha = Captcha::new();
            captcha
                .add_chars(4)
                .apply_filter(Noise::new(0.1))
                .view(160, 60);

            let code = captcha.chars_as_string();
            let base64 = captcha.as_base64().unwrap_or_default();

            (base64, code)
        }
    }
    impl CaptchaService for InMemoryCaptchaService {
        fn generate_captcha(
            &self,
            captcha_type: CaptchaType,
        ) -> Result<(String, String), anyhow::Error> {
            // 根据类型生成验证码
            let (img, code) = match captcha_type {
                CaptchaType::Math => self.gen_math_captcha(),
                CaptchaType::AlphaNumeric => self.gen_alphanum_captcha(),
            };

            // 生成唯一标识
            let uuid = Uuid::new_v4().to_string();

            // 将验证码存入缓存
            self.cache.insert(uuid.clone(), code);

            Ok((img, uuid))
        }

        /// 验证验证码
        fn verify_captcha(&self, uuid: &str, code: &str) -> bool {
            if let Some((_, _)) = self.cache.remove_if(uuid, |_, stored_code| {
                stored_code.eq_ignore_ascii_case(code)
            }) {
                true
            } else {
                false
            }
        }
    }
}
