# CLAUDE.md

本文件为 Claude Code (claude.ai/code) 在此代码库中工作时提供指导。

## 项目概述

这是 **jav-tidy-rs**，一个用于整理成人影片文件的 Rust 应用程序。它能够自动：
- 监控输入目录中的新视频文件
- 使用模式匹配从文件名中提取影片ID
- 使用可配置模板从成人影片数据库爬取元数据
- 生成包含电影信息的 NFO 文件
- 将文件整理到输出目录并正确命名

## 开发命令

### 构建和运行
```bash
# 构建项目
cargo build

# 发布版本构建
cargo build --release

# 运行应用程序
cargo run -- --config config.toml --log ./log --template ./template

# 使用自定义参数运行
cargo run -- -c config.toml -l ./log -t ./template
```

### 测试
```bash
# 运行所有测试
cargo test

# 运行特定包的测试
cargo test -p crawler_template

# 运行特定测试
cargo test test_name
```

### 代码质量
```bash
# 格式化代码
cargo fmt

# 运行代码检查
cargo clippy

# 运行所有目标的代码检查
cargo clippy --all-targets
```

## 架构概览

### 核心组件

1. **主应用程序 (`src/main.rs`)**
   - 使用异步 tokio 运行时的入口点
   - 初始化日志、配置、文件监控和爬虫
   - 使用 mpsc 通道进行文件处理管道

2. **文件监控 (`src/file/`)**
   - `notify.rs`: 监控输入目录中的新视频文件
   - `mod.rs`: 处理文件事件和按扩展名过滤

3. **配置 (`src/config.rs`)**
   - 从 TOML 文件和环境变量（JAVTIDY_ 前缀）加载
   - 定义支持的文件扩展名、目录、模板优先级
   - 处理文件名清理模式

4. **爬虫系统 (`src/crawler.rs`)**
   - 协调文件处理管道
   - 管理模板加载和优先级
   - 使用进度条处理并发

5. **模板引擎 (`crawler_template/`)**
   - 基于 YAML 的网页抓取模板
   - 支持 CSS 选择器和 XPath 表达式
   - 使用过程宏进行模板编译
   - 多步骤工作流，支持搜索 → 详情页爬取

6. **文件整理 (`src/file_organizer.rs`)**
   - 移动和重命名处理过的文件
   - 创建输出目录结构
   - 处理字幕文件迁移

7. **NFO 生成 (`src/nfo_generator.rs`, `src/nfo.rs`)**
   - 创建 Kodi 兼容的 NFO 文件
   - 从爬取的数据构建电影元数据

8. **文件名解析 (`src/parser.rs`)**
   - 从各种文件名模式中提取影片ID
   - 应用配置中的清理规则

### 工作空间结构

项目使用 Cargo 工作空间，包含两个主要包：
- **根包**: 主应用程序逻辑
- **`crawler_template`**: 带有派生宏的模板引擎

### 配置系统

应用程序使用分层配置系统：
1. 默认 TOML 文件 (`config.toml`)
2. 带有 `JAVTIDY_` 前缀的环境变量
3. 命令行参数覆盖配置文件设置

### 模板系统

模板是定义网页抓取工作流的 YAML 文件，位于根目录的 `template/` 文件夹中：
- **入口点**: 带有参数替换的起始 URL
- **节点**: 使用自定义脚本语言进行数据提取
- **工作流**: 多步骤过程（搜索 → 详情页）
- **参数**: 在步骤间传递的运行时变量

#### 模板位置
- **生产模板**: `template/javdb.yaml` - 主要的 JavDB 爬取模板
- **测试模板**: `test_html/` - 用于测试的 HTML 文件
- **库内部模板**: `crawler_template/template/` - 仅供库内部测试使用

#### 脚本语言语法

模板中的 `script` 字段使用基于 pest 解析器的自定义脚本语言，支持链式调用：

