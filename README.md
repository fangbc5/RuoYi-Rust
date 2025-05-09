# RuoYi-Rust

基于 Rust 实现的若依管理系统。

## 简介

RuoYi-Rust 是使用 Rust 语言重新实现的若依管理系统，它保持了原版若依管理系统的功能特性和设计理念，同时利用 Rust 语言高性能、高可靠性和内存安全的特点，提供更高效、更安全的后台管理系统。

## 技术选型

### 后端技术

* Rust 语言: 高性能、内存安全的系统编程语言
* Actix Web: 高性能 Web 框架
* Sea-ORM: Rust 异步 ORM 框架
* Redis: 缓存数据库
* JWT: 用于身份验证的 JSON Web Token
* Argon2: 密码哈希算法

### 前端技术

* Vue: 渐进式 JavaScript 框架
* Element UI: 基于 Vue.js 2.0 的桌面端组件库
* Axios: 基于 promise 的 HTTP 客户端

## 项目模块

项目采用 Rust 工作空间 (workspace) 方式组织，包含以下模块：

* **ruoyi-common**: 通用工具模块，包含常量定义、错误处理、工具类等
* **ruoyi-framework**: 框架模块，包含配置管理、数据库连接、中间件等
* **ruoyi-system**: 系统模块，实现核心业务逻辑
* **ruoyi-admin**: 管理模块，提供 Web 接口

## 安装部署

### 环境要求

* Rust 1.70+
* MySQL 5.7+
* Redis 5.0+

### 开发环境搭建

1. 克隆代码

```bash
git clone https://github.com/your-username/ruoyi-rust.git
cd ruoyi-rust
```

2. 修改配置

编辑 `.env` 文件，可以调整日志级别、设置配置文件路径、切换环境
编辑 `config/development.toml` 文件，配置数据库连接等信息：

```
# 数据库配置
[database]
url = "mysql://root:123456@localhost:3306/ruoyi"

# 缓存配置,支持多级缓存
[cache]
# 默认开启 = true
enabled = true
# 默认类型 = local（local、redis、multi）
cache_type = "redis"

[cache.redis]
# 连接类型 (standalone、cluster)
connection_type = "standalone"
url = "redis://localhost:6379" 
password = "123456"
```

3. 编译运行

```bash
cargo run
```

系统将在 http://localhost:8080 启动。

## 功能特性

RuoYi-Rust 包含以下主要功能：

* 用户管理：用户是系统操作者，该功能主要完成系统用户配置
* 部门管理：配置系统组织机构（公司、部门、小组）
* 角色管理：角色菜单权限分配
* 菜单管理：配置系统菜单，操作权限，按钮权限标识等
* 字典管理：对系统中经常使用的一些固定数据进行维护
* 参数管理：对系统动态配置常用参数
* 通知公告：系统通知公告信息发布维护
* 操作日志：系统正常操作日志记录和查询
* 登录日志：系统登录日志记录和查询
* 在线用户：当前系统中活跃用户状态监控
* 定时任务：在线（添加、修改、删除）任务调度包含执行结果日志
* 代码生成：前后端代码的生成
* 系统接口：根据业务代码自动生成相关的API接口文档

## 许可证

[MIT Licensed](LICENSE) 