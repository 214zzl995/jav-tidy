# 四大影片整理平台NFO数据格式标准对比分析

## 核心发现总结

**Kodi是NFO格式标准的创始者**，其他三个平台都基于Kodi的规范进行实现。**Plex不原生支持NFO**，完全依赖第三方代理。**Emby和Jellyfin共享几乎相同的格式**，Jellyfin作为Emby的开源分支保持了高度兼容性。所有平台都使用UTF-8编码的XML格式，但在具体标签支持和处理逻辑上存在差异。

## 1. 平台NFO支持概况对比

| 平台 | 原生支持 | 格式基础 | 主要特点 |
|------|----------|----------|----------|
| **Kodi** | ✅ 完全原生 | 自创标准 | 最全面的标签系统，技术参数详细 |
| **Emby** | ✅ 完全支持 | 基于Kodi | 商业化平台，支持用户数据同步 |
| **Jellyfin** | ✅ 完全支持 | 基于Kodi/Emby | 开源免费，与Emby高度兼容 |
| **Plex** | ❌ 第三方代理 | 基于Kodi | 需要安装XBMCnfoImporter等代理 |

## 2. 完整NFO标签分类对比

### 2.1 基本信息标签

#### 核心标识标签
| 标签名 | Kodi | Emby | Jellyfin | Plex | 数据类型 | 说明 |
|--------|------|------|----------|------|----------|------|
| `<title>` | ✅ 必需 | ✅ 必需 | ✅ 必需 | ✅ 必需 | 字符串 | 媒体标题 |
| `<originaltitle>` | ✅ | ✅ | ✅ | ✅ | 字符串 | 原始语言标题 |
| `<sorttitle>` | ✅ | ✅ | ✅ | ✅ | 字符串 | 排序标题 |
| `<localtitle>` | ❌ | ❌ | ✅ 特有 | ❌ | 字符串 | Jellyfin本地标题 |
| `<sortname>` | ❌ | ❌ | ✅ 特有 | ❌ | 字符串 | Jellyfin排序名称 |
| `<showtitle>` | ✅ | ✅ | ✅ | ✅ | 字符串 | 剧集所属节目标题 |

#### 时间相关标签
| 标签名 | Kodi | Emby | Jellyfin | Plex | 数据类型 | 格式 |
|--------|------|------|----------|------|----------|------|
| `<year>` | ✅ | ✅ | ✅ | ✅ | 数字 | 年份 |
| `<premiered>` | ✅ | ✅ | ✅ | ✅ | 日期 | yyyy-mm-dd |
| `<releasedate>` | ✅ | ✅ | ✅ | ❌ | 日期 | yyyy-mm-dd |
| `<aired>` | ✅ | ✅ | ✅ | ✅ | 日期 | yyyy-mm-dd (仅剧集) |
| `<dateadded>` | ✅ | ✅ | ✅ | ❌ | 时间戳 | yyyy-MM-dd HH:mm:ss |
| `<enddate>` | ✅ | ✅ | ✅ | ❌ | 日期 | yyyy-mm-dd |

#### 内容描述标签
| 标签名 | Kodi | Emby | Jellyfin | Plex | 数据类型 |
|--------|------|------|----------|------|----------|
| `<plot>` | ✅ | ✅ | ✅ | ✅ | 字符串 |
| `<outline>` | ✅ | ✅ | ❌ | ✅ | 字符串 |
| `<tagline>` | ✅ | ✅ | ✅ | ✅ | 字符串 |
| `<biography>` | ✅ | ❌ | ✅ | ❌ | 字符串 |
| `<review>` | ✅ | ❌ | ✅ | ❌ | 字符串 |

### 2.2 评分标签系统

#### 简单评分格式
```xml
<!-- 所有平台通用 -->
<rating max="10">8.5</rating>
<userrating max="10">9.0</userrating>
<votes>12345</votes>
<top250>50</top250>
<criticrating>85</criticrating>
```

