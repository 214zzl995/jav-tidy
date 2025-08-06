/// 媒体中心集成验证示例
///
/// 这个模块展示了 jav-tidy-rs 生成的文件结构如何符合 Emby/Jellyfin/Kodi 标准
use crate::config::AppConfig;
use crate::file_organizer::FileOrganizer;
use crate::nfo::{MovieNfo, MovieSet};
use std::path::Path;

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;

    fn create_test_config() -> AppConfig {
        let test_config_content = r#"
migrate_files = ["mp4", "mkv", "avi"]
migrate_subtitles = true
ignored_id_pattern = []
capital = false
input_dir = "./test_input"
output_dir = "./test_media_center"
thread_limit = 4
template_priority = ["javdb.yaml"]
maximum_fetch_count = 3
file_naming_template = "$series$/$title$ ($year$)"
multi_actor_strategy = "symlink"
subtitle_language = "zh-CN"
"#;

        let temp_dir = env::temp_dir();
        let config_path = temp_dir.join("test_media_center_config.toml");
        fs::write(&config_path, test_config_content).unwrap();

        AppConfig::new(&config_path).unwrap()
    }

    fn create_sample_nfo_with_series() -> MovieNfo {
        MovieNfo {
            title: "人妻自宅エステサロン".to_string(),
            original_title: "Married Woman Home Salon".to_string(),
            plot: "一个关于家庭按摩沙龙的故事...".to_string(),
            year: Some(2024),
            premiered: "2024-07-10".to_string(),
            release_date: "2024-07-10".to_string(),
            genres: vec!["Drama".to_string(), "Adult".to_string()],
            studios: vec!["IDEA POCKET".to_string()],
            set: Some(MovieSet {
                name: "家庭按摩系列".to_string(),
                overview: "探索家庭服务行业的系列作品".to_string(),
            }),
            ..Default::default()
        }
    }

    fn create_sample_nfo_without_series() -> MovieNfo {
        MovieNfo {
            title: "单独作品".to_string(),
            original_title: "Single Work".to_string(),
            plot: "这是一个独立的作品".to_string(),
            year: Some(2023),
            premiered: "2023-12-01".to_string(),
            release_date: "2023-12-01".to_string(),
            genres: vec!["Drama".to_string()],
            studios: vec!["Test Studio".to_string()],
            set: None, // 没有系列信息
            ..Default::default()
        }
    }

    #[test]
    fn test_media_center_structure_with_series() {
        let organizer = FileOrganizer::new();
        let config = create_test_config();
        let nfo = create_sample_nfo_with_series();

        let original_path = Path::new("./test_input/IPZZ-315.mp4");
        let result = organizer.preview_media_center_structure(original_path, &nfo, &config);

        assert!(result.is_ok());
        let (video_path, nfo_path) = result.unwrap();

        // 验证目录结构：家庭按摩系列/人妻自宅エステサロン (2024) (基于默认模板 $series$/$title$ ($year$))
        let expected_series_dir = "家庭按摩系列";
        let expected_title_dir = "人妻自宅エステサロン (2024)";
        assert!(video_path.to_string_lossy().contains(expected_series_dir));
        assert!(video_path.to_string_lossy().contains(expected_title_dir));
        assert!(nfo_path.to_string_lossy().contains(expected_series_dir));
        assert!(nfo_path.to_string_lossy().contains(expected_title_dir));

        // 验证文件名：人妻自宅エステサロン (2024).mp4
        let expected_filename = "人妻自宅エステサロン (2024).mp4";
        assert!(video_path.file_name().unwrap().to_str().unwrap() == expected_filename);

        // 验证NFO文件名：人妻自宅エステサロン (2024).nfo
        let expected_nfo_filename = "人妻自宅エステサロン (2024).nfo";
        assert!(nfo_path.file_name().unwrap().to_str().unwrap() == expected_nfo_filename);

        // 验证它们在同一个目录中
        assert_eq!(video_path.parent(), nfo_path.parent());

        println!("📁 媒体中心结构 (有系列):");
        println!("   视频文件: {}", video_path.display());
        println!("   NFO文件:  {}", nfo_path.display());
    }

    #[test]
    fn test_media_center_structure_without_series() {
        let organizer = FileOrganizer::new();
        let config = create_test_config();
        let nfo = create_sample_nfo_without_series();

        let original_path = Path::new("./test_input/TEST-001.mp4");
        let result = organizer.preview_media_center_structure(original_path, &nfo, &config);

        assert!(result.is_ok());
        let (video_path, nfo_path) = result.unwrap();

        // 验证目录结构：单独作品 (2023)/ （没有系列时使用标题作为目录名）
        let expected_dir = "单独作品 (2023)";
        assert!(video_path.to_string_lossy().contains(expected_dir));
        assert!(nfo_path.to_string_lossy().contains(expected_dir));

        // 验证文件名
        let expected_filename = "单独作品 (2023).mp4";
        assert!(video_path.file_name().unwrap().to_str().unwrap() == expected_filename);

        let expected_nfo_filename = "单独作品 (2023).nfo";
        assert!(nfo_path.file_name().unwrap().to_str().unwrap() == expected_nfo_filename);

        println!("📁 媒体中心结构 (无系列):");
        println!("   视频文件: {}", video_path.display());
        println!("   NFO文件:  {}", nfo_path.display());
    }

    #[test]
    fn test_emby_jellyfin_kodi_compatibility() {
        let organizer = FileOrganizer::new();
        let config = create_test_config();
        let nfo = create_sample_nfo_with_series();

        let original_path = Path::new("./test_input/IPZZ-315.mp4");
        let (video_path, nfo_path) = organizer
            .preview_media_center_structure(original_path, &nfo, &config)
            .unwrap();

        // 验证符合媒体中心扫描标准

        // 1. 目录命名包含年份：系列名 (年份)
        let dir_name = video_path
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();
        assert!(dir_name.contains("(2024)"));

        // 2. 视频文件和NFO文件在同一目录
        assert_eq!(video_path.parent(), nfo_path.parent());

        // 3. NFO文件名与视频文件名匹配（除了扩展名）
        let video_stem = video_path.file_stem().unwrap().to_str().unwrap();
        let nfo_stem = nfo_path.file_stem().unwrap().to_str().unwrap();
        assert_eq!(video_stem, nfo_stem);

        // 4. 文件名包含年份信息
        assert!(video_stem.contains("(2024)"));

        // 5. 非法字符已被清理（测试日文字符处理）
        assert!(!video_stem.contains("<"));
        assert!(!video_stem.contains(">"));
        assert!(!video_stem.contains(":"));
        assert!(!video_stem.contains("\""));
        assert!(!video_stem.contains("/"));
        assert!(!video_stem.contains("\\"));
        assert!(!video_stem.contains("|"));
        assert!(!video_stem.contains("?"));
        assert!(!video_stem.contains("*"));

        println!("✅ 媒体中心兼容性验证通过:");
        println!("   🎯 Emby: 支持系列分组和NFO元数据");
        println!("   🎯 Jellyfin: 支持标准目录结构和文件命名");
        println!("   🎯 Kodi: 支持.nfo文件和艺术作品");
        println!("   📂 结构: {}", video_path.parent().unwrap().display());
        println!(
            "   🎬 视频: {}",
            video_path.file_name().unwrap().to_str().unwrap()
        );
        println!(
            "   📄 NFO:  {}",
            nfo_path.file_name().unwrap().to_str().unwrap()
        );
    }
}
