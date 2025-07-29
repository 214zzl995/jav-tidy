#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use crawler_template::Template;
    use crate::nfo::MovieNfoCrawler;

    /// 通用的 JavDB 模板爬取测试
    /// 可以通过修改 movie_id 来测试不同的影片
    #[tokio::test]
    async fn test_javdb_template_crawling() {
        let movie_id = "IPX-620"; // 可修改为其他影片ID进行测试
        
        println!("==================== 开始 JavDB 模板爬取测试 ====================");
        println!("🎯 目标影片ID: {}", movie_id);
        
        // 加载并解析模板
        println!("📂 加载模板文件: template/javdb.yaml");
        let template_content = std::fs::read_to_string("template/javdb.yaml")
            .expect("Failed to read template/javdb.yaml");
        println!("📝 模板文件大小: {} bytes", template_content.len());
        
        let template: Template<MovieNfoCrawler> = Template::from_yaml(&template_content)
            .expect("Failed to parse template");
        println!("✅ 模板解析成功");
        
        // 设置爬取参数
        let mut params = HashMap::new();
        params.insert("crawl_name", movie_id.to_string());
        params.insert("base_url", "https://javdb.com".to_string());
        println!("🔧 爬取参数设置完成: crawl_name={}, base_url=https://javdb.com", movie_id);
        
        // 执行爬取并验证结果
        println!("🚀 开始执行爬取...");
        let result = template.crawler(&params).await
            .expect("Template crawling should succeed");
        println!("✅ 爬取完成！");
        
        println!("\n==================== 爬取结果详情 ====================");
        println!("🎬 标题: {}", result.title);
        println!("📜 原始标题: {}", result.original_title);
        println!("🏠 本地标题: {}", result.local_title);
        println!("📝 剧情简介: {}", result.plot);
        println!("📅 首映日期: {}", result.premiered);
        println!("📆 发行日期: {}", result.release_date);
        println!("⭐ 评分: {:?}", result.rating);
        println!("📊 自定义评分: {}", result.custom_rating);
        println!("🔞 成人内容: {:?}", result.is_adult);
        
        println!("\n🎭 类型标签 ({} 个):", result.genres.len());
        for (i, genre) in result.genres.iter().enumerate() {
            println!("   {}. {}", i + 1, genre);
        }
        
        println!("\n🏢 制作商 ({} 个):", result.studios.len());
        for (i, studio) in result.studios.iter().enumerate() {
            println!("   {}. {}", i + 1, studio);
        }
        
        println!("\n🎬 导演 ({} 个):", result.directors.len());
        for (i, director) in result.directors.iter().enumerate() {
            println!("   {}. {}", i + 1, director);
        }
        
        println!("\n👥 演员信息 ({} 个):", result.actors.len());
        for (i, actor) in result.actors.iter().enumerate() {
            println!("   {}. {} (角色: {})", i + 1, actor.name, actor.role);
            if !actor.thumb.is_empty() {
                println!("       头像: {}", actor.thumb);
            }
        }
        
        println!("\n🖼️  海报图片 ({} 个):", result.posters.len());
        for (i, poster) in result.posters.iter().enumerate() {
            println!("   {}. {}", i + 1, poster);
        }
        
        println!("\n==================== 数据完整性验证 ====================");
        
        // 验证基本数据结构完整性
        assert!(!result.title.is_empty(), "Title should not be empty");
        println!("✅ 标题字段验证通过");
        
        assert!(!result.original_title.is_empty(), "Original title should not be empty");
        println!("✅ 原始标题字段验证通过");
        
        assert!(!result.genres.is_empty(), "Genres should not be empty");
        println!("✅ 类型字段验证通过 ({} 个标签)", result.genres.len());
        
        assert!(!result.studios.is_empty(), "Studios should not be empty");
        println!("✅ 制作商字段验证通过 ({} 个制作商)", result.studios.len());
        
        assert!(!result.actors.is_empty(), "Actors should not be empty");
        println!("✅ 演员字段验证通过 ({} 个演员)", result.actors.len());
        
        assert!(!result.posters.is_empty(), "Posters should not be empty");
        println!("✅ 海报字段验证通过 ({} 个海报)", result.posters.len());
        
        println!("\n🎉 所有验证通过！爬取测试成功完成！");
        println!("==================== 测试结束 ====================");
    }
}