#### 高级评分格式 (Kodi v18+, Emby, Jellyfin)
```xml
<ratings>
    <rating name="imdb" max="10" default="true">
        <value>8.5</value>
        <votes>50000</votes>
    </rating>
    <rating name="themoviedb" max="10">
        <value>7.8</value>
        <votes>1200</votes>
    </rating>
    <rating name="metacritic" max="100">
        <value>78</value>
    </rating>
</ratings>
```

| 评分支持 | Kodi | Emby | Jellyfin | Plex |
|----------|------|------|----------|------|
| 简单评分 | ✅ | ✅ | ✅ | ✅ |
| 多源评分 | ✅ v18+ | ✅ | ✅ 部分 | ❌ |
| 自定义评分 | ✅ | ✅ | ✅ | ❌ |

### 2.3 人员信息标签

#### 制作人员
| 标签名 | Kodi | Emby | Jellyfin | Plex | 多值支持 |
|--------|------|------|----------|------|----------|
| `<director>` | ✅ | ✅ | ✅ | ✅ | 是 |
| `<credits>` | ✅ | ✅ | ✅ | ✅ | 是 |
| `<writer>` | ✅ v19+ | ✅ | ✅ | ❌ | 是 |
| `<producer>` | ✅ | ✅ | 通过actor | ✅ | 是 |

#### 演员信息结构
```xml
<actor>
    <name>演员姓名</name>
    <role>角色名称</role>
    <order>1</order>                    <!-- Kodi, Emby, Jellyfin -->
    <sortorder>0</sortorder>            <!-- Jellyfin特有 -->
    <type>Actor</type>                  <!-- Jellyfin特有 -->
    <thumb>演员头像URL或路径</thumb>
</actor>
```

### 2.4 技术信息标签

#### 基本技术参数
| 标签名 | Kodi | Emby | Jellyfin | Plex |
|--------|------|------|----------|------|
| `<runtime>` | ✅ | ✅ | ✅ | ✅ |
| `<aspectratio>` | ✅ | ❌ | ✅ | ❌ |
| `<mpaa>` | ✅ | ✅ | ✅ | ✅ |

#### 详细流媒体信息
所有平台都支持 `<fileinfo><streamdetails>` 结构，但具体标签略有差异：

```xml
<fileinfo>
    <streamdetails>
        <video>
            <codec>h264</codec>
            <micodec>h264</micodec>          <!-- Jellyfin特有 -->
            <aspect>1.78</aspect>
            <aspectratio>16:9</aspectratio>  <!-- Jellyfin特有 -->
            <width>1920</width>
            <height>1080</height>
            <resolution>1080</resolution>    <!-- Emby特有 -->
            <framerate>23.976</framerate>
            <durationinseconds>7200</durationinseconds>
            <stereomode>mono</stereomode>    <!-- Kodi特有 -->
            <hdrtype>HDR10</hdrtype>         <!-- Kodi特有 -->
            <scantype>progressive</scantype>
            <language>eng</language>
            <bitrate>2249020</bitrate>       <!-- Jellyfin特有 -->
            <default>True</default>          <!-- Jellyfin特有 -->
            <forced>False</forced>           <!-- Jellyfin特有 -->
        </video>
        <audio>
            <codec>dtshd_ma</codec>
            <micodec>aac</micodec>           <!-- Jellyfin特有 -->
            <language>eng</language>
            <channels>8</channels>
            <samplingrate>48000</samplingrate> <!-- Jellyfin特有 -->
            <bitrate>131061</bitrate>        <!-- Jellyfin特有 -->
            <default>True</default>          <!-- Jellyfin特有 -->
            <forced>False</forced>           <!-- Jellyfin特有 -->
        </audio>
        <subtitle>
            <language>eng</language>
            <codec>srt</codec>               <!-- Jellyfin特有 -->
            <default>False</default>         <!-- Jellyfin特有 -->
            <forced>False</forced>           <!-- Jellyfin特有 -->
        </subtitle>
    </streamdetails>
</fileinfo>
```

### 2.5 收藏信息标签

#### 系列/合集标签
```xml
<set>
    <name>系列名称</name>
    <overview>系列描述</overview>
</set>
<collectionnumber>123456</collectionnumber>  <!-- Emby特有 -->
```

