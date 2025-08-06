# Crawler模块工作流程文档

## 概述

本文档详细描述了jav-tidy-rs项目中crawler模块的完整工作流程，包括文件安全保护机制、处理阶段和错误处理策略。

## 架构概览

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   File Monitor  │───▶│  File Queue     │───▶│  Crawler        │
│   (notify.rs)   │    │  (mpsc channel) │    │  (crawler.rs)   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                                        │
                                                        ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│ File Organizer  │◀───│ Transaction     │◀───│ File Protection │
│ (file_organizer)│    │ Manager         │    │ (locks & checks)│
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## 核心组件

### 1. 文件保护系统

#### FileProcessingLock
- **功能**: 防止同一文件被多个进程同时处理
- **机制**: 创建 `.javtidy.lock` 文件，包含进程ID和时间戳
- **自动清理**: 超时锁文件（5分钟）自动清理
- **格式**:
  ```
  <进程ID>
  <创建时间戳>
  <文件路径>
  ```

#### FileIntegrityChecker
- **功能**: 检测文件在处理过程中是否被外部修改
- **检查项**: 文件大小、修改时间
- **检查时机**: 处理的关键节点进行多次验证

#### FileProcessingTransaction
- **功能**: 确保所有文件操作的原子性
- **支持操作**:
  - 创建NFO文件
  - 移动视频文件
  - 创建目录
- **回滚机制**: 失败时自动记录，便于手动清理

### 2. 处理阶段详解

#### 阶段1: 文件保护和预检查
```rust
// 获取文件处理锁
progress_bar.set_message("获取文件锁...");
let _lock = FileProcessingLock::acquire(file_path)?;

// 创建完整性检查器
let integrity_checker = FileIntegrityChecker::new(file_path)?;

// 验证文件存在性
if !file_path.exists() {
    return Err(anyhow::anyhow!("文件不存在"));
}
```

**目标**: 确保文件在处理期间不会被其他进程干扰

#### 阶段2: 文件名解析
```rust
progress_bar.set_message("解析文件名...");
let movie_id = parser.extract_movie_id(file_path, config)?;

// 第一次完整性检查
if !integrity_checker.verify_integrity()? {
    return Err(anyhow::anyhow!("文件在处理过程中被修改"));
}
```

**目标**: 提取影片ID，为后续爬取做准备

#### 阶段3: 数据爬取
```rust
progress_bar.set_message(format!("搜索影片信息: {}", movie_id));
let crawler_data = crawler(&movie_id, progress_bar, templates, config).await?;

// 第二次完整性检查
if !integrity_checker.verify_integrity()? {
    return Err(anyhow::anyhow!("文件在爬取过程中被修改"));
}
```

**目标**: 从各个模板源获取影片信息

#### 阶段4: NFO验证和生成
```rust
progress_bar.set_message("验证NFO数据...");
let movie_nfo = MovieNfo::for_universal(crawler_data);
let warnings = nfo_generator.validate_nfo(&movie_nfo);
```

**目标**: 生成标准化的NFO数据

#### 阶段5: 事务准备
```rust
progress_bar.set_message("准备文件操作...");
let mut transaction = FileProcessingTransaction::new(file_path);

// 添加NFO创建操作
transaction.add_nfo_creation(nfo_path, nfo_content);

// 添加文件移动操作（如需要）
if file_organizer.needs_organization(file_path, config) {
    let new_path = file_organizer.generate_new_file_path(file_path, &movie_nfo, config)?;
    transaction.add_file_move(file_path.to_path_buf(), new_path);
}
```

**目标**: 准备所有需要执行的文件操作

#### 阶段6: 事务执行
```rust
progress_bar.set_message("执行文件操作...");
transaction.commit()?;
```

**目标**: 原子性执行所有文件操作

#### 阶段7: 完成处理
```rust
progress_bar.set_message("处理完成");
log::info!("影片 {} 处理完成", movie_id);
```

**目标**: 记录处理结果，释放资源

## 错误处理策略

