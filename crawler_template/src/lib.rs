use std::{collections::HashMap, marker::PhantomData};

use crate::script::Rule;
use scraper::ElementRef;
use script::CrawlerScript;
use serde::{Deserialize, Deserializer, Serialize};

pub use error::CrawlerErr;

mod error;
mod script;

#[derive(Debug, Clone, Serialize)]
pub struct Template<T>
where
    T: CrawlerData + Default + Send,
{
    nodes: HashMap<String, CrawlerNode>,
    #[serde(skip)]
    resource_type: PhantomData<T>,
    #[serde(skip)]
    parameters: HashMap<String, Vec<String>>,
    #[serde(skip)]
    workflow: Vec<WorkflowRoot>,
}

#[derive(Debug, Clone, Serialize)]
struct CrawlerNode {
    #[serde(rename = "script")]
    script_raw: String,
    #[serde(default = "crate::default_false", skip_serializing_if = "is_false")]
    request: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    children: Option<HashMap<String, CrawlerNode>>,
    #[serde(skip)]
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

    pub(crate) fn _get_all_fields(&self) -> Vec<String> {
        let mut fields = Vec::new();
        self.nodes.iter().for_each(|(name, node)| {
            fields.append(&mut node._get_all_fields(name.clone()));
        });

        fields
    }

    fn get_start_parameters(&self) -> HashMap<String, Vec<String>> {
        let mut map = HashMap::new();
        self.parameters.clone().into_iter().for_each(|(k, v)| {
            map.insert(k, v);
        });

        map
    }

    pub async fn crawler(&self, url: &str) -> Result<T, CrawlerErr> {
        let mut value = T::default();
        let mut runtime_variable = self.get_start_parameters();

        for (index, root) in self.workflow.iter().enumerate() {
            let url = if index == 0 {
                url.to_string()
            } else {
                runtime_variable
                    .get(&root.url_key)
                    .unwrap()
                    .first()
                    .unwrap()
                    .clone()
            };

            root.crawler(&url, &mut value, &mut runtime_variable)
                .await?;
        }

        Ok(value)
    }

    pub fn crawler_block(&self, url: &str) -> Result<T, CrawlerErr> {
        let mut value = T::default();
        let mut runtime_variable = self.get_start_parameters();

        for (index, root) in self.workflow.iter().enumerate() {
            let url = if index == 0 {
                url.to_string()
            } else {
                runtime_variable
                    .get(&root.url_key)
                    .unwrap()
                    .first()
                    .unwrap()
                    .clone()
            };

            root.crawler_blocking(&url, &mut value, &mut runtime_variable)
                .unwrap();
        }

        Ok(value)
    }
}

impl WorkflowRoot {
    async fn crawler<'a, T>(
        &'a self,
        url: &str,
        value: &'a mut T,
        runtime_variable: &'a mut HashMap<String, Vec<String>>,
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
            node.crawler(root_element_refs.clone(), value, runtime_variable)?;
        }

        Ok(())
    }

    fn crawler_blocking<'a, T>(
        &'a self,
        url: &str,
        value: &'a mut T,
        runtime_variable: &'a mut HashMap<String, Vec<String>>,
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
            node.crawler(root_element_refs.clone(), value, runtime_variable)?;
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
    fn crawler<'a, T>(
        &'a self,
        root_element_refs: Vec<ElementRef<'a>>,
        value: &'a mut T,
        runtime_variable: &'a mut HashMap<String, Vec<String>>,
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
                    node.crawler(elements.clone(), value, runtime_variable)?;
                }
            }
            Rule::value_access => {
                let values = self
                    .script
                    .get_values(root_element_refs, runtime_variable)?;
                value.try_set(&self.name, values.clone())?;
                runtime_variable.insert(self.name.clone(), values);
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

fn _default_false() -> bool {
    false
}

fn is_false(value: &bool) -> bool {
    !value
}

impl<'de, T> Deserialize<'de> for Template<T>
where
    T: CrawlerData + Default + Send,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct TemplateData {
            nodes: HashMap<String, CrawlerNode>,
            parameters: Option<HashMap<String, Vec<String>>>,
        }

        let data = TemplateData::deserialize(deserializer)?;

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
            nodes: data.nodes,
            parameters: data.parameters.unwrap_or_default(),
            workflow,
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
        struct CrawlerNodeData {
            script: String,
            request: Option<bool>,
            children: Option<HashMap<String, CrawlerNode>>,
        }

        let data = CrawlerNodeData::deserialize(deserializer)?;

        let script = match CrawlerScript::new(&data.script, false) {
            Ok(script) => script,
            Err(e) => return Err(serde::de::Error::custom(e.to_string())),
        };

        if script.rule == Rule::value_access
            && data.children.clone().map_or(false, |c| !c.is_empty())
            && !data.request.unwrap_or(true)
        {
            return Err(serde::de::Error::custom(
                "Element access is not allowed in the root node",
            ));
        }

        Ok(CrawlerNode {
            script_raw: data.script,
            request: data.request.unwrap_or(false),
            children: data.children,
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
