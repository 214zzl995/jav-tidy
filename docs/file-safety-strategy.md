# 文件安全策略文档

## 概述

在视频文件自动整理过程中，确保文件不会在爬取过程中被意外移动、删除或损坏是至关重要的。本文档详细说明了jav-tidy-rs项目的文件安全策略和实现方案。

## 文件安全风险分析

### 主要风险点

1. **并发访问冲突**
   - 文件在爬取过程中被其他程序移动或删除
   - 多个jav-tidy实例同时处理同一文件
   - 用户手动操作文件导致冲突

2. **系统资源竞争**
   - 磁盘空间不足导致移动失败
   - 文件被其他程序锁定无法访问
   - 网络存储的连接中断

3. **处理过程中断**
   - 程序异常退出时文件处于不一致状态
   - 爬取过程中网络中断
   - 用户强制终止程序

## 文件保护策略

### 1. 文件锁定机制

#### 独占文件锁
```rust
use std::fs::File;
use std::io::Read;

pub struct FileGuard {
    file: File,
    path: PathBuf,
}

impl FileGuard {
    /// 创建文件保护，获取独占锁
    pub fn new(path: &Path) -> anyhow::Result<Self> {
        let file = File::open(path)?;
        // 在Unix系统上使用flock，Windows上使用LockFile
        #[cfg(unix)]
        {
            use std::os::unix::io::AsRawFd;
            let fd = file.as_raw_fd();
            unsafe {
                if libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB) == -1 {
                    return Err(anyhow::anyhow!("文件被其他进程锁定"));
                }
            }
        }
        
        Ok(FileGuard {
            file,
            path: path.to_path_buf(),
        })
    }
}

impl Drop for FileGuard {
    fn drop(&mut self) {
        // 自动释放文件锁
        #[cfg(unix)]
        {
            use std::os::unix::io::AsRawFd;
            let fd = self.file.as_raw_fd();
            unsafe {
                libc::flock(fd, libc::LOCK_UN);
            }
        }
    }
}
```

#### 锁文件机制
```rust
pub struct ProcessLockFile {
    lock_path: PathBuf,
}

impl ProcessLockFile {
    pub fn acquire(file_path: &Path) -> anyhow::Result<Self> {
        let lock_path = file_path.with_extension("javtidy.lock");
        
        // 检查锁文件是否已存在
        if lock_path.exists() {
            // 验证进程是否仍在运行
            if let Ok(content) = std::fs::read_to_string(&lock_path) {
                if let Ok(pid) = content.trim().parse::<u32>() {
                    if is_process_running(pid) {
                        return Err(anyhow::anyhow!("文件正在被进程 {} 处理", pid));
                    }
                }
            }
            // 清理僵尸锁文件
            std::fs::remove_file(&lock_path)?;
        }
        
        // 创建新的锁文件
        std::fs::write(&lock_path, std::process::id().to_string())?;
        
        Ok(ProcessLockFile { lock_path })
    }
}

impl Drop for ProcessLockFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.lock_path);
    }
}
```

### 2. 文件状态检查

#### 文件完整性验证
```rust
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub struct FileIntegrityChecker {
    path: PathBuf,
    initial_hash: u64,
    initial_size: u64,
    initial_modified: SystemTime,
}

impl FileIntegrityChecker {
    pub fn new(path: &Path) -> anyhow::Result<Self> {
        let metadata = std::fs::metadata(path)?;
        let initial_size = metadata.len();
        let initial_modified = metadata.modified()?;
        let initial_hash = Self::calculate_file_hash(path)?;
        
        Ok(FileIntegrityChecker {
            path: path.to_path_buf(),
            initial_hash,
            initial_size,
            initial_modified,
        })
    }
    
    pub fn verify_integrity(&self) -> anyhow::Result<bool> {
        if !self.path.exists() {
            return Ok(false);
        }
        
        let metadata = std::fs::metadata(&self.path)?;
        let current_size = metadata.len();
        let current_modified = metadata.modified()?;
        
        // 快速检查：大小和修改时间
        if current_size != self.initial_size || current_modified != self.initial_modified {
            return Ok(false);
        }
        
        // 深度检查：文件哈希（仅在怀疑时使用）
        let current_hash = Self::calculate_file_hash(&self.path)?;
        Ok(current_hash == self.initial_hash)
    }
    
    fn calculate_file_hash(path: &Path) -> anyhow::Result<u64> {
        let mut file = File::open(path)?;
        let mut hasher = DefaultHasher::new();
        let mut buffer = [0; 8192];
        
        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            buffer[..bytes_read].hash(&mut hasher);
        }
        
        Ok(hasher.finish())
    }
}
```

### 3. 原子操作