#### 分类标签
| 标签名 | Kodi | Emby | Jellyfin | Plex | 多值支持 |
|--------|------|------|----------|------|----------|
| `<genre>` | ✅ | ✅ | ✅ | ✅ | 是 |
| `<tag>` | ✅ | ✅ | ✅ | ✅ | 是 |
| `<studio>` | ✅ | ✅ | ✅ | ✅ | 是 |
| `<country>` | ✅ | ✅ | ✅ | ✅ | 是 |
| `<style>` | ✅ | ✅ | ✅ 音乐特有 | ❌ | 是 |

### 2.6 唯一标识符系统

#### 新格式标识符 (推荐)
```xml
<uniqueid type="imdb" default="true">tt1234567</uniqueid>
<uniqueid type="tmdb">12345</uniqueid>
<uniqueid type="tvdb">67890</uniqueid>
```

#### 传统格式标识符
| 标签名 | Kodi | Emby | Jellyfin | Plex |
|--------|------|------|----------|------|
| `<imdbid>` | ✅ | ✅ | ✅ | ✅ |
| `<tmdbid>` | ✅ | ✅ | ✅ | ✅ |
| `<tvdbid>` | ✅ | ✅ | ✅ | ✅ |
| `<id>` | ✅ 已废弃 | ❌ | ✅ | ❌ |

#### 动态提供商ID (Jellyfin特有)
Jellyfin支持格式：`<PROVIDER_NAME+id>`
- `<imdb_id>`, `<tmdbid>`, `<tvdbid>`, `<thesportsdbid>` 等

### 2.7 图像艺术标签

#### Kodi特有的丰富艺术品支持
```xml
<thumb>海报图URL</thumb>
<poster>海报URL</poster>
<landscape>横版图URL</landscape>
<banner>横幅图URL</banner>
<clearlogo>透明logo URL</clearlogo>
<clearart>透明艺术图URL</clearart>
<discart>光盘艺术图URL</discart>

<fanart>
    <thumb preview="预览图URL">高清背景图URL</thumb>
    <thumb preview="预览图URL2">高清背景图URL2</thumb>
</fanart>
```

#### 其他平台艺术品支持
```xml
<!-- Emby/Jellyfin/Plex通用 -->
<thumb aspect="poster">海报图片URL</thumb>
<thumb aspect="banner">横幅图片URL</thumb>

<art>
    <poster>海报路径</poster>
    <fanart>背景图路径</fanart>
    <banner>横幅路径</banner>          <!-- Emby特有 -->
    <clearart>透明艺术图路径</clearart>   <!-- Emby特有 -->
    <clearlogo>透明Logo路径</clearlogo> <!-- Emby特有 -->
    <landscape>横向图路径</landscape>   <!-- Emby特有 -->
    <disc>光盘艺术图路径</disc>          <!-- Emby特有 -->
</art>
```

### 2.8 电视剧特殊标签

#### 剧集定位标签
| 标签名 | Kodi | Emby | Jellyfin | Plex | 用途 |
|--------|------|------|----------|------|------|
| `<season>` | ✅ | ✅ | ✅ | ✅ | 季数 |
| `<episode>` | ✅ | ✅ | ✅ | ✅ | 集数 |
| `<displayseason>` | ✅ | ✅ | ✅ | ❌ | 显示季数 |
| `<displayepisode>` | ✅ | ✅ | ✅ | ❌ | 显示集数 |
| `<airsbefore_season>` | ✅ | ✅ | ✅ | ❌ | 特别篇排序 |
| `<airsbefore_episode>` | ✅ | ✅ | ✅ | ❌ | 特别篇排序 |
| `<airsafter_season>` | ✅ | ✅ | ✅ | ❌ | 特别篇排序 |

#### 电视剧状态标签
| 标签名 | Kodi | Emby | Jellyfin | Plex |
|--------|------|------|----------|------|
| `<status>` | ✅ | ✅ | ✅ | ✅ |
| `<displayorder>` | ✅ | ✅ | ✅ | ❌ |

