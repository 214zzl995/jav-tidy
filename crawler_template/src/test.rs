#[cfg(test)]
mod tests {
    use crate::Template;

    #[derive(Default, Debug)]
    struct Movie {
        title: String,
        thumbnail: Option<String>,
        detail_url: Option<String>,
        tags: Option<Vec<String>>,
        actors: Vec<String>,
    }

    const SAMPLE_YAML: &str = include_str!("../template/sample.yaml");

    const SAMPLE_SEARCH: &str = include_str!("../template/sample_search.html");

    const SAMPLE_DETAIL: &str = include_str!("../template/sample_detail.html");

    impl crate::CrawlerData for Movie {
        type Error = crate::CrawlerParseError;

        fn parse(
            map: &std::collections::HashMap<String, Vec<String>>,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                title: {
                    map.get("title")
                        .and_then(|v| v.first())
                        .ok_or(crate::CrawlerParseError::MissingField("title"))
                        .and_then(|s| {
                            s.parse()
                                .map_err(|_| crate::CrawlerParseError::ConversionFailed("title"))
                        })?
                },
                thumbnail: {
                    map.get("thumbnail")
                        .and_then(|v| v.first())
                        .map(|s| s.parse())
                        .transpose()
                        .map_err(|_| crate::CrawlerParseError::ConversionFailed("thumbnail"))?
                },
                detail_url: {
                    map.get("detail_url")
                        .and_then(|v| v.first())
                        .map(|s| s.parse())
                        .transpose()
                        .map_err(|_| crate::CrawlerParseError::ConversionFailed("detail_url"))?
                },
                tags: {
                    map.get("tags")
                        .map(|values| {
                            values
                                .iter()
                                .map(|s| s.parse())
                                .collect::<Result<Vec<_>, _>>()
                                .map(Some)
                        })
                        .transpose()
                        .map_err(|_| crate::CrawlerParseError::ConversionFailed("tags"))?
                        .flatten()
                },
                actors: {
                    map.get("actors")
                        .map(|values| {
                            values
                                .iter()
                                .map(|s| s.parse())
                                .collect::<Result<Vec<_>, _>>()
                        })
                        .unwrap_or(Ok(Vec::new()))
                        .map_err(|_| crate::CrawlerParseError::ConversionFailed("actors"))?
                },
            })
        }
    }

    #[test]
    fn test_workflow_format() {
        let mut template = Template::<Movie>::from_yaml(SAMPLE_YAML).unwrap();
        template.add_parameters("crawl_name", "TEST-001");
        template.add_parameters("base_url", "https://example.com");

        assert_eq!(
            template.build_entrypoint_url().unwrap(),
            "https://example.com/search?q=TEST-001&f=all"
        );
        assert_eq!(template.workflows.len(), 2);
    }

    #[test]
    fn test_workflow_execution() {
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async move {
            let mut server = mockito::Server::new_async().await;

            let url = server.url();

            let _m = server
                .mock("GET", "/search?q=TEST-MOVIE1&f=all")
                .with_status(200)
                .with_body(SAMPLE_SEARCH)
                .create();

            let _m2 = server
                .mock("GET", "/detail/1")
                .with_status(200)
                .with_body(SAMPLE_DETAIL)
                .create();

            let mut template = Template::<Movie>::from_yaml(SAMPLE_YAML).unwrap();

            template.add_parameters("base_url", &url);
            template.add_parameters("crawl_name", "TEST-MOVIE1");

            let result = template.crawler().await.unwrap();

            assert_eq!(result.title, "TEST-MOVIE1 的title");
            assert_eq!(
                result.thumbnail,
                Some("https://cdn.example.com/111.jpg".to_string())
            );
            assert_eq!(result.detail_url, Some(format!("{}/detail/1", url)));
            assert_eq!(
                result.tags,
                Some(vec![
                    "Tag1".to_string(),
                    "Tag2".to_string(),
                    "Tag3".to_string(),
                    "Tag4".to_string(),
                    "Tag5".to_string(),
                    "Tag6".to_string(),
                    "Tag7".to_string()
                ])
            );
            assert_eq!(result.actors, vec!["演员1".to_string(),]);
        });
    }
}
