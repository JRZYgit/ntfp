use clap::CommandFactory;
use clap::{Parser, Subcommand};
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
    process::Command,
};
use anyhow::{Context, Result};
mod compile;

#[derive(Parser)]
#[command(
    author, 
    version, 
    about = "Netflu包管理器和编译器",
    long_about = "Netflu - 一个简洁的编程语言

使用方法:
  ntfp new <name>      创建新项目
  ntfp build [path]    编译项目
  ntfp run [path]      构建并运行项目
  ntfp init [path]     初始化现有目录为Netflu项目

示例:
  ntfp new hello_world
  ntfp build
  ntfp run"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// 创建新的Netflu项目
    #[command(about = "创建新的Netflu项目", long_about = "创建一个新的Netflu项目目录并初始化基本文件结构

参数:
  <name>      项目名称
  --template  使用的模板名称 (默认: 'default')

示例:
  ntfp new hello_world
  ntfp new my_project --template default")]
    New {
        /// 项目名称
        name: String,
        /// 使用的模板名称
        #[arg(short, long, default_value = "default")]
        template: String,
    },
    
    /// 构建并运行项目
    #[command(about = "构建并运行项目", long_about = "编译项目并运行生成的可执行文件

参数:
  --path  项目路径 (默认: 当前目录)

示例:
  ntfp run
  ntfp run --path ./my_project")]
    Run {
        #[arg(short, long, default_value = ".")]
        path: String,
    },
    
    /// 初始化现有目录为Netflu项目
    #[command(about = "初始化现有目录为Netflu项目", long_about = "将现有目录初始化为Netflu项目，创建必要的配置文件

参数:
  --path  要初始化的目录路径 (默认: 当前目录)

示例:
  ntfp init
  ntfp init --path ./existing_project")]
    Init {
        #[arg(short, long, default_value = ".")]
        path: String,
    },
    
    /// 编译项目但不运行
    #[command(about = "编译项目但不运行", long_about = "将.ntf源文件编译为Rust代码并生成可执行文件

参数:
  --path  项目路径 (默认: 当前目录)

示例:
  ntfp build
  ntfp build --path ./my_project")]
    Build {
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
                    "fun main() {{\n    print(\"Welcome to Netflu!\");\n}}\n"
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
        anyhow::bail!("不是有效的`Netflu`项目: 未找到`ntfp.toml`。");
    }

    build_project(path)?;

    let binary_path = project_path.join("target").join("debug").join("main.exe");
    if !binary_path.exists() {
        anyhow::bail!("未找到编译后的二进制文件，请先运行`ntfp build`。");
    }

    println!("正在运行项目: {}", path);
    let output = Command::new(binary_path)
        .current_dir(project_path)
        .output()
        .with_context(|| "无法运行项目")?;

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

fn build_project(path: &str) -> Result<()> {
    let project_path = Path::new(path);

    if !project_path.exists() {
        anyhow::bail!("项目路径不存在: {}", path);
    }

    let ntfp_toml = project_path.join("ntfp.toml");
    if !ntfp_toml.exists() {
        anyhow::bail!("不是有效的`Netflu`项目: 未找到`ntfp.toml`");
    }

    let src_dir = project_path.join("src");
    let main_ntf = src_dir.join("main.ntf");
    if !main_ntf.exists() {
        anyhow::bail!("未找到主程序文件: `src/main.ntf`");
    }

    println!("正在构建项目: {}", path);

    let ntf_content = fs::read_to_string(&main_ntf)
        .with_context(|| format!("无法读取文件: {:?}", main_ntf))?;

    let tokens = compile::lexer(&ntf_content)
        .map_err(|e| anyhow::anyhow!("词法分析错误: {}", e))?;
    
    let mut parser = compile::Parser::new(tokens);
    let mut ast = parser.parse()
        .map_err(|e| anyhow::anyhow!("语法分析错误: {}", e))?;

    let mut analyzer = compile::SemanticAnalyzer::new();
    analyzer.analyze(&mut ast)
        .map_err(|e| anyhow::anyhow!("语义分析错误: {}", e))?;

    let generated_code = compile::generate_code(&ast)
        .map_err(|e| anyhow::anyhow!("代码生成错误: {}", e))?;

    let target_dir = project_path.join("target").join("debug");
    fs::create_dir_all(&target_dir)
        .with_context(|| format!("无法创建目录: {:?}", target_dir))?;

    let main_rs_path = target_dir.join("main.rs");
    fs::write(&main_rs_path, &generated_code)
        .with_context(|| format!("无法写入文件: {:?}", main_rs_path))?;

    let binary_path = target_dir.join("main.exe");
    let compile_output = Command::new("rustc")
        .arg(&main_rs_path)
        .arg("-o")
        .arg(&binary_path)
        .current_dir(project_path)
        .output()
        .with_context(|| "调用rustc编译失败")?;

    if !compile_output.status.success() {
        let err_msg = String::from_utf8_lossy(&compile_output.stderr);
        anyhow::bail!("编译失败: {}", err_msg);
    }

    println!("构建成功! 二进制文件: {:?}", binary_path);
    Ok(())
}

fn init_project(path: &str) -> Result<()> {
    let project_path = Path::new(path);

    if !project_path.exists() {
        anyhow::bail!("目录不存在: {}", path);
    }

    let cargo_toml = project_path.join("ntfp.toml");
    if cargo_toml.exists() {
        anyhow::bail!("目录已经是一个Netflu项目: 已存在ntfp.toml");
    }

    let project_name = project_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("my_project");

    let template = default_template(project_name);

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
        Some(Commands::Build { path }) => build_project(&path),
        None => {
            Cli::command().print_help()?;
            Ok(())
        }
    }
}