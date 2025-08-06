#[cfg(test)]
mod tests {
    use crate::config::AppConfig;
    use std::path::Path;

    #[test]
    fn test_config_loading_with_new_fields() {
        // 创建一个临时的测试配置文件
        let config_content = r#"
migrate_files = ["mp4", "mkv"]
migrate_subtitles = true
ignored_id_pattern = ["-HD", "_"]
capital = false
input_dir = "./input"
output_dir = "./output"  
thread_limit = 2
maximum_fetch_count = 1
template_priority = ["javdb.yaml"]
file_naming_template = "$actor$/$title$ ($year$)"
multi_actor_strategy = "symlink"
subtitle_extensions = ["srt", "ass"]
subtitle_language = "zh-CN"
"#;

        // 写入临时文件
        let temp_path = "./temp_test_config.toml";
        std::fs::write(temp_path, config_content).unwrap();

        // 加载配置
        let config = AppConfig::new(Path::new(temp_path));
        
        // 清理临时文件
        std::fs::remove_file(temp_path).ok();
        
        assert!(config.is_ok(), "配置加载失败: {:?}", config.err());
        
        let config = config.unwrap();
        
        // 验证新字段
        assert_eq!(config.get_file_naming_template(), "$actor$/$title$ ($year$)");
        assert_eq!(config.get_multi_actor_strategy(), "symlink");
        assert_eq!(config.get_subtitle_extensions(), &["srt", "ass"]);
        assert_eq!(config.get_subtitle_language(), "zh-CN");
        
        // 验证现有字段仍然正常
        assert_eq!(config.migrate_files, vec!["mp4", "mkv"]);
        assert_eq!(config.maximum_fetch_count, 1);
        assert_eq!(config.template_priority, vec!["javdb.yaml"]);
    }

    #[test]
    fn test_config_with_default_values() {
        // 测试不包含新字段的配置文件（应该使用默认值）
        let config_content = r#"
migrate_files = ["mp4"]
migrate_subtitles = false
ignored_id_pattern = []
capital = false
input_dir = "./input"
output_dir = "./output"
thread_limit = 1
template_priority = ["javdb.yaml"]
"#;

        let temp_path = "./temp_test_config_default.toml";
        std::fs::write(temp_path, config_content).unwrap();

        let config = AppConfig::new(Path::new(temp_path));
        std::fs::remove_file(temp_path).ok();
        
        assert!(config.is_ok());
        let config = config.unwrap();
        
        // 验证默认值
        assert_eq!(config.get_file_naming_template(), "$series$/$title$ ($year$)");
        assert_eq!(config.get_multi_actor_strategy(), "symlink");
        
        // 验证默认字幕扩展名
        let default_extensions = vec!["srt", "ass", "ssa", "vtt", "sub", "idx", "sup"];
        assert_eq!(config.get_subtitle_extensions(), default_extensions);
        
        // 验证默认字幕语言
        assert_eq!(config.get_subtitle_language(), "zh-CN");
    }
}