**选择器规则 (Selector Rules)**
- `selector("css_selector")` - CSS 选择器
- `parent(n)` - 向上查找 n 层父元素（默认 1）
- `prev(n)` - 向前查找 n 个兄弟元素（默认 1）
- `nth(n)` - 向后查找 n 个兄弟元素（默认 1）

**访问器规则 (Accessor Rules)**
- `html()` - 获取元素的 HTML 内容
- `attr("attribute_name")` - 获取元素的指定属性值
- `val()` - 获取元素的文本内容

**转换规则 (Transform Rules)**
- `replace("from", "to")` - 字符串替换
- `uppercase()` - 转为大写
- `lowercase()` - 转为小写
- `insert(index, "text")` - 在指定位置插入文本
- `prepend("text")` - 在开头添加文本
- `append("text")` - 在末尾添加文本
- `delete("text")` - 删除指定文本
- `regex_extract("pattern")` - 正则表达式提取
- `regex_replace("pattern", "replacement")` - 正则表达式替换
- `trim()` - 去除首尾空白字符
- `split("separator")` - 按分隔符分割字符串
- `substring(start, end)` - 提取子字符串 (end 可选)

**条件规则 (Condition Rules)**
- `equals("value")` - 等值比较过滤
- `regex_match("pattern")` - 正则表达式匹配过滤

**参数类型**
- **静态参数**: 使用引号的固定字符串，如 `"css_selector"`
- **动态参数**: 使用 `${variable_name}` 格式的运行时变量

**脚本类型**
- **element_access**: 返回 HTML 元素，用于进一步链式操作
- **value_access**: 返回字符串值，用于数据提取

模板结构示例：
```yaml
entrypoint: "${base_url}/search?q=${crawl_name}&f=all"
env:
  page: ["1"]
nodes:
  main:
    script: selector(".movie-list")
    children:
      match_div:
        script: selector(".video-title>strong").val().uppercase().equals(${crawl_name}).parent(2)
        children:
          name: selector(".video-title>strong").val()
          title: attr("title")
          thumbnail: selector("img").attr("src")
          detail_url:
            script: attr("href").insert(0,${base_url})
            request: true
            children:
              main_image: selector(".video-meta-panel>div>div.column.column-video-cover>a>img").attr("src")
              detail_title: selector(".video-detail .current-title").val()
              detail_imgs: selector(".tile-images.preview-images>.tile-item").attr("href")
```

## 开发提示

### 添加新模板
1. 在 `crawler_template/template/` 中创建 YAML 文件
2. 使用 `${parameter}` 占位符定义入口点 URL
3. 使用脚本语言构建节点结构，支持链式调用
4. 对生成新 HTTP 请求的节点使用 `request: true`
5. 在 `test_html/` 中使用示例 HTML 测试

### 脚本语言使用提示
1. **链式调用**: 脚本支持方法链式调用，如 `selector(".class").val().uppercase()`
2. **条件过滤**: 使用 `equals()` 或 `regex_match()` 进行元素过滤
3. **动态参数**: 使用 `${variable}` 引用运行时变量
4. **元素导航**: 使用 `parent()`, `prev()`, `nth()` 在 DOM 树中导航
5. **错误处理**: 脚本解析失败会在编译时报错，确保语法正确

### 测试模板
可以使用 crawler_template 测试套件测试模板：
```bash
cd crawler_template
cargo test
```

### 文件处理管道
1. 文件监控检测新文件
2. 文件名解析器提取电影ID
3. 按优先级顺序尝试模板
4. 成功爬取生成 NFO 数据
5. 文件整理器移动/重命名文件
6. 通过 indicatif 进度条跟踪进度

### 关键依赖
- **tokio**: 用于文件 I/O 和 HTTP 请求的异步运行时
- **notify**: 跨平台文件系统事件
- **scraper**: HTML 解析和 CSS 选择器引擎
- **reqwest**: 网页爬取的 HTTP 客户端
- **serde**: 配置和数据序列化
- **indicatif**: 进度条和日志集成