#### 安全文件移动
```rust
pub fn safe_move_file(
    source: &Path, 
    destination: &Path,
    backup_enabled: bool
) -> anyhow::Result<()> {
    // 1. 创建目标目录
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    // 2. 备份目标文件（如果存在）
    let backup_path = if backup_enabled && destination.exists() {
        let backup_path = destination.with_extension(
            format!("{}.backup.{}", 
                destination.extension().unwrap_or_default().to_str().unwrap_or(""),
                chrono::Utc::now().timestamp()
            )
        );
        std::fs::rename(destination, &backup_path)?;
        Some(backup_path)
    } else {
        None
    };
    
    // 3. 执行原子移动
    match std::fs::rename(source, destination) {
        Ok(_) => {
            // 移动成功，清理备份文件
            if let Some(backup_path) = backup_path {
                let _ = std::fs::remove_file(backup_path);
            }
            Ok(())
        }
        Err(e) => {
            // 移动失败，恢复备份
            if let Some(backup_path) = backup_path {
                let _ = std::fs::rename(backup_path, destination);
            }
            Err(e.into())
        }
    }
}
```

### 4. 事务性操作

#### 文件处理事务
```rust
pub struct FileProcessingTransaction {
    original_path: PathBuf,
    temp_dir: TempDir,
    operations: Vec<TransactionOperation>,
    completed: bool,
}

#[derive(Debug)]
enum TransactionOperation {
    CreateNfo { path: PathBuf, content: String },
    MoveFile { from: PathBuf, to: PathBuf },
    CreateDirectory { path: PathBuf },
}

impl FileProcessingTransaction {
    pub fn new(original_path: &Path) -> anyhow::Result<Self> {
        let temp_dir = TempDir::new()?;
        
        Ok(FileProcessingTransaction {
            original_path: original_path.to_path_buf(),
            temp_dir,
            operations: Vec::new(),
            completed: false,
        })
    }
    
    pub fn add_nfo_creation(&mut self, path: PathBuf, content: String) {
        self.operations.push(TransactionOperation::CreateNfo { path, content });
    }
    
    pub fn add_file_move(&mut self, from: PathBuf, to: PathBuf) {
        self.operations.push(TransactionOperation::MoveFile { from, to });
    }
    
    pub fn commit(mut self) -> anyhow::Result<()> {
        // 执行所有操作
        for operation in &self.operations {
            match operation {
                TransactionOperation::CreateNfo { path, content } => {
                    if let Some(parent) = path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    std::fs::write(path, content)?;
                }
                TransactionOperation::MoveFile { from, to } => {
                    safe_move_file(from, to, true)?;
                }
                TransactionOperation::CreateDirectory { path } => {
                    std::fs::create_dir_all(path)?;
                }
            }
        }
        
        self.completed = true;
        Ok(())
    }
}

impl Drop for FileProcessingTransaction {
    fn drop(&mut self) {
        if !self.completed {
            log::warn!("文件处理事务未完成，正在回滚操作");
            // 这里可以实现回滚逻辑
        }
    }
}
```

## 推荐的最佳实践

### 1. 处理流程
1. **预检查阶段**
   - 验证文件存在性和可访问性
   - 获取文件锁和处理锁
   - 创建文件完整性检查器

2. **安全处理阶段**
   - 开启文件处理事务
   - 执行爬取和NFO生成
   - 准备文件移动操作

3. **提交阶段**
   - 验证文件完整性
   - 原子性执行所有文件操作
   - 提交事务并释放锁

4. **清理阶段**
   - 释放所有资源
   - 清理临时文件
   - 记录处理结果

### 2. 错误处理
- 所有文件操作都应该有适当的错误处理
- 使用Result类型进行错误传播
- 记录详细的错误日志
- 提供用户友好的错误消息

### 3. 监控和日志
- 记录所有文件操作的开始和结束
- 监控文件锁的获取和释放
- 跟踪文件处理的性能指标
- 提供处理进度的实时反馈

### 4. 配置选项
```toml
[file_safety]
# 启用文件锁定机制
enable_file_locking = true
# 启用完整性检查
enable_integrity_check = true
# 启用文件备份
enable_backup = true
# 处理超时时间（秒）
processing_timeout = 300
# 锁文件清理间隔（秒）
lock_cleanup_interval = 3600
```

## 实施建议

1. **渐进式部署**
   - 先在测试环境验证所有安全机制
   - 逐步启用各项安全功能
   - 监控性能影响

2. **用户教育**
   - 提供清晰的使用文档
   - 说明文件安全机制的工作原理
   - 提供故障排除指南

3. **持续监控**
   - 定期检查锁文件的状态
   - 监控文件操作的成功率
   - 收集用户反馈

通过实施这些文件安全策略，可以显著降低文件在处理过程中被意外操作的风险，确保系统的稳定性和数据的完整性。