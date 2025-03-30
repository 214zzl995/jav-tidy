use std::{
    collections::{HashMap, HashSet},
    marker::PhantomData,
};

use crate::script::Rule;
use scraper::ElementRef;
use script::CrawlerScript;
use serde::{Deserialize, Deserializer};

pub use error::CrawlerErr;

mod error;
mod script;

#[derive(Debug, Clone)]
pub struct Template<T>
where
    T: CrawlerData + Default + Send,
{
    entrypoint: String,
    resource_type: PhantomData<T>,
    parameters: RuntimeVariable,
    workflows: Vec<WorkflowRoot>,
}

#[derive(Debug, Clone)]
struct CrawlerNode {
    _script_raw: String,
    request: bool,
    children: Option<HashMap<String, CrawlerNode>>,
    script: CrawlerScript,
}

#[derive(Debug, Clone)]
struct WorkflowRoot {
    url_key: String,
    node: Vec<WorkflowNode>,
}

#[derive(Debug, Clone)]
struct WorkflowNode {
    name: String,
    script: CrawlerScript,
    children: Vec<WorkflowNode>,
}

type RuntimeVariable = HashMap<String, Vec<String>>;

pub trait CrawlerData
where
    Self: Default,
{
    fn try_set(&mut self, field: &str, values: Vec<String>) -> Result<(), CrawlerErr>;
}

impl<T> Template<T>
where
    T: CrawlerData + Default + Send,
{
    pub fn from_yaml(yaml: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(yaml)
    }
    pub fn add_parameters(&mut self, key: &str, value: &str) {
        self.parameters
            .insert(key.to_string(), vec![value.to_string()]);
    }

    fn get_start_parameters(&self) -> RuntimeVariable {
        self.parameters
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    pub async fn crawler(&self) -> Result<T, CrawlerErr> {
        let mut value = T::default();
        let mut runtime_variable = self.get_start_parameters();

        for (index, workflow) in self.workflows.iter().enumerate() {
            let urls = if index == 0 {
                vec![self.build_entrypoint_url()?]
            } else {
                runtime_variable
                    .get(&workflow.url_key)
                    .unwrap_or(&vec![])
                    .iter()
                    .cloned()
                    .collect::<Vec<String>>()
            };

            for url in urls {
                workflow
                    .crawler(&url, &mut value, &mut runtime_variable)
                    .await?;
            }
        }

        Ok(value)
    }

    pub fn crawler_block(&self) -> Result<T, CrawlerErr> {
        let mut value = T::default();
        let mut runtime_variable = self.get_start_parameters();

        for (index, workflow) in self.workflows.iter().enumerate() {
            let urls = if index == 0 {
                vec![self.build_entrypoint_url()?]
            } else {
                runtime_variable
                    .get(&workflow.url_key)
                    .unwrap_or(&vec![])
                    .iter()
                    .cloned()
                    .collect::<Vec<String>>()
            };
            for url in urls {
                workflow
                    .crawler_blocking(&url, &mut value, &mut runtime_variable)
                    .unwrap();
            }
        }

        Ok(value)
    }

    fn build_entrypoint_url(&self) -> Result<String, CrawlerErr> {
        let mut entrypoint = self.entrypoint.to_string();
        for (key, values) in self.parameters.iter() {
            if values.is_empty() {
                return Err(CrawlerErr::DynNoValidData(key.clone()));
            }
            if values.len() > 1 {
                return Err(CrawlerErr::MultipleEntrypointParameterError(key.clone()));
            }
            let value = values[0].clone();
            entrypoint = entrypoint.replace(&format!("${{{}}}", key), &value);
        }
        Ok(entrypoint)
    }
}

impl WorkflowRoot {
    async fn crawler<'a, T>(
        &'a self,
        url: &str,
        value: &'a mut T,
        runtime_variable: &'a mut RuntimeVariable,
    ) -> Result<(), CrawlerErr>
    where
        T: CrawlerData + Default,
    {
        let root_html = {
            let response = reqwest::get(url).await?;
            let body = response.text().await?;
            scraper::Html::parse_document(&body)
        };

        let root_element_refs = vec![root_html.root_element()];

        for node in &self.node {
            node.process(root_element_refs.clone(), value, runtime_variable)?;
        }

        Ok(())
    }

    fn crawler_blocking<'a, T>(
        &'a self,
        url: &str,
        value: &'a mut T,
        runtime_variable: &'a mut RuntimeVariable,
    ) -> Result<(), CrawlerErr>
    where
        T: CrawlerData + Default,
    {
        let root_html = {
            let response = reqwest::blocking::get(url)?;
            let body = response.text()?;
            scraper::Html::parse_document(&body)
        };

        let root_element_refs = vec![root_html.root_element()];

        for node in &self.node {
            node.process(root_element_refs.clone(), value, runtime_variable)?;
        }

        Ok(())
    }

    fn new(url_key: &str, node: HashMap<String, CrawlerNode>) -> Self {
        let node = node
            .into_iter()
            .map(|node| node.into())
            .collect::<Vec<WorkflowNode>>();
        WorkflowRoot {
            url_key: url_key.to_string(),
            node,
        }
    }
}