#### 剧集指南 (Kodi特有)
```xml
<episodeguide>
    <url cache="auth.json">https://api.thetvdb.com/series/12345</url>
</episodeguide>
```

### 2.9 用户数据标签

| 标签名 | Kodi | Emby | Jellyfin | Plex | 数据类型 |
|--------|------|------|----------|------|----------|
| `<watched>` | ✅ | ✅ | ✅ | ❌ | 布尔值 |
| `<playcount>` | ✅ | ✅ | ✅ | ❌ | 数字 |
| `<lastplayed>` | ✅ | ✅ | ✅ | ❌ | 时间戳 |

#### 播放进度 (Kodi/Emby特有)
```xml
<resume>
    <position>1800.0</position>
    <total>7200.0</total>
</resume>
```

### 2.10 控制和锁定标签

| 标签名 | Kodi | Emby | Jellyfin | Plex | 用途 |
|--------|------|------|----------|------|------|
| `<lockdata>` | ❌ | ✅ | ✅ | ❌ | 锁定元数据 |
| `<lockedfields>` | ❌ | ✅ | ❌ | ❌ | 锁定特定字段 |

## 3. 平台特有标签和功能

### 3.1 Kodi特有功能
- **最全面的艺术品标签**: clearlogo, clearart, discart, landscape等
- **版本管理** (v21+): `<hasvideoversions>`, `<isdefaultvideoversion>`
- **技术标签**: `<stereomode>`, `<hdrtype>`, `<scantype>`
- **季度命名**: `<namedseason number="1">第一季</namedseason>`
- **主题音乐**: `<theme>主题音乐路径</theme>`

### 3.2 Emby特有功能
- **数据锁定**: `<lockdata>`, `<lockedfields>`
- **合集编号**: `<collectionnumber>`
- **用户评分**: `<customrating>`
- **更丰富的art结构**: banner, clearart, clearlogo, landscape, disc
- **分辨率标识**: `<resolution>1080</resolution>`

### 3.3 Jellyfin特有功能
- **本地化标签**: `<localtitle>`, `<sortname>`
- **演员类型**: `<type>Actor</type>`, `<sortorder>`
- **详细技术信息**: `<micodec>`, `<bitrate>`, `<samplingrate>`
- **流标识**: `<default>`, `<forced>` 用于音频和字幕
- **动态provider ID**: 支持任意格式的provider标识

### 3.4 Plex限制
- **无原生NFO支持**: 完全依赖第三方代理
- **功能受限**: 不支持用户数据、播放进度等
- **兼容性问题**: 新版Plex Agent与NFO代理兼容性差
- **维护风险**: 第三方代理可能停止维护

## 4. NFO文件命名规范对比

| 媒体类型 | Kodi | Emby | Jellyfin | Plex |
|----------|------|------|----------|------|
| 电影 | `<文件名>.nfo` | `movie.nfo` 或 `<文件名>.nfo` | `movie.nfo` 或 `<文件名>.nfo` | `<文件名>.nfo` |
| 电视剧 | `tvshow.nfo` | `tvshow.nfo` | `tvshow.nfo` | `tvshow.nfo` |
| 剧集 | `<集文件名>.nfo` | `<集文件名>.nfo` | `<集文件名>.nfo` | `<集文件名>.nfo` |
| 音乐艺术家 | `artist.nfo` | `artist.nfo` | `artist.nfo` | 不支持 |
| 音乐专辑 | `album.nfo` | `album.nfo` | `album.nfo` | 不支持 |

## 5. 实际NFO文件示例对比

### 5.1 电影NFO完整示例

