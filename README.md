# Netflu

Netflu 是一个简洁高效的编程语言和包管理工具，旨在提供类似 Rust 的开发体验，包含编译器和包管理器功能。


## 特性

- **一体化工具链**：集成编译器和包管理器
- **简洁语法**：类似 Rust 的简洁语法设计
- **快速编译**：高效的代码生成和优化
- **项目管理**：完整的项目创建、构建和运行流程
- **中文支持**：全中文帮助信息，便于理解和使用

## 安装

### 前提条件
- Rust 1.60+ 和 Cargo
- Git

### 从源码安装

```bash
# 克隆仓库
git clone https://github.com/JRZYgit/ntfp.git
cd ntfp

# 构建并安装
cargo install --path .

# 验证安装
ntfp -V
```

## 快速开始

### 创建新项目

```bash
ntfp new hello_world
cd hello_world
```

### 编写代码

编辑 `src/main.ntf` 文件：

```rust
fun main() {
    println("Hello, Netflu!");
}
```

### 构建并运行

```bash
# 构建项目
ntfp build

# 运行项目
ntfp run
```

## 命令详解

### `ntfp new <name>`
创建新的 Netflu 项目

```bash
# 创建默认项目
ntfp new my_project

# 指定模板创建
ntfp new my_project
```

### `ntfp build [path]`
编译项目但不运行

```bash
# 编译当前项目
ntfp build

# 编译指定路径项目
ntfp build --path ./my_project
```

### `ntfp run [path]`
构建并运行项目

```bash
# 运行当前项目
ntfp run

# 运行指定路径项目
ntfp run --path ./my_project
```

### `ntfp init [path]`
初始化现有目录为 Netflu 项目

```bash
# 初始化当前目录
ntfp init

# 初始化指定目录
ntfp init --path ./existing_directory
```

## 项目结构

典型的 Netflu 项目结构如下：

```
my_project/
├── ntfp.toml       # 项目配置文件
├── src/
│   └── main.ntf     # 主程序文件
├── target/          # 编译输出目录
└── .gitignore
```

## 编译器工作流程

1. **词法分析**：将源代码转换为标记流
2. **语法分析**：构建抽象语法树(AST)
3. **语义分析**：验证代码正确性
4. **代码生成**：将 AST 转换为 Rust 代码
5. **编译**：调用 rustc 编译生成可执行文件

## 贡献指南

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add some amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 打开 Pull Request
