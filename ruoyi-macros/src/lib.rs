//! 若依Rust版本的宏定义模块

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, AttributeArgs, FnArg, ItemFn, Lit, Meta, MetaNameValue, NestedMeta, PatType,
};

/// 操作日志宏，用于标记控制器方法
///
/// # 示例
///
/// ```rust
/// #[oper_log(title = "用户管理", business_type = "Insert", oper_type = "Web")]
/// async fn add_user(req: HttpRequest, user: web::Json<AddUserReq>) -> Result<impl Responder> {
///     // 方法体
/// }
/// ```
#[proc_macro_attribute]
pub fn oper_log(args: TokenStream, input: TokenStream) -> TokenStream {
    // 解析宏参数
    let args = parse_macro_input!(args as AttributeArgs);
    let mut title = String::from("未知功能");
    let mut business_type = String::from("Other"); // 默认为其他
    let mut oper_type = String::from("Web"); // 默认为后台用户

    // 解析宏参数
    for arg in args {
        if let NestedMeta::Meta(Meta::NameValue(MetaNameValue { path, lit, .. })) = arg {
            let ident = path.get_ident();
            if ident.is_none() {
                continue;
            }

            let ident = ident.unwrap().to_string();
            match ident.as_str() {
                "title" => {
                    if let Lit::Str(lit_str) = lit {
                        title = lit_str.value();
                    }
                }
                "business_type" => {
                    if let Lit::Str(lit_str) = lit {
                        business_type = lit_str.value();
                    }
                }
                "oper_type" => {
                    if let Lit::Str(lit_str) = lit {
                        oper_type = lit_str.value();
                    }
                }
                _ => {}
            }
        }
    }

    // 解析函数
    let input_fn = parse_macro_input!(input as ItemFn);
    // 获取函数可见性
    let fn_vis = &input_fn.vis;
    // 获取函数签名
    let fn_sig = &input_fn.sig;
    // 获取函数体
    let fn_block = &input_fn.block;
    // 获取函数名
    let fn_name = &fn_sig.ident;
    // 获取函数参数
    let fn_inputs = &fn_sig.inputs;

    // 查找HttpRequest参数
    let req_var_name = find_http_request_param(fn_inputs);

    // 构建业务类型标识符
    let business_type_ident = format_ident!("{}", business_type);
    // 构建操作者类型标识符
    let oper_type_ident = format_ident!("{}", oper_type);

    // 构建输出TokenStream
    let output = quote! {
        #fn_vis #fn_sig {
            let title = #title;

            // 导入所需类型和工具
            use ruoyi_common::enums::{OperLogBusinessType, OperLogOperatorType};
            use ruoyi_common::utils::{ip, http};
            use num_enum::IntoPrimitive;
            use actix_web::HttpRequest;
            use chrono::Utc;
            use ruoyi_framework::logger::entity::OperLogModel;
            use ruoyi_framework::web::tls::get_sync_user_context;

            // 记录操作日志开始
            let start_time = std::time::Instant::now();
            // 提前准备操作日志
            #req_var_name
            let mut oper_log = OperLogModel {
                oper_id: 0,
                title: Some(title.to_string()),
                business_type: Some(OperLogBusinessType::#business_type_ident.into()),
                method: Some(stringify!(#fn_name).to_string()),
                request_method: None,
                operator_type: Some(OperLogOperatorType::#oper_type_ident.into()),
                oper_name: None,
                dept_name: None,
                oper_url: None,
                oper_ip: None,
                oper_location: None,
                oper_param: None,
                json_result: None,
                status: Some(0), // 默认为成功
                error_msg: None,
                oper_time: Some(Utc::now()),
                cost_time: None,
            };

            // 从请求中获取更多信息填充日志
            if let Some(req) = __http_request {
                // 请求方法
                oper_log.request_method = Some(req.method().to_string());

                // 请求URL
                oper_log.oper_url = Some(req.uri().to_string());

                // 获取客户端IP
                let client_ip = ip::get_real_ip_by_request(req);
                oper_log.oper_ip = Some(client_ip.clone());

                // 获取IP位置
                oper_log.oper_location = Some(ip::get_ip_location(&client_ip));

                // 获取请求参数
                oper_log.oper_param = http::get_request_params(req);

                // 获取用户信息(如果有的话)
                if let Some(user_context) = get_sync_user_context() {
                    oper_log.oper_name = Some(user_context.user_name.clone());
                }
            }
            // 执行原函数体
            let result = #fn_block;
            result
        }
    };

    output.into()
}

/// 查找函数参数中的HttpRequest类型，并返回获取它的代码
fn find_http_request_param(
    inputs: &syn::punctuated::Punctuated<FnArg, syn::token::Comma>,
) -> proc_macro2::TokenStream {
    // 寻找HttpRequest类型的参数
    for input in inputs {
        if let FnArg::Typed(PatType { ty, pat, .. }) = input {
            if let syn::Pat::Ident(pat_ident) = &**pat {
                let param_name = &pat_ident.ident;

                // 检查参数类型
                if let syn::Type::Path(type_path) = &**ty {
                    if let Some(segment) = type_path.path.segments.last() {
                        if segment.ident == "HttpRequest" {
                            // 找到了HttpRequest参数
                            return quote! {
                                let __http_request = Some(&#param_name);
                            };
                        }
                    }
                }
            }
        }
    }

    // 如果没有找到HttpRequest参数
    quote! {
        let __http_request: Option<&HttpRequest> = None;
    }
}