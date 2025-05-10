use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=config");

    // 获取输出目录
    let out_dir = env::var("OUT_DIR").unwrap();
    let profile = env::var("RUST_ENV").unwrap();

    // 只在 release 模式下执行
    if profile == "release" {
        // 获取目标目录
        let target_dir = Path::new(&out_dir)
            .ancestors()
            .find(|p| p.ends_with("target"))
            .unwrap_or_else(|| Path::new(&out_dir));

        let release_dir = target_dir.join("release");

        // 创建配置目录
        let config_dir = release_dir.join("config");
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir).unwrap_or_else(|e| {
                println!("cargo:warning=无法创建配置目录: {}", e);
            });
        }

        // 复制配置文件
        let src_config_dir = Path::new("config");
        if src_config_dir.exists() && src_config_dir.is_dir() {
            copy_dir_contents(src_config_dir, &config_dir).unwrap_or_else(|e| {
                println!("cargo:warning=无法复制配置文件: {}", e);
            });
            println!("cargo:warning=配置文件已复制到: {}", config_dir.display());
        } else {
            println!(
                "cargo:warning=源配置目录不存在: {}",
                src_config_dir.display()
            );
        }
    }
}

// 复制目录内容的辅助函数
fn copy_dir_contents(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_contents(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
            println!(
                "cargo:warning=已复制: {} -> {}",
                src_path.display(),
                dst_path.display()
            );
        }
    }

    Ok(())
}