#### Kodi格式 (最全面)
```xml
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<movie>
    <title>复仇者联盟</title>
    <originaltitle>The Avengers</originaltitle>
    <sorttitle>Avengers, The</sorttitle>
    <year>2012</year>
    <premiered>2012-05-04</premiered>
    <plot>当地球面临前所未有的威胁时，神盾局局长尼克·弗瑞发现他必须启动一个他一直在酝酿的计划...</plot>
    <tagline>有些事情无法独自面对</tagline>
    <runtime>143</runtime>
    <mpaa>PG-13</mpaa>
    
    <uniqueid type="imdb" default="true">tt0848228</uniqueid>
    <uniqueid type="tmdb">24428</uniqueid>
    
    <ratings>
        <rating name="imdb" max="10" default="true">
            <value>8.0</value>
            <votes>1345678</votes>
        </rating>
    </ratings>
    
    <genre>动作</genre>
    <genre>冒险</genre>
    <genre>科幻</genre>
    
    <set>
        <name>复仇者联盟系列</name>
        <overview>漫威电影宇宙中复仇者联盟的故事</overview>
    </set>
    
    <director>乔斯·韦登</director>
    <credits>扎克·佩恩</credits>
    <credits>乔斯·韦登</credits>
    
    <actor>
        <name>小罗伯特·唐尼</name>
        <role>托尼·斯塔克 / 钢铁侠</role>
        <order>1</order>
        <thumb>https://image.tmdb.org/t/p/original/5qHNjhtjMD4YWH3UP0rm4tKwxCL.jpg</thumb>
    </actor>
    
    <thumb>https://image.tmdb.org/t/p/original/RYMX2wcKCBAr24UyPD7xwmjaTn.jpg</thumb>
    <fanart>
        <thumb preview="https://image.tmdb.org/t/p/w780/9BBTo63ANSmhC4e6r62OJFuK2GL.jpg">https://image.tmdb.org/t/p/original/9BBTo63ANSmhC4e6r62OJFuK2GL.jpg</thumb>
    </fanart>
    
    <fileinfo>
        <streamdetails>
            <video>
                <codec>h264</codec>
                <aspect>1.78</aspect>
                <width>1920</width>
                <height>1080</height>
                <durationinseconds>8580</durationinseconds>
                <stereomode>mono</stereomode>
            </video>
            <audio>
                <codec>dtshd_ma</codec>
                <language>eng</language>
                <channels>8</channels>
            </audio>
            <subtitle>
                <language>chi</language>
            </subtitle>
        </streamdetails>
    </fileinfo>
</movie>
```

#### Emby/Jellyfin格式
```xml
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<movie>
    <title>复仇者联盟</title>
    <originaltitle>The Avengers</originaltitle>
    <year>2012</year>
    <plot>当地球面临前所未有的威胁时...</plot>
    <runtime>143</runtime>
    <mpaa>PG-13</mpaa>
    <lockdata>false</lockdata>
    
    <uniqueid type="imdb" default="true">tt0848228</uniqueid>
    <uniqueid type="tmdb">24428</uniqueid>
    
    <ratings>
        <rating name="imdb" max="10" default="true">
            <value>8.0</value>
            <votes>1345678</votes>
        </rating>
    </ratings>
    
    <genre>动作</genre>
    <genre>冒险</genre>
    
    <set>
        <name>复仇者联盟系列</name>
    </set>
    
    <director>乔斯·韦登</director>
    <writer>扎克·佩恩</writer>
    
    <actor>
        <name>小罗伯特·唐尼</name>
        <role>托尼·斯塔克 / 钢铁侠</role>
        <order>1</order>
    </actor>
    
    <art>
        <poster>poster.jpg</poster>
        <fanart>fanart.jpg</fanart>
    </art>
    
    <fileinfo>
        <streamdetails>
            <video>
                <codec>h264</codec>
                <width>1920</width>
                <height>1080</height>
                <resolution>1080</resolution>
            </video>
            <audio>
                <codec>dts</codec>
                <language>eng</language>
                <channels>6</channels>
            </audio>
        </streamdetails>
    </fileinfo>
</movie>
```

### 5.2 电视剧剧集NFO示例

#### 多集单文件格式 (所有平台支持)
```xml
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<episodedetails>
    <title>第一集标题</title>
    <showtitle>电视剧名称</showtitle>
    <season>1</season>
    <episode>1</episode>
    <plot>第一集剧情...</plot>
    <aired>2023-01-01</aired>
    <director>导演姓名</director>
    <credits>编剧姓名</credits>
</episodedetails>
<episodedetails>
    <title>第二集标题</title>
    <showtitle>电视剧名称</showtitle>
    <season>1</season>
    <episode>2</episode>
    <plot>第二集剧情...</plot>
    <aired>2023-01-08</aired>
    <director>导演姓名</director>
    <credits>编剧姓名</credits>
</episodedetails>
```

