use std::collections::HashMap;
use crate::nfo::MovieNfo;
use anyhow::{anyhow, Result};
use regex::Regex;

/// 文件命名模板解析器
#[derive(Debug, Clone)]
pub struct TemplateParser {
    /// 可用的模板变量映射
    variables: HashMap<String, String>,
}

/// 多演员处理策略
#[derive(Debug, Clone, PartialEq)]
pub enum MultiActorStrategy {
    /// 创建硬链接到每个演员目录
    HardLink,
    /// 创建符号链接到每个演员目录
    SymLink,
    /// 只使用第一个演员
    FirstOnly,
    /// 合并所有演员名称
    Merge,
}

impl MultiActorStrategy {
    pub fn from_string(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "hardlink" => Self::HardLink,
            "symlink" => Self::SymLink,
            "first_only" | "first" => Self::FirstOnly,
            "merge" => Self::Merge,
            _ => Self::SymLink, // 默认使用符号链接
        }
    }
}

/// 模板解析结果
#[derive(Debug, Clone)]
pub struct ParseResult {
    /// 主要路径（使用第一个演员或合并演员名称）
    pub primary_path: String,
    /// 额外路径（当策略为HardLink或SymLink时的其他演员路径）
    pub additional_paths: Vec<String>,
    /// 使用的策略
    #[allow(dead_code)]
    pub strategy: MultiActorStrategy,
}

impl TemplateParser {
    /// 创建新的模板解析器
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    /// 从NFO数据填充模板变量
    pub fn populate_from_nfo(&mut self, nfo: &MovieNfo) -> Result<()> {
        // 基本信息
        self.variables.insert("title".to_string(), 
            if nfo.title.is_empty() { "Unknown".to_string() } else { nfo.title.clone() });
        
        self.variables.insert("original_title".to_string(), 
            if nfo.original_title.is_empty() { 
                if nfo.title.is_empty() { "Unknown".to_string() } else { nfo.title.clone() }
            } else { 
                nfo.original_title.clone() 
            });
        
        self.variables.insert("year".to_string(), 
            nfo.year.map(|y| y.to_string()).unwrap_or_else(|| "Unknown".to_string()));
        
        // 系列信息
        if let Some(set) = &nfo.set {
            self.variables.insert("series".to_string(), set.name.clone());
        } else {
            self.variables.insert("series".to_string(), "".to_string());
        }
        
        // 演员信息（第一个演员）
        if !nfo.actors.is_empty() {
            self.variables.insert("actor".to_string(), nfo.actors[0].name.clone());
            // 保存所有演员用于多演员处理
            let all_actors: Vec<String> = nfo.actors.iter().map(|a| a.name.clone()).collect();
            self.variables.insert("all_actors".to_string(), all_actors.join(","));
        } else {
            self.variables.insert("actor".to_string(), "Unknown".to_string());
            self.variables.insert("all_actors".to_string(), "".to_string());
        }
        
        // 导演信息
        if !nfo.directors.is_empty() {
            self.variables.insert("director".to_string(), nfo.directors[0].clone());
        } else {
            self.variables.insert("director".to_string(), "Unknown".to_string());
        }
        
        // 制片厂信息
        if !nfo.studios.is_empty() {
            self.variables.insert("studio".to_string(), nfo.studios[0].clone());
        } else {
            self.variables.insert("studio".to_string(), "Unknown".to_string());
        }
        
        // 类型信息（第一个类型）
        if !nfo.genres.is_empty() {
            self.variables.insert("genre".to_string(), nfo.genres[0].clone());
        } else {
            self.variables.insert("genre".to_string(), "Unknown".to_string());
        }
        
        // ID信息（使用IMDB ID或第一个unique ID）
        if !nfo.imdb_id.is_empty() {
            self.variables.insert("id".to_string(), nfo.imdb_id.clone());
        } else if !nfo.unique_ids.is_empty() {
            self.variables.insert("id".to_string(), nfo.unique_ids[0].value.clone());
        } else {
            self.variables.insert("id".to_string(), "Unknown".to_string());
        }

        Ok(())
    }

