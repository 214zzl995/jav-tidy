// 库入口文件，用于导出公共 API 给测试使用

pub mod config;
pub mod crawler;
pub mod error;
pub mod file;
pub mod file_organizer;
pub mod image_manager;
pub mod nfo;
pub mod nfo_generator;
pub mod parser;
pub mod template_parser;
pub mod translator;

// 测试模块
#[cfg(test)]
mod tests;