impl WorkflowNode {
    fn process<T>(
        &self,
        root_element_refs: Vec<ElementRef<'_>>,
        value: &mut T,
        runtime_variable: &mut RuntimeVariable,
    ) -> Result<(), CrawlerErr>
    where
        T: CrawlerData + Default,
    {
        match self.script.rule {
            Rule::element_access => {
                let elements = self
                    .script
                    .get_elements(root_element_refs, runtime_variable)?;

                for node in &self.children {
                    node.process(elements.clone(), value, runtime_variable)?;
                }
            }
            Rule::value_access => {
                let values = self
                    .script
                    .get_values(root_element_refs, runtime_variable)?;

                value.try_set(&self.name, values.clone())?;

                if !runtime_variable.contains_key(&self.name) {
                    runtime_variable.insert(self.name.clone(), values.clone());
                }
                runtime_variable
                    .get_mut(&self.name)
                    .unwrap()
                    .extend(values.clone());
            }
            _ => {}
        };

        Ok(())
    }
}

impl CrawlerNode {
    fn _get_all_fields(&self, node_name: String) -> Vec<String> {
        let mut fields = Vec::new();
        if Rule::value_access == self.script.rule {
            fields.push(node_name);
        }

        if let Some(children) = &self.children {
            children.iter().for_each(|(name, node)| {
                fields.append(&mut node._get_all_fields(name.clone()));
            });
        }

        fields
    }
}

fn default_false() -> bool {
    false
}

impl<'de, T> Deserialize<'de> for Template<T>
where
    T: CrawlerData + Default + Send,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize, Clone)]
        struct TemplateData {
            entrypoint: String,
            nodes: HashMap<String, CrawlerNode>,
            env: Option<RuntimeVariable>,
        }

        fn check_tree_keys_unique(nodes: &HashMap<String, CrawlerNode>) -> Result<(), String> {
            let mut global_keys = HashSet::new();
            check_node_keys(nodes, &mut global_keys)
        }

        fn check_node_keys(
            nodes: &HashMap<String, CrawlerNode>,
            seen_keys: &mut HashSet<String>,
        ) -> Result<(), String> {
            for (key, node) in nodes {
                if !seen_keys.insert(key.clone()) {
                    return Err(format!("Duplicate key '{}' found in tree", key));
                }

                if let Some(children) = &node.children {
                    check_node_keys(children, seen_keys)?;
                }
            }
            Ok(())
        }

        let data = TemplateData::deserialize(deserializer)?;

        check_tree_keys_unique(&data.nodes)
            .map_err(|e| serde::de::Error::custom(format!("Duplicate key error: {}", e)))?;

        let root_node = WorkflowRoot::new("", data.nodes.clone());

        let mut workflow = vec![root_node];

        fn collect_requested_nodes(
            node_map: &HashMap<String, CrawlerNode>,
            collected_nodes: &mut Vec<WorkflowRoot>,
        ) {
            for (name, node) in node_map {
                if node.request {
                    collected_nodes.push((name.clone(), node.clone()).into());
                } else if let Some(children) = &node.children {
                    collect_requested_nodes(children, collected_nodes);
                }
            }
        }

        collect_requested_nodes(&data.nodes, &mut workflow);

        Ok(Template {
            entrypoint: data.entrypoint,
            parameters: data.env.unwrap_or_default(),
            workflows: workflow,
            resource_type: PhantomData,
        })
    }
}

impl<'de> Deserialize<'de> for CrawlerNode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum CrawlerNodeData {
            Complex {
                script: String,
                #[serde(default = "crate::default_false")]
                request: bool,
                #[serde(default)]
                children: Option<HashMap<String, CrawlerNode>>,
            },
            Simple(String),
        }

        let data = CrawlerNodeData::deserialize(deserializer)?;

        let (script_raw, request, children) = match data {
            CrawlerNodeData::Complex {
                script,
                request,
                children,
            } => (script, request, children),
            CrawlerNodeData::Simple(script) => (script, false, None),
        };

        let script = match CrawlerScript::new(&script_raw) {
            Ok(script) => script,
            Err(e) => return Err(serde::de::Error::custom(e.to_string())),
        };

        if script.rule == Rule::value_access
            && matches!(children.as_ref(), Some(c) if !c.is_empty())
            && !request
        {
            return Err(serde::de::Error::custom(
                "Element access is not allowed in the root node",
            ));
        }

        Ok(CrawlerNode {
            _script_raw: script_raw,
            request,
            children,
            script,
        })
    }
}

