// tests/derive_works.rs

use crawler_template::{CrawlerData, CrawlerParseError,Crawler}; 
use std::collections::HashMap; 

#[derive(Crawler, Debug, PartialEq)]
struct IntegrationTestData {
    required_field: String,
    optional_field: Option<i32>,
    vec_field: Vec<u8>,
    option_vec_field: Option<Vec<String>>,
}

#[test]
fn test_crawler_derive_in_integration() {
    let mut map = HashMap::new();
    map.insert("required_field".to_string(), vec!["hello".to_string()]);
    map.insert("optional_field".to_string(), vec!["-10".to_string()]);
    map.insert(
        "vec_field".to_string(),
        vec!["1".to_string(), "255".to_string()],
    );

    let expected = IntegrationTestData {
        required_field: "hello".to_string(),
        optional_field: Some(-10),
        vec_field: vec![1, 255],
        option_vec_field: None,
    };

    let parsed = IntegrationTestData::parse(&map).expect("在集成测试中解析失败");
    assert_eq!(parsed, expected);

    let mut map_empty_opt_vec = HashMap::new();
    map_empty_opt_vec.insert("required_field".to_string(), vec!["empty_test".to_string()]);
    map_empty_opt_vec.insert("option_vec_field".to_string(), vec![]); 

    let expected_empty_opt_vec = IntegrationTestData {
        required_field: "empty_test".to_string(),
        optional_field: None,
        vec_field: vec![],
        option_vec_field: None, 
    };
    let parsed_empty_opt_vec =
        IntegrationTestData::parse(&map_empty_opt_vec).expect("解析空的 Option<Vec> 失败");
    assert_eq!(parsed_empty_opt_vec, expected_empty_opt_vec);

    let mut map_present_opt_vec = HashMap::new();
    map_present_opt_vec.insert(
        "required_field".to_string(),
        vec!["present_test".to_string()],
    );
    map_present_opt_vec.insert(
        "option_vec_field".to_string(),
        vec!["a".to_string(), "b".to_string()],
    );

    let expected_present_opt_vec = IntegrationTestData {
        required_field: "present_test".to_string(),
        optional_field: None,
        vec_field: vec![],
        option_vec_field: Some(vec!["a".to_string(), "b".to_string()]),
    };
    let parsed_present_opt_vec =
        IntegrationTestData::parse(&map_present_opt_vec).expect("解析存在的 Option<Vec> 失败");
    assert_eq!(parsed_present_opt_vec, expected_present_opt_vec);
}

#[test]
fn test_missing_required_field() {
    let map: HashMap<String, Vec<String>> = HashMap::new(); // 空 map
    let result = IntegrationTestData::parse(&map);
    // 使用 assert! 和 matches! 来检查错误类型和内容
    assert!(
        matches!(result, Err(CrawlerParseError::MissingField(field)) if field == "required_field")
    );
}

#[test]
fn test_conversion_error() {
    let mut map = HashMap::new();
    map.insert("required_field".to_string(), vec!["hello".to_string()]);
    map.insert(
        "optional_field".to_string(),
        vec!["not a number".to_string()],
    ); 
    let result = IntegrationTestData::parse(&map);
    assert!(
        matches!(result, Err(CrawlerParseError::ConversionFailed(field)) if field == "optional_field")
    );
}
