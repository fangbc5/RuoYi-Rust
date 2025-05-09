use std::sync::Arc;

use ruoyi_framework::config::AppConfig;
use sysinfo::{CpuExt, DiskExt, PidExt, ProcessExt, System, SystemExt};

use actix_web::web;
use actix_web::{get, HttpResponse, Responder};
use ruoyi_common::vo::RData;
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CpuInfo {
    // 核心数
    pub cpu_num: i32,
    // CPU总的使用率
    pub total: f64,
    // CPU系统使用率
    pub sys: f32,
    // CPU用户使用率
    pub used: f32,
    // CPU当前等待率
    pub wait: f32,
    // CPU当前空闲率
    pub free: f32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryInfo {
    // 内存总量
    pub total: String,
    // 已用内存
    pub used: String,
    // 剩余内存
    pub free: String,
    // 使用率
    pub usage: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    // 服务器名称
    pub computer_name: String,

    // 服务器Ip
    pub computer_ip: String,

    // 项目路径
    pub user_dir: String,

    // 操作系统
    pub os_name: String,

    // 操作系统架构
    pub os_arch: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessInfo {
    // 进程名称
    pub name: String,
    // 进程ID
    pub pid: u32,
    // 进程端口
    pub port: u16,
    // 进程CPU使用率
    pub cpu_usage: f32,
    // 进程内存
    pub mem_usage: String,
    // 启动时间
    pub start_time: String,
    // 运行时长
    pub run_time: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]

pub struct DiskInfo {
    // 盘符路径
    pub dir_name: String,

    // 盘符类型
    pub sys_type_name: String,

    // 文件类型
    pub type_name: String,

    // 总大小
    pub total: String,

    // 剩余大小
    pub free: String,

    // 已经使用量
    pub used: String,

    // 资源的使用率
    pub usage: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerInfo {
    // CPU相关信息
    pub cpu: CpuInfo,

    // 內存相关信息
    pub mem: MemoryInfo,

    // 服务器相关信息
    pub sys: SystemInfo,

    // 磁盘相关信息
    pub sys_files: Vec<DiskInfo>,

    // 进程相关信息
    pub process: ProcessInfo,
}

#[get("")]
pub async fn get_server_info(config: web::Data<Arc<AppConfig>>) -> impl Responder {
    // 获取端口号
    let port = config.server.port;

    // 初始化系统信息
    let mut sys = System::new_all();

    // 获取CPU信息
    sys.refresh_cpu();
    let cpu_info = CpuInfo {
        cpu_num: sys.cpus().len() as i32,
        total: 100.0,
        sys: (sys.global_cpu_info().cpu_usage() * 100.0).round() / 100.0,
        used: (sys.global_cpu_info().cpu_usage() * 100.0).round() / 100.0,
        wait: 0.0,
        free: ((100.0 - sys.global_cpu_info().cpu_usage()) * 100.0).round() / 100.0,
    };

    // 获取内存信息
    sys.refresh_memory();
    let total_memory = sys.total_memory();
    let used_memory = sys.used_memory();
    let free_memory = total_memory - used_memory;
    let usage = if total_memory > 0 {
        ((used_memory as f64 / total_memory as f64) * 100.0 * 100.0).round() / 100.0
    } else {
        0.0
    };

    let memory_info = MemoryInfo {
        total: format!("{:.2}", total_memory as f64 / 1024.0 / 1024.0 / 1024.0),
        used: format!("{:.2}", used_memory as f64 / 1024.0 / 1024.0 / 1024.0),
        free: format!("{:.2}", free_memory as f64 / 1024.0 / 1024.0 / 1024.0),
        usage,
    };

    // 获取系统信息
    let system_info = SystemInfo {
        computer_name: sys.host_name().unwrap_or_else(|| "未知".to_string()),
        computer_ip: "127.0.0.1".to_string(), // 本地IP地址
        user_dir: std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "未知".to_string()),
        os_name: sys.name().unwrap_or_else(|| "未知".to_string()),
        os_arch: std::env::consts::ARCH.to_string(),
    };

    // 获取磁盘信息
    sys.refresh_disks_list();
    let mut disk_info_list = Vec::new();
    for disk in sys.disks() {
        let total = disk.total_space();
        let free = disk.available_space();
        let used = total - free;
        let usage = if total > 0 {
            ((used as f64 / total as f64) * 100.0 * 100.0).round() / 100.0
        } else {
            0.0
        };

        let mount_point = disk.mount_point();
        let dir_name = mount_point.to_string_lossy().to_string();

        let disk_info = DiskInfo {
            dir_name,
            sys_type_name: format!("{:?}", disk.file_system()),
            type_name: "本地磁盘".to_string(),
            total: format!("{:.2} GB", total as f64 / 1024.0 / 1024.0 / 1024.0),
            free: format!("{:.2} GB", free as f64 / 1024.0 / 1024.0 / 1024.0),
            used: format!("{:.2} GB", used as f64 / 1024.0 / 1024.0 / 1024.0),
            usage,
        };

        disk_info_list.push(disk_info);
    }
    // 刷新进程信息
    sys.refresh_processes();
    // 获取进程信息
    let current_pid = std::process::id();

    // 确保系统已刷新进程信息
    let mut process_info = ProcessInfo {
        name: "".to_string(),
        pid: current_pid,
        port, // 获取端口需要额外的库支持，这里暂时设为0
        cpu_usage: 0.0,
        mem_usage: format!("{:.2} MB", 0.0),
        start_time: "".to_string(),
        run_time: "".to_string(),
    };

    if let Some(process) = sys.process(sysinfo::Pid::from_u32(current_pid)) {
        // 获取进程名称
        process_info.name = process.name().to_string();
        // 获取进程CPU使用率
        process_info.cpu_usage = process.cpu_usage();
        // 获取进程内存使用量
        process_info.mem_usage = format!("{:.2} MB", process.memory() as f64 / 1024.0 / 1024.0);

        // 格式化进程开始时间和运行时长
        let start_time_secs = process.start_time();
        let run_time_secs = process.run_time();
        let (start_time_str, run_time_str) = if start_time_secs > 0 && run_time_secs > 0 {
            let start_time = chrono::DateTime::<chrono::Local>::from(
                std::time::UNIX_EPOCH + std::time::Duration::from_secs(start_time_secs),
            );
            // 计算进程运行时长，将其格式化成小时、分钟、秒
            let run_time = std::time::Duration::from_secs(run_time_secs);
            let run_time_str = ruoyi_common::utils::time::format_duration(run_time);

            (
                start_time.format("%Y-%m-%d %H:%M:%S").to_string(),
                run_time_str,
            )
        } else {
            ("未知".to_string(), "未知".to_string())
        };

        process_info.start_time = start_time_str;
        process_info.run_time = run_time_str;
    }

    // 组装服务器信息
    let server_info = ServerInfo {
        cpu: cpu_info,
        mem: memory_info,
        sys: system_info,
        sys_files: disk_info_list,
        process: process_info,
    };

    HttpResponse::Ok().json(RData::<ServerInfo>::ok(server_info))
}

pub fn load_server_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/server").service(get_server_info));
}