## 6. 数据类型和层级关系

### 6.1 数据类型规范
- **字符串**: UTF-8编码文本，如标题、描述、人员姓名
- **数字**: 整数或浮点数，如年份(2023)、评分(8.5)、时长(120)
- **日期**: yyyy-mm-dd格式，如首映日期
- **时间戳**: yyyy-MM-dd HH:mm:ss格式，如添加时间
- **布尔值**: "true"/"false"字符串，如观看状态
- **URL**: HTTP/HTTPS链接或本地文件路径

### 6.2 XML结构层级
```
根元素 (<movie>/<tvshow>/<episodedetails>)
├── 基本信息标签 (title, year, plot等)
├── 评分信息
│   ├── 简单评分 (rating, votes)
│   └── 复杂评分 (ratings容器)
│       └── 单个评分 (rating元素)
│           ├── value
│           └── votes
├── 人员信息
│   ├── 简单人员 (director, writer)
│   └── 演员信息 (actor容器)
│       ├── name
│       ├── role
│       ├── order
│       └── thumb
├── 技术信息 (fileinfo容器)
│   └── streamdetails
│       ├── video
│       │   ├── codec, width, height等
│       ├── audio
│       │   ├── codec, language, channels等
│       └── subtitle
│           └── language等
├── 收藏信息
│   ├── 类型信息 (genre, tag, studio)
│   └── 系列信息 (set容器)
│       ├── name
│       └── overview
├── 标识符信息
│   ├── 新格式 (uniqueid容器)
│   └── 传统格式 (imdbid, tmdbid等)
└── 艺术品信息
    ├── 简单图像 (thumb, poster)
    ├── fanart容器
    └── art容器 (Emby/Jellyfin)
```

## 7. 标签必需性分析

### 7.1 必需标签
所有平台都要求的最低标签：
- `<title>` - 媒体标题 (字符串)
- 对应的根元素 (`<movie>`, `<tvshow>`, `<episodedetails>`)

### 7.2 强烈推荐标签
- `<year>` - 年份，有助于识别
- `<plot>` - 剧情描述，提升用户体验
- `<genre>` - 类型，方便分类
- `<uniqueid>` 或相应ID标签 - 准确识别媒体

### 7.3 可选但有价值的标签
- `<director>`, `<actor>` - 人员信息
- `<rating>`, `<votes>` - 评分信息
- `<fileinfo>` - 技术参数
- 艺术品标签 - 视觉体验

## 8. 兼容性和迁移建议

### 8.1 跨平台兼容性策略
1. **使用通用标签**: 优先使用所有平台都支持的标签
2. **保持核心信息**: title, year, plot, genre等基础信息
3. **标准化ID**: 使用uniqueid格式，同时保留传统ID标签
4. **简化技术信息**: 只包含关键的streamdetails信息

### 8.2 平台迁移建议
- **从Plex迁移**: 考虑迁移到支持原生NFO的平台
- **Emby到Jellyfin**: 迁移成本最低，几乎完全兼容
- **Kodi作为标准**: 以Kodi格式为基准，其他平台向下兼容
- **工具推荐**: 使用tinyMediaManager、MediaElch等工具统一管理

### 8.3 最佳实践总结
1. **文件编码**: 统一使用UTF-8编码
2. **命名规范**: 推荐使用与媒体文件同名的NFO文件
3. **版本控制**: 定期备份NFO文件
4. **工具辅助**: 使用专业NFO管理工具而非手动编辑
5. **渐进增强**: 从基础标签开始，逐步添加高级功能

这份对比分析为选择合适的媒体管理平台和制定NFO标准化策略提供了全面的技术依据。Kodi作为格式标准的制定者提供了最全面的功能，Emby和Jellyfin在保持兼容性的基础上各有特色，而Plex的NFO支持需要额外的技术投入。