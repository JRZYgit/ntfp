use clap::CommandFactory;
use clap::{Parser, Subcommand};
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
    process::Command,
};
use anyhow::{Context, Result};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    New {
        name: String,
        #[arg(short, long, default_value = "default")]
        template: String,
    },
    Run {
        #[arg(short, long, default_value = ".")]
        path: String,
    },
    Init {
        #[arg(short, long, default_value = ".")]
        path: String,
    },
}

struct ProjectTemplate {
    name: String,
    files: Vec<(String, String)>,
}

fn default_template(project_name: &str) -> ProjectTemplate {
    ProjectTemplate {
        name: "default".to_string(),
        files: vec![
            (
                "ntfp.toml".to_string(),
                format!(
                    "[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2025\"\n\n[dependencies]\n", project_name
                ),
            ),
            (
                "src/main.ntf".to_string(),
                format!(
                    "fn main() {{\n    println(\"Hello from {}!\");\n}}\n", project_name
                ),
            ),
            (
                ".gitignore".to_string(),
                "target/\nntfp.lock\n".to_string(),
            ),
        ],
    }
}

fn create_project(name: &str, template_name: &str) -> Result<()> {
    let template = match template_name {
        "default" => default_template(name),
        _ => anyhow::bail!("未知模板: {}", template_name),
    };

    let project_dir = Path::new(name);
    fs::create_dir_all(project_dir)
        .with_context(|| format!("无法创建项目目录: {:?}", project_dir))?;

    // 写入模板文件
    for (file_path, content) in template.files {
        let full_path = project_dir.join(file_path);
        let parent = full_path.parent().unwrap();
        fs::create_dir_all(parent)
            .with_context(|| format!("无法创建目录: {:?}", parent))?;

        let mut file = File::create(&full_path)
            .with_context(|| format!("无法创建文件: {:?}", full_path))?;

        file.write_all(content.as_bytes())
            .with_context(|| format!("无法写入文件: {:?}", full_path))?;
    }

    println!("项目 '{}' 已创建成功! 使用模板: {}", name, template.name);
    Ok(())
}

fn run_project(path: &str) -> Result<()> {
    let project_path = Path::new(path);

    if !project_path.exists() {
        anyhow::bail!("项目路径不存在: {}", path);
    }

    let cargo_toml = project_path.join("ntfp.toml");
    if !cargo_toml.exists() {
        anyhow::bail!("不是有效的Netflu项目: 未找到ntfp.toml");
    }

    println!("正在运行项目: {}", path);
    let output = Command::new("ntfp")
        .arg("run")
        .current_dir(project_path)
        .output()
        .with_context(|| "无法运行ntfp run")?;

    // 打印输出
    if output.status.success() {
        println!("项目运行成功!");
        println!("标准输出:\n{}", String::from_utf8_lossy(&output.stdout));
    } else {
        anyhow::bail!(
            "项目运行失败!\n错误输出:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(())
}

// 初始化现有项目
fn init_project(path: &str) -> Result<()> {
    let project_path = Path::new(path);

    // 检查目录是否存在
    if !project_path.exists() {
        anyhow::bail!("目录不存在: {}", path);
    }

    // 检查是否已经有Cargo.toml
    let cargo_toml = project_path.join("ntfp.toml");
    if cargo_toml.exists() {
        anyhow::bail!("目录已经是一个Netflu项目: 已存在ntfp.toml");
    }

    // 获取项目名称 (使用目录名)
    let project_name = project_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("my_project");

    // 创建默认模板文件
    let template = default_template(project_name);

    // 写入模板文件
    for (file_path, content) in template.files {
        let full_path = project_path.join(file_path);
        let parent = full_path.parent().unwrap();
        fs::create_dir_all(parent)
            .with_context(|| format!("无法创建目录: {:?}", parent))?;

        let mut file = File::create(&full_path)
            .with_context(|| format!("无法创建文件: {:?}", full_path))?;

        file.write_all(content.as_bytes())
            .with_context(|| format!("无法写入文件: {:?}", full_path))?;
    }

    println!("项目已初始化成功! 项目名称: {}", project_name);
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::New { name, template }) => create_project(&name, &template),
        Some(Commands::Run { path }) => run_project(&path),
        Some(Commands::Init { path }) => init_project(&path),
        None => {
            Cli::command().print_help()?;
            Ok(())
        }
    }
}