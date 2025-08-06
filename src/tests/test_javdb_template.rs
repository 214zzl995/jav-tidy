#[cfg(test)]
mod tests {
    use crawler_template::Template;
    use crate::nfo::MovieNfoCrawler;
    use std::collections::HashMap;

    /// 通用的 JavDB 模板爬取测试
    /// 可以通过修改 movie_id 来测试不同的影片
    #[tokio::test]
    async fn test_javdb_template_crawling() {
        let movie_id = "IPZZ-315"; // 可修改为其他影片ID进行测试

        println!("==================== 开始 JavDB 模板爬取测试 ====================");
        println!("🎯 目标影片ID: {}", movie_id);

        // 加载并解析模板
        println!("📂 加载模板文件: template/javdb.yaml");
        let template_content = std::fs::read_to_string("template/javdb.yaml")
            .expect("Failed to read template/javdb.yaml");
        println!("📝 模板文件大小: {} bytes", template_content.len());

        let template: Template<MovieNfoCrawler> =
            Template::from_yaml(&template_content).expect("Failed to parse template");
        println!("✅ 模板解析成功");

        // 设置爬取参数
        let mut params = HashMap::new();
        params.insert("crawl_name", movie_id.to_string());
        params.insert("base_url", "https://javdb.com".to_string());
        println!(
            "🔧 爬取参数设置完成: crawl_name={}, base_url=https://javdb.com",
            movie_id
        );

        // 执行爬取并验证结果
        println!("🚀 开始执行爬取...");
        let result = template
            .crawler(&params)
            .await
            .expect("Template crawling should succeed");
        println!("✅ 爬取完成！");

        println!("\n==================== 爬取结果详情 ====================");
        println!("🎬 标题: {}", result.title);
        println!(
            "📜 原始标题: {}",
            result.original_title.as_ref().unwrap_or(&String::new())
        );
        println!("📝 剧情简介: {}", result.plot);
        println!("📅 首映日期: {}", result.premiered);
        println!("📆 发行日期: {}", result.release_date);
        println!("⭐ 评分: {:?}", result.rating);
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

        println!("\n🎨 预览图片 ({} 个):", result.preview_images.len());
        if result.preview_images.is_empty() {
            println!("   ❌ 未找到预览图片");
        } else {
            for (i, preview) in result.preview_images.iter().enumerate() {
                println!("   {}. {}", i + 1, preview);
            }
        }

        println!("\n🏆 TOP250 排名信息:");
        println!("   排名数字 ({} 个):", result.ranking_numbers.len());
        for (i, rank_num) in result.ranking_numbers.iter().enumerate() {
            let cleaned_rank = rank_num.strip_prefix("No.").unwrap_or(rank_num);
            println!("     {}. 第 {} 名", i + 1, cleaned_rank);
        }
        println!("   排名类别 ({} 个):", result.ranking_categories.len());
        for (i, category) in result.ranking_categories.iter().enumerate() {
            println!("     {}. {}", i + 1, category);
        }

        println!("\n🎥 系列信息:");
        if !result.series_name.is_empty() {
            println!("   系列名称: {}", result.series_name);
        } else {
            println!("   系列名称: 无");
        }
        if !result.series_overview.is_empty() {
            println!("   系列描述: {}", result.series_overview);
        } else {
            println!("   系列描述: 无");
        }

        // 显示最终的 Rating 结构预览
        if !result.ranking_numbers.is_empty() && !result.ranking_categories.is_empty() {
            println!("   组合排名信息:");
            for (rank_str, category) in result
                .ranking_numbers
                .iter()
                .zip(result.ranking_categories.iter())
            {
                let cleaned_rank = rank_str.strip_prefix("No.").unwrap_or(rank_str);
                if let Ok(rank_num) = cleaned_rank.parse::<f32>() {
                    println!("     - {} (第 {:.0} 名/250)", category, rank_num);
                }
            }
        }

        println!("\n==================== 数据完整性验证 ====================");

        // 验证基本数据结构完整性
        assert!(!result.title.is_empty(), "Title should not be empty");
        println!("✅ 标题字段验证通过");

        // 原始标题可能为空，这是正常的，只验证类型正确
        println!("✅ 原始标题字段验证通过 (Option类型支持可选值)");

        assert!(!result.genres.is_empty(), "Genres should not be empty");
        println!("✅ 类型字段验证通过 ({} 个标签)", result.genres.len());

        assert!(!result.studios.is_empty(), "Studios should not be empty");
        println!("✅ 制作商字段验证通过 ({} 个制作商)", result.studios.len());

        assert!(!result.actors.is_empty(), "Actors should not be empty");
        println!("✅ 演员字段验证通过 ({} 个演员)", result.actors.len());

        assert!(!result.posters.is_empty(), "Posters should not be empty");
        println!("✅ 海报字段验证通过 ({} 个海报)", result.posters.len());

        assert!(
            !result.preview_images.is_empty(),
            "Preview images should not be empty"
        );
        println!(
            "✅ 预览图片字段验证通过 ({} 个预览图)",
            result.preview_images.len()
        );

        // 注意：排名信息不是所有影片都有，所以不做强制验证
        if !result.ranking_numbers.is_empty() && !result.ranking_categories.is_empty() {
            assert_eq!(
                result.ranking_numbers.len(),
                result.ranking_categories.len(),
                "Ranking numbers and categories should have same length"
            );
            println!(
                "✅ TOP250 排名信息字段验证通过 ({} 个排名)",
                result.ranking_numbers.len()
            );
        } else {
            println!("ℹ️ 该影片暂无 TOP250 排名信息");
        }

        println!("\n🎉 所有验证通过！爬取测试成功完成！");
        println!("==================== 测试结束 ====================");
    }
}