### 1. 预防性错误处理
- **文件锁冲突**: 检测并清理僵尸锁
- **文件不存在**: 在开始处理前验证
- **权限问题**: 在执行操作前检查目录权限

### 2. 运行时错误处理
- **网络错误**: 重试机制和超时控制
- **文件系统错误**: 详细错误日志和回滚
- **数据验证错误**: 警告记录但继续处理

### 3. 恢复机制
- **事务回滚**: 失败时的状态恢复
- **资源清理**: 锁文件和临时文件的自动清理
- **状态记录**: 详细的处理日志便于故障排查

## 配置参数

### 核心配置
```toml
[processing]
# 处理超时时间（秒）
timeout = 300

# 最大并发处理文件数
max_concurrent = 4

# 文件锁超时时间（秒）
lock_timeout = 300

[file_safety]
# 启用文件锁定
enable_locking = true

# 启用完整性检查
enable_integrity_check = true

# 检查间隔（毫秒）
check_interval = 1000

[retry]
# 最大重试次数
max_retries = 3

# 重试间隔（秒）
retry_interval = 5
```

## 性能监控

### 关键指标
1. **处理成功率**: 成功处理的文件比例
2. **平均处理时间**: 每个文件的处理耗时
3. **锁冲突率**: 文件锁冲突的频率
4. **完整性检查失败率**: 文件被外部修改的频率

### 监控实现
```rust
// 处理开始时间
let start_time = std::time::Instant::now();

// 处理结束时记录
let duration = start_time.elapsed();
log::info!("文件处理耗时: {:?}", duration);

// 记录到性能指标
PROCESSING_METRICS.record_processing_time(duration);
```

## 最佳实践

### 1. 部署建议
- **独立运行**: 避免多个实例同时监控同一目录
- **定期清理**: 定期清理过期的锁文件
- **监控空间**: 确保足够的磁盘空间用于文件移动

### 2. 错误处理
- **日志级别**: 合理设置日志级别，平衡信息量和性能
- **告警机制**: 设置关键错误的告警通知
- **重试策略**: 对临时性错误实施适当的重试

### 3. 性能优化
- **并发控制**: 根据系统资源调整并发数
- **缓存策略**: 缓存模板和配置数据
- **批量处理**: 对小文件实施批量处理

## 故障排除

### 常见问题

#### 1. 文件锁冲突
**症状**: 错误消息"文件正在被其他进程处理"
**原因**: 
- 多个jav-tidy实例运行
- 进程异常退出导致锁文件残留
**解决**:
```bash
# 查找并清理锁文件
find /path/to/files -name "*.javtidy.lock" -mtime +1 -delete
```

#### 2. 文件完整性检查失败
**症状**: 错误消息"文件在处理过程中被修改"
**原因**: 
- 文件被其他程序访问
- 网络存储的同步延迟
**解决**:
- 暂停其他可能访问文件的程序
- 调整完整性检查的严格程度

#### 3. 事务提交失败
**症状**: 错误消息"文件处理事务失败"
**原因**: 
- 磁盘空间不足
- 目标目录权限问题
**解决**:
- 检查磁盘空间
- 验证目录权限

### 调试工具

#### 1. 日志分析
```bash
# 查看处理日志
tail -f jav-tidy.log | grep "处理文件"

# 统计处理成功率
grep "处理完成" jav-tidy.log | wc -l
```

#### 2. 锁文件检查
```bash
# 检查当前锁文件
find /path/to/files -name "*.javtidy.lock" -exec cat {} \;
```

#### 3. 性能分析
```bash
# 分析处理耗时
grep "处理耗时" jav-tidy.log | awk '{print $NF}' | sort -n
```

## 总结

通过实施完整的文件保护机制和事务性操作，crawler模块能够安全、可靠地处理视频文件的整理工作。多阶段的完整性检查和详细的错误处理确保了系统的稳定性，而事务机制则保证了操作的原子性。

这个设计在保证文件安全的同时，也提供了良好的性能和用户体验，是一个在生产环境中可以放心使用的解决方案。