    /// 解析模板字符串，返回解析结果
    pub fn parse_template(&self, template: &str, strategy: MultiActorStrategy) -> Result<ParseResult> {
        // 创建正则表达式来匹配 $variable$ 格式的变量
        let re = Regex::new(r"\$([a-zA-Z_][a-zA-Z0-9_]*)\$")
            .map_err(|e| anyhow!("正则表达式创建失败: {}", e))?;
        
        // 处理主要路径（使用第一个演员或合并演员）
        let primary_path = self.replace_variables(template, &re, &strategy)?;
        
        // 处理额外路径（多个演员的情况）
        let additional_paths = self.generate_additional_paths(template, &re, &strategy)?;
        
        Ok(ParseResult {
            primary_path,
            additional_paths,
            strategy,
        })
    }

    /// 替换模板中的变量
    fn replace_variables(&self, template: &str, re: &Regex, strategy: &MultiActorStrategy) -> Result<String> {
        let mut result = template.to_string();
        
        for cap in re.captures_iter(template) {
            let var_name = &cap[1];
            let placeholder = &cap[0]; // 包含 $ 的完整占位符
            
            let replacement = if var_name == "actor" {
                match strategy {
                    MultiActorStrategy::Merge => {
                        // 合并所有演员名称
                        if let Some(all_actors) = self.variables.get("all_actors") {
                            if all_actors.is_empty() {
                                "Unknown".to_string()
                            } else {
                                all_actors.replace(",", " & ")
                            }
                        } else {
                            "Unknown".to_string()
                        }
                    },
                    _ => {
                        // 其他策略使用第一个演员
                        self.variables.get("actor").unwrap_or(&"Unknown".to_string()).clone()
                    }
                }
            } else {
                self.variables.get(var_name).ok_or_else(|| {
                    anyhow!("未知的模板变量: ${}", var_name)
                })?.clone()
            };
            
            // 清理文件名中的非法字符
            let clean_replacement = self.sanitize_filename(&replacement);
            result = result.replace(placeholder, &clean_replacement);
        }
        
        Ok(result)
    }

    /// 生成额外的路径（用于多演员链接）
    fn generate_additional_paths(&self, template: &str, re: &Regex, strategy: &MultiActorStrategy) -> Result<Vec<String>> {
        if !matches!(strategy, MultiActorStrategy::HardLink | MultiActorStrategy::SymLink) {
            return Ok(vec![]);
        }
        
        // 检查模板是否包含 $actor$ 变量
        if !template.contains("$actor$") {
            return Ok(vec![]);
        }
        
        // 获取所有演员
        let empty_string = "".to_string();
        let all_actors = self.variables.get("all_actors").unwrap_or(&empty_string);
        if all_actors.is_empty() {
            return Ok(vec![]);
        }
        
        let actors: Vec<&str> = all_actors.split(',').collect();
        if actors.len() <= 1 {
            return Ok(vec![]);
        }
        
        // 为除第一个演员外的每个演员生成路径
        let mut additional_paths = Vec::new();
        for actor in actors.iter().skip(1) {
            let mut temp_variables = self.variables.clone();
            temp_variables.insert("actor".to_string(), actor.trim().to_string());
            
            let temp_parser = TemplateParser { variables: temp_variables };
            let path = temp_parser.replace_variables(template, re, &MultiActorStrategy::FirstOnly)?;
            additional_paths.push(path);
        }
        
        Ok(additional_paths)
    }

    /// 清理文件名中的非法字符
    fn sanitize_filename(&self, filename: &str) -> String {
        // 移除或替换文件名中的非法字符
        let illegal_chars = ['<', '>', ':', '"', '|', '?', '*'];
        let mut sanitized = filename.to_string();
        
        for char in illegal_chars {
            sanitized = sanitized.replace(char, "");
        }
        
        // 替换路径分隔符（在Windows下）
        sanitized = sanitized.replace('\\', "");
        
        // 移除多余的空格
        sanitized = sanitized.trim().to_string();
        
        // 如果结果为空，返回默认值
        if sanitized.is_empty() {
            "Unknown".to_string()
        } else {
            sanitized
        }
    }

