# 文件命名模板系统

## 概述

jav-tidy-rs 现在支持灵活的文件命名模板系统，允许用户根据影片的元数据信息自定义文件和目录的命名规则。

## 配置参数

在 `config.toml` 中可以配置以下参数：

```toml
# 文件命名模板，支持变量如 $title$, $actor$, $year$, $series$ 等
file_naming_template = "$series$/$title$ ($year$)"

# 多演员处理策略：symlink (符号链接) / hardlink (硬链接) / first_only (仅第一个) / merge (合并)
multi_actor_strategy = "symlink"
```

## 可用的模板变量

| 变量名 | 描述 | 示例 |
|--------|------|------|
| `$title$` | 影片标题 | "人妻自宅エステサロン" |
| `$original_title$` | 原始标题 | "Married Woman Home Spa" |
| `$year$` | 年份 | "2024" |
| `$series$` | 系列名 | "家庭按摩系列" |
| `$actor$` | 演员名（第一个或根据策略处理） | "演员A" |
| `$director$` | 导演名 | "导演A" |
| `$studio$` | 制片厂 | "IDEA POCKET" |
| `$genre$` | 类型（第一个） | "Drama" |
| `$id$` | 影片ID | "IPZZ-315" |

## 模板示例

### 1. 默认模板（系列/标题 年份）
```toml
file_naming_template = "$series$/$title$ ($year$)"
```
生成结构：
```
输出目录/
├── 家庭按摩系列/
│   └── 人妻自宅エステサロン (2024).mp4
│   └── 人妻自宅エステサロン (2024).nfo
```

### 2. 演员分类模板
```toml
file_naming_template = "$actor$/$title$ ($year$)"
```
生成结构：
```
输出目录/
├── 演员A/
│   └── 人妻自宅エステサロン (2024).mp4
│   └── 人妻自宅エステサロン (2024).nfo
```

### 3. 制片厂分类模板
```toml
file_naming_template = "$studio$/$series$/$title$ ($year$)"
```
生成结构：
```
输出目录/
├── IDEA POCKET/
│   └── 家庭按摩系列/
│       └── 人妻自宅エステサロン (2024).mp4
│       └── 人妻自宅エステサロン (2024).nfo
```

### 4. 年份分类模板
```toml
file_naming_template = "$year$/$genre$/$title$"
```
生成结构：
```
输出目录/
├── 2024/
│   └── Drama/
│       └── 人妻自宅エステサロン.mp4
│       └── 人妻自宅エステサロン.nfo
```

## 多演员处理策略

当影片有多个演员时，可以选择不同的处理策略：

### 1. SymLink（符号链接）- 默认推荐
```toml
multi_actor_strategy = "symlink"
```
- 在第一个演员目录下创建实际文件
- 在其他演员目录下创建符号链接
- 节省磁盘空间，跨平台兼容性好

### 2. HardLink（硬链接）
```toml
multi_actor_strategy = "hardlink"
```
- 创建硬链接，多个目录指向同一个文件
- 更好的性能，但有文件系统限制
- 如果硬链接创建失败，会自动回退到符号链接

### 3. FirstOnly（仅第一个演员）
```toml
multi_actor_strategy = "first_only"
```
- 只使用第一个演员的名字
- 不创建额外的链接

### 4. Merge（合并演员名称）
```toml
multi_actor_strategy = "merge"
```
- 将所有演员名称合并，用" & "连接
- 例如："演员A & 演员B & 演员C"

## 实际示例

假设有以下影片信息：
- 标题："人妻自宅エステサロン"
- 年份：2024
- 系列："家庭按摩系列"
- 演员：["演员A", "演员B"]
- 制片厂："IDEA POCKET"

使用模板 `"$actor$/$title$ ($year$)"` 和策略 `"symlink"`：

```
输出目录/
├── 演员A/
│   └── 人妻自宅エステサロン (2024).mp4    # 实际文件
│   └── 人妻自宅エステサロン (2024).nfo    # 实际文件
└── 演员B/
    └── 人妻自宅エステサロン (2024).mp4    # 符号链接 -> ../演员A/人妻自宅エステサロン (2024).mp4
    └── 人妻自宅エステサロン (2024).nfo    # 符号链接 -> ../演员A/人妻自宅エステサロン (2024).nfo
```

## 环境变量支持

也可以通过环境变量设置：

```bash
export JAVTIDY_FILE_NAMING_TEMPLATE='$actor$/$title$ ($year$)'
export JAVTIDY_MULTI_ACTOR_STRATEGY='symlink'
```

## 注意事项

1. **文件名清理**：模板变量中的非法字符会被自动清理
2. **路径分隔符**：使用 `/` 作为路径分隔符，跨平台兼容
3. **空值处理**：如果某个变量为空，会使用 "Unknown" 作为默认值
4. **系列信息**：如果没有系列信息，`$series$` 会使用标题作为回退值
5. **性能考虑**：符号链接通常是最佳选择，既节省空间又保持良好性能

## 兼容性

该模板系统完全兼容现有的媒体中心软件：
- ✅ Kodi
- ✅ Emby  
- ✅ Jellyfin
- ✅ Plex

生成的 NFO 文件遵循标准格式，确保在各种媒体中心软件中正确显示影片信息。