type WorkflowNodeWithName = (String, CrawlerNode);

impl From<WorkflowNodeWithName> for WorkflowRoot {
    fn from(node: WorkflowNodeWithName) -> Self {
        WorkflowRoot {
            url_key: node.0.clone(),
            node: node.1.children.clone().map_or(vec![], |c| {
                c.into_iter()
                    .map(|node| node.into())
                    .collect::<Vec<WorkflowNode>>()
            }),
        }
    }
}

impl From<WorkflowNodeWithName> for WorkflowNode {
    fn from(node: WorkflowNodeWithName) -> Self {
        WorkflowNode {
            name: node.0,
            script: node.1.script,
            children: node.1.children.clone().map_or(vec![], |c| {
                if node.1.request {
                    vec![]
                } else {
                    c.into_iter()
                        .map(|node| node.into())
                        .collect::<Vec<WorkflowNode>>()
                }
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default, Debug)]
    struct Movie {
        title: String,
        thumbnail: Option<String>,
        detail_url: Option<String>,
        tags: Option<Vec<String>>,
        actors: Vec<String>,
    }

    impl CrawlerData for Movie {
        fn try_set(&mut self, field: &str, values: Vec<String>) -> Result<(), CrawlerErr> {
            match field {
                "title" => {
                    if values.len() != 1 {
                        return Err(CrawlerErr::InvalidValueCount(
                            "title".to_string(),
                            values.len(),
                        ));
                    }
                    let value = values[0].parse::<String>().map_err(|err| {
                        CrawlerErr::ParseError("title".to_string(), err.to_string())
                    })?;
                    self.title = value;
                    Ok(())
                }
                "thumbnail" => {
                    if values.len() > 1 {
                        return Err(CrawlerErr::InvalidValueCount(
                            "thumbnail".to_string(),
                            values.len(),
                        ));
                    }
                    self.thumbnail = if let Some(v) = values.first() {
                        Some(v.parse::<String>().map_err(|err| {
                            CrawlerErr::ParseError("thumbnail".to_string(), err.to_string())
                        })?)
                    } else {
                        None
                    };
                    Ok(())
                }
                "detail_url" => {
                    if values.len() > 1 {
                        return Err(CrawlerErr::InvalidValueCount(
                            "detail_url".to_string(),
                            values.len(),
                        ));
                    }
                    self.detail_url = if let Some(v) = values.first() {
                        Some(v.parse::<String>().map_err(|err| {
                            CrawlerErr::ParseError("detail_url".to_string(), err.to_string())
                        })?)
                    } else {
                        None
                    };
                    Ok(())
                }
                _ => Ok(()),
            }
        }
    }

    const SAMPLE_YAML: &str = include_str!("../template/sample.yaml");

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
                .mock("GET", "/movies?page=1")
                .with_status(200)
                .with_body(
                    r#"
                    <div class="movie-list">
                        <div class="video-title"><strong>TEST-MOVIE</strong></div>
                        <img src="thumbnail.jpg">
                        <a href="detail/123"></a>
                    </div>
                    "#,
                )
                .create();

            let _m2 = server
                .mock("GET", "/detail/123")
                .with_status(200)
                .with_body("<div class='detail'>...</div>")
                .create();

            let mut template = Template::<Movie>::from_yaml(SAMPLE_YAML).unwrap();

            template.add_parameters("base_url", &url);
            template.add_parameters("crawl_name", "TEST-MOVIE");

            let result = template.crawler().await.unwrap();

            assert_eq!(result.title, "TEST-MOVIE"); 
            assert_eq!(result.thumbnail, Some("thumbnail.jpg".to_string()));
            assert_eq!(result.detail_url, Some("https://cdn.example.comdetail/123".to_string()));
        });
    }

    #[test]
    fn test_invalid_script() {
        let yaml = r#"
            entrypoint: "invalid"
            nodes:
              main:
                script: "invalid[selector"
        "#;

        let result = Template::<Movie>::from_yaml(yaml);
        assert!(matches!(result, Err(serde_yaml::Error { .. })));
    }
}