    /// 获取所有可用的模板变量列表
    #[allow(dead_code)]
    pub fn get_available_variables() -> Vec<&'static str> {
        vec![
            "title",         // 影片标题
            "original_title", // 原始标题
            "year",          // 年份
            "series",        // 系列名
            "actor",         // 演员（第一个或合并）
            "director",      // 导演
            "studio",        // 制片厂
            "genre",         // 类型（第一个）
            "id",            // 影片ID
        ]
    }
}

impl Default for TemplateParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nfo::{MovieNfo, Actor, MovieSet};

    #[allow(clippy::field_reassign_with_default)]
    fn create_test_nfo() -> MovieNfo {
        let mut nfo = MovieNfo::default();
        nfo.title = "测试电影".to_string();
        nfo.original_title = "Test Movie".to_string();
        nfo.year = Some(2023);
        nfo.studios = vec!["Test Studio".to_string()];
        
        // 添加系列信息
        nfo.set = Some(MovieSet {
            name: "测试系列".to_string(),
            overview: "测试系列简介".to_string(),
        });
        
        // 添加演员信息
        nfo.actors = vec![
            Actor {
                name: "演员A".to_string(),
                role: "主角".to_string(),
                thumb: "".to_string(),
                order: Some(1),
            },
            Actor {
                name: "演员B".to_string(),
                role: "配角".to_string(),
                thumb: "".to_string(),
                order: Some(2),
            },
        ];
        
        nfo.directors = vec!["导演A".to_string()];
        nfo.genres = vec!["动作".to_string(), "冒险".to_string()];
        
        nfo
    }

    #[test]
    fn test_template_parser_basic() {
        let mut parser = TemplateParser::new();
        let nfo = create_test_nfo();
        
        parser.populate_from_nfo(&nfo).unwrap();
        
        let result = parser.parse_template(
            "$series$/$title$ ($year$)", 
            MultiActorStrategy::FirstOnly
        ).unwrap();
        
        assert_eq!(result.primary_path, "测试系列/测试电影 (2023)");
        assert!(result.additional_paths.is_empty());
    }

    #[test]
    fn test_template_parser_with_actor() {
        let mut parser = TemplateParser::new();
        let nfo = create_test_nfo();
        
        parser.populate_from_nfo(&nfo).unwrap();
        
        let result = parser.parse_template(
            "$actor$/$title$ ($year$)", 
            MultiActorStrategy::FirstOnly
        ).unwrap();
        
        assert_eq!(result.primary_path, "演员A/测试电影 (2023)");
        assert!(result.additional_paths.is_empty());
    }

    #[test]
    fn test_template_parser_symlink_strategy() {
        let mut parser = TemplateParser::new();
        let nfo = create_test_nfo();
        
        parser.populate_from_nfo(&nfo).unwrap();
        
        let result = parser.parse_template(
            "$actor$/$title$ ($year$)", 
            MultiActorStrategy::SymLink
        ).unwrap();
        
        assert_eq!(result.primary_path, "演员A/测试电影 (2023)");
        assert_eq!(result.additional_paths.len(), 1);
        assert_eq!(result.additional_paths[0], "演员B/测试电影 (2023)");
    }

    #[test]
    fn test_template_parser_merge_strategy() {
        let mut parser = TemplateParser::new();
        let nfo = create_test_nfo();
        
        parser.populate_from_nfo(&nfo).unwrap();
        
        let result = parser.parse_template(
            "$actor$/$title$ ($year$)", 
            MultiActorStrategy::Merge
        ).unwrap();
        
        assert_eq!(result.primary_path, "演员A & 演员B/测试电影 (2023)");
        assert!(result.additional_paths.is_empty());
    }

    #[test]
    fn test_available_variables() {
        let vars = TemplateParser::get_available_variables();
        assert!(vars.contains(&"title"));
        assert!(vars.contains(&"actor"));
        assert!(vars.contains(&"year"));
        assert!(vars.contains(&"series"));
    }
}