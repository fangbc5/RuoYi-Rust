use actix_web::web;

/// 使用sea-orm生成entity命令
/// sea-orm-cli generate entity --tables gen_table,gen_table_column  --path src/entity --output src/entity --with-serde both --database-url mysql://root:123456@localhost:3306/ruoyi

pub fn register_routes(cfg: &mut web::ServiceConfig) {
}