use std::{
    collections::HashMap,
    fmt::{Debug, Display},
};

use pest::Parser;
use pest_derive::Parser;
use regex::Regex;
use scraper::{ElementRef, Selector};

use crate::error::CrawlerErr;

#[derive(Parser)]
#[grammar = "../script.pest"]
pub struct ScriptParser;

#[derive(Debug, Clone)]
pub struct CrawlerScript {
    _raw: String,
    commands: Vec<Command>,
    pub(crate) rule: Rule,
}

#[derive(Debug, Clone)]
enum Command {
    Selector(Param),
    Parent(usize),
    Prev(usize),
    Nth(usize),
    Replace(Param, Param),
    Uppercase,
    Lowercase,
    Insert(usize, Param),
    Prepend(Param),
    Append(Param),
    Delete(Param),
    RegexMatch(Param),
    Equals(Param),
    Html,
    Attr(Param),
    Val,
    RegexExtract(Param),
    RegexReplace(Param, Param),
}

#[derive(Debug, Clone, PartialEq)]
enum Param {
    StaticStr(String),
    DynamicStr(String),
}

impl Param {
    pub(crate) fn get_value(
        &self,
        runtime_variable: &HashMap<String, Vec<String>>,
    ) -> Result<String, CrawlerErr> {
        match self {
            Param::StaticStr(param) => Ok(param.to_string()),
            Param::DynamicStr(param) => {
                let values = runtime_variable
                    .get(param)
                    .ok_or_else(|| CrawlerErr::FieldNotFound(param.to_string()))?;

                if values.is_empty() {
                    return Err(CrawlerErr::DynNoValidData(param.to_string()));
                }

                if values.len() > 1 {
                    return Err(CrawlerErr::DynMultipleResults(
                        param.to_string(),
                        values.join(","),
                    ));
                }

                if let Some(value) = values.first() {
                    Ok(value.to_string())
                } else {
                    Err(CrawlerErr::DynNotYetInitialised(param.to_string()))
                }
            }
        }
    }
}

impl CrawlerScript {
    pub fn new(script: &str, is_text_script: bool) -> Result<CrawlerScript, CrawlerErr> {
        let mut commands = Vec::new();
        let mut pairs = ScriptParser::parse(Rule::script, script)?;

        let script_pair = pairs.next().unwrap();
        let rule = script_pair.as_rule();

        match rule {
            Rule::text_access => {
                if !is_text_script {
                    return Err(CrawlerErr::CharProcessAlone);
                }
            }
            Rule::value_access => {}
            Rule::element_access => {}
            _ => {}
        }

        for pair in script_pair.into_inner() {
            fn get_commands(
                handle: fn(pest::iterators::Pair<Rule>) -> Result<Command, CrawlerErr>,
                pairs: pest::iterators::Pair<Rule>,
            ) -> Result<Vec<Command>, CrawlerErr> {
                let mut commands = Vec::new();
                for pair in pairs.into_inner() {
                    commands.push(handle(pair)?);
                }
                Ok(commands)
            }
            match pair.as_rule() {
                Rule::selector_rule => {
                    commands.append(&mut get_commands(parse_selector_rule, pair)?)
                }
                Rule::transform_rule => {
                    commands.append(&mut get_commands(parse_transform_rule, pair)?)
                }
                Rule::condition_rule => {
                    commands.append(&mut get_commands(parse_condition_rule, pair)?)
                }
                Rule::accessor_rule => {
                    commands.append(&mut get_commands(parse_accessor_rule, pair)?)
                }
                _ => {}
            }
        }

        Ok(CrawlerScript {
            _raw: script.to_string(),
            commands,
            rule,
        })
    }

    pub(crate) fn get_value_with_element<'a>(
        &self,
        root_element_ref: Vec<ElementRef<'a>>,
        runtime_variable: &mut HashMap<String, Vec<String>>,
    ) -> Result<Vec<(String, ElementRef<'a>)>, CrawlerErr> {
        let mut element_values: Vec<(String, ElementRef)> = root_element_ref
            .into_iter()
            .map(|element| (String::new(), element))
            .collect();

        for command in self.commands.clone() {
            match command {
                Command::Selector(selector) => {
                    let selector = selector.get_value(runtime_variable)?;

                    let selector = Selector::parse(&selector)
                        .map_err(|err| CrawlerErr::SelectorError(err.to_string()))?;

                    element_values = element_values
                        .into_iter()
                        .flat_map(|(_, element)| {
                            element
                                .select(&selector)
                                .map(|element| (String::new(), element))
                                .collect::<Vec<_>>()
                        })
                        .collect();

                    if element_values.is_empty() {
                        return Ok(vec![]);
                    }
                }
                Command::Parent(index) => {
                    for element_value in element_values.iter_mut() {
                        let mut r_parent = element_value.1;
                        for erg in 0..index {
                            r_parent = ElementRef::wrap(
                                r_parent
                                    .parent()
                                    .ok_or(CrawlerErr::ParentNodeOverflow(index, erg))?,
                            )
                            .unwrap();
                        }
                        element_value.1 = r_parent;
                    }
                }
                Command::Prev(index) => {
                    for element_value in element_values.iter_mut() {
                        let prev_siblings = element_value
                            .1
                            .clone()
                            .prev_siblings()
                            .filter_map(ElementRef::wrap)
                            .collect::<Vec<_>>();
                        let prev_siblings_len = prev_siblings.len();

                        if prev_siblings_len < index {
                            return Err(CrawlerErr::PrevNodeOverflow(index, prev_siblings_len));
                        }

                        element_value.1 = prev_siblings[index - 1];
                    }
                }
                Command::Nth(index) => {
                    for element_value in element_values.iter_mut() {
                        let next_siblings = element_value
                            .1
                            .clone()
                            .next_siblings()
                            .filter_map(ElementRef::wrap)
                            .collect::<Vec<_>>();
                        let next_siblings_len = next_siblings.len();

                        if next_siblings_len < index {
                            return Err(CrawlerErr::PrevNodeOverflow(index, next_siblings_len));
                        }

                        element_value.1 = next_siblings[index - 1];
                    }
                }
                Command::Html => {
                    element_values.iter_mut().for_each(|element_values| {
                        element_values.0 = element_values.1.html().to_string();
                    });
                }
                Command::Attr(attr) => {
                    let attr = attr.get_value(runtime_variable)?;
                    element_values.iter_mut().for_each(|value| {
                        value.0 = value.1.value().attr(&attr).unwrap_or("").to_string();
                    });
                }
                Command::Val => {
                    element_values.iter_mut().for_each(|value| {
                        value.0 = value.1.text().collect();
                    });
                }
                Command::Replace(from, to) => {
                    let from = from.get_value(runtime_variable)?;
                    let to = to.get_value(runtime_variable)?;
                    element_values.iter_mut().for_each(|element_value| {
                        element_value.0 = element_value.0.replace(&from, &to);
                    });
                }
                Command::Uppercase => {
                    element_values.iter_mut().for_each(|element_value| {
                        element_value.0 = element_value.0.to_uppercase();
                    });
                }
                Command::Lowercase => {
                    element_values.iter_mut().for_each(|element_value| {
                        element_value.0 = element_value.0.to_lowercase();
                    });
                }
                Command::Insert(index, param) => {
                    let param = param.get_value(runtime_variable)?;

                    element_values.iter_mut().for_each(|(value, _)| {
                        value.insert_str(index, &param);
                    });
                }
                Command::Prepend(param) => {
                    let param = param.get_value(runtime_variable)?;
                    element_values.iter_mut().for_each(|(value, _)| {
                        value.insert_str(0, &param);
                    });
                }

                Command::Append(param) => {
                    let param = param.get_value(runtime_variable)?;

                    element_values.iter_mut().for_each(|(value, _)| {
                        value.push_str(&param);
                    });
                }
                Command::Delete(param) => {
                    let param = param.get_value(runtime_variable)?;

                    element_values.iter_mut().for_each(|element_value| {
                        element_value.0 = element_value.0.replace(&param, "");
                    });
                }
                Command::RegexExtract(param) => {
                    let regex = Regex::new(&param.get_value(runtime_variable)?)
                        .map_err(CrawlerErr::from)?;
                    element_values.iter_mut().for_each(|element_value| {
                        element_value.0 = regex
                            .find_iter(&element_value.0)
                            .map(|m| m.as_str())
                            .collect();
                    });
                }
                Command::RegexMatch(param) => {
                    let regex = Regex::new(&param.get_value(runtime_variable)?)
                        .map_err(CrawlerErr::from)?;
                    element_values.retain(|value| regex.is_match(&value.0));

                    if element_values.is_empty() {
                        return Ok(vec![]);
                    }
                }
                Command::RegexReplace(param, replace) => {
                    let regex = Regex::new(&param.get_value(runtime_variable)?)
                        .map_err(CrawlerErr::from)?;

                    let replace = replace.get_value(runtime_variable)?;
                    element_values.iter_mut().for_each(|element_value| {
                        element_value.0 = regex.replace_all(&element_value.0, &replace).to_string();
                    });
                }
                Command::Equals(param) => {
                    let param = param.get_value(runtime_variable)?;

                    element_values.retain(|value| value.0 == param);

                    if element_values.is_empty() {
                        return Ok(vec![]);
                    }
                }
            }
        }

        Ok(element_values)
    }

    pub(crate) fn get_values(
        &self,
        root_element_ref: Vec<ElementRef<'_>>,
        runtime_variable: &mut HashMap<String, Vec<String>>,
    ) -> Result<Vec<String>, CrawlerErr> {
        Ok(self
            .get_value_with_element(root_element_ref, runtime_variable)?
            .into_iter()
            .map(|(value, _)| value)
            .collect())
    }

    pub(crate) fn get_elements<'a>(
        &self,
        root_element_ref: Vec<ElementRef<'a>>,
        runtime_variable: &mut HashMap<String, Vec<String>>,
    ) -> Result<Vec<ElementRef<'a>>, CrawlerErr> {
        Ok(self
            .get_value_with_element(root_element_ref, runtime_variable)?
            .into_iter()
            .map(|(_, element)| element)
            .collect())
    }

    pub(crate) fn get_text_value(
        &self,
        text_value: &str,
        runtime_variable: &HashMap<String, Vec<String>>,
    ) -> Result<String, CrawlerErr> {
        let mut text_value = text_value.to_string();
        for command in self.commands.clone() {
            match command {
                Command::Replace(from, to) => {
                    let from = from.get_value(&mut HashMap::new())?;
                    let to = to.get_value(&mut HashMap::new())?;
                    text_value = text_value.replace(&from, &to);
                }
                Command::Uppercase => {
                    text_value = text_value.to_uppercase();
                }
                Command::Lowercase => {
                    text_value = text_value.to_lowercase();
                }
                Command::Append(param) => {
                    let param = param.get_value(&mut HashMap::new())?;
                    text_value.push_str(&param);
                }
                Command::Prepend(param) => {
                    let param = param.get_value(&mut HashMap::new())?;
                    text_value.insert_str(0, &param);
                }
                Command::Insert(index, param) => {
                    let param = param.get_value(&mut HashMap::new())?;
                    text_value.insert_str(index, &param);
                }
                Command::Delete(param) => {
                    let param = param.get_value(&mut HashMap::new())?;
                    text_value = text_value.replace(&param, "");
                }
                Command::RegexExtract(regex) => {
                    let regex = Regex::new(&regex.get_value(&mut HashMap::new())?)
                        .map_err(CrawlerErr::from)?;
                    text_value = regex.find_iter(&text_value).map(|m| m.as_str()).collect();
                }
                Command::RegexReplace(regex, replace) => {
                    let regex = Regex::new(&regex.get_value(&mut HashMap::new())?)
                        .map_err(CrawlerErr::from)?;
                    let replace = replace.get_value(runtime_variable)?;
                    text_value = regex.replace_all(&text_value, &replace).to_string();
                }
                _ => {}
            }
        }
        Ok(text_value)
    }
}

fn parse_transform_rule(pair: pest::iterators::Pair<Rule>) -> Result<Command, CrawlerErr> {
    match pair.as_rule() {
        Rule::replace => {
            let from = get_pair_param_with_index(&pair, 0);
            let to = get_pair_param_with_index(&pair, 1);
            Ok(Command::Replace(from, to))
        }
        Rule::uppercase => Ok(Command::Uppercase),
        Rule::lowercase => Ok(Command::Lowercase),
        Rule::insert => {
            let index = get_pair_string_with_index(&pair, 0)
                .trim()
                .parse()
                .unwrap_or(0);
            
            let param = get_pair_param_with_index(&pair, 1);
            Ok(Command::Insert(index, param))
        }
        Rule::prepend => Ok(Command::Prepend(get_pair_param(&pair))),
        Rule::append => Ok(Command::Append(get_pair_param(&pair))),
        Rule::delete => Ok(Command::Delete(get_pair_param(&pair))),
        Rule::regex_extract => {
            let pattern = get_pair_param(&pair);
            Ok(Command::RegexExtract(pattern))
        }
        Rule::regex_replace => {
            let regex_str = get_pair_param_with_index(&pair, 0);
            let replace_str = get_pair_param_with_index(&pair, 1);
            Ok(Command::RegexReplace(regex_str, replace_str))
        }
        _ => Err(CrawlerErr::UnsupportedTransformRule),
    }
}

fn parse_selector_rule(pair: pest::iterators::Pair<Rule>) -> Result<Command, CrawlerErr> {
    match pair.as_rule() {
        Rule::selector => {
            let param = get_pair_param(&pair);

            Ok(Command::Selector(param))
        }
        Rule::parent => {
            let index = pair.into_inner().as_str().parse().unwrap_or(1);
            Ok(Command::Parent(index))
        }
        Rule::prev => {
            let index = pair.into_inner().as_str().parse().unwrap_or(1);
            Ok(Command::Prev(index))
        }
        Rule::nth => {
            let index = pair.into_inner().as_str().parse().unwrap_or(1);
            Ok(Command::Nth(index))
        }
        _ => Err(CrawlerErr::UnsupportedSelectorRule),
    }
}

fn parse_condition_rule(pair: pest::iterators::Pair<Rule>) -> Result<Command, CrawlerErr> {
    match pair.as_rule() {
        Rule::equals => Ok(Command::Equals(get_pair_param(&pair))),
        Rule::regex_match => {
            let pattern = get_pair_param(&pair);

            Ok(Command::RegexMatch(pattern))
        }
        _ => Err(CrawlerErr::UnsupportedSelectorRule),
    }
}

fn parse_accessor_rule(pair: pest::iterators::Pair<Rule>) -> Result<Command, CrawlerErr> {
    match pair.as_rule() {
        Rule::html => Ok(Command::Html),
        Rule::attr => Ok(Command::Attr(get_pair_param(&pair))),
        Rule::val => Ok(Command::Val),
        _ => Err(CrawlerErr::UnsupportedSelectorRule),
    }
}

fn get_pair_string_with_index(pair: &pest::iterators::Pair<Rule>, index: usize) -> String {
    match pair.clone().into_inner().nth(index) {
        Some(pair) => pair.into_inner().to_string(),
        None => "".to_string(),
    }
}

fn get_pair_param(pair: &pest::iterators::Pair<Rule>) -> Param {
    get_pair_param_with_index(pair, 0)
}

fn get_pair_param_with_index(pair: &pest::iterators::Pair<Rule>, index: usize) -> Param {
    pair.clone()
        .into_inner()
        .nth(index)
        .map_or(Param::StaticStr(String::new()), |inner_pair| {
            let pair_str = inner_pair
                .clone()
                .into_inner()
                .map(|p| p.as_str())
                .collect::<String>();

            match inner_pair.as_rule() {
                Rule::param => Param::StaticStr(pair_str),
                Rule::dynamic_param => Param::DynamicStr(pair_str),
                _ => panic!("Unexpected rule type"),
            }
        })
}

impl Command {}

impl Display for CrawlerScript {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "rule: {} => ",
            match self.rule {
                Rule::element_access => "element get queue",
                Rule::value_access => "value gets the queue",
                _ => {
                    "unknown"
                }
            }
        )?;
        write!(
            f,
            "{}",
            self.commands
                .clone()
                .into_iter()
                .map(|command| command.to_string())
                .collect::<Vec<_>>()
                .join(" -> ")
        )
        .unwrap();
        Ok(())
    }
}

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::Selector(param) => write!(f, "selector({})", param),
            Command::Parent(param) => write!(f, "parent({})", param),
            Command::Prev(param) => write!(f, "prev({})", param),
            Command::Nth(param) => write!(f, "nth({})", param),
            Command::Replace(param1, param2) => {
                write!(f, "replace(from:{}, to:{})", param1, param2)
            }
            Command::Uppercase => write!(f, "uppercase"),
            Command::Lowercase => write!(f, "lowercase"),
            Command::Insert(param1, param2) => {
                write!(f, "insert(index:{}, value:{})", param1, param2)
            }
            Command::Prepend(param) => write!(f, "prepend({})", param),
            Command::Append(param) => write!(f, "append({})", param),
            Command::Delete(param) => write!(f, "delete({})", param),
            Command::RegexExtract(param) => write!(f, "regex_extract({})", param),
            Command::RegexMatch(param) => write!(f, "regex_match({})", param),
            Command::RegexReplace(param1, param2) => {
                write!(f, "regex_replace(reg:{}, replace:{})", param1, param2)
            }
            Command::Equals(param) => write!(f, "equal({})", param),
            Command::Html => write!(f, "html()"),
            Command::Attr(param) => write!(f, "attr({})", param),
            Command::Val => write!(f, "val()"),
        }
    }
}

impl Display for Param {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Param::StaticStr(param) => write!(f, "{}", param),
            Param::DynamicStr(param) => write!(f, "${{{}}}", param),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::CrawlerErr;

    #[test]
    fn test_new_element_access_selector() {
        let script = r#"selector("div.content")"#;
        let crawler_script = CrawlerScript::new(script, false).unwrap();

        assert_eq!(crawler_script.rule, Rule::element_access);
        assert_eq!(crawler_script.commands.len(), 1);

        match &crawler_script.commands[0] {
            Command::Selector(selector_str) => {
                assert_eq!(*selector_str, Param::StaticStr("div.content".to_string()));
            }
            _ => panic!("Unexpected command type"),
        }
    }

    #[test]
    fn test_new_element_access_with_multiple_commands() {
        let script = r#"selector("div.content").parent(1).attr("href")"#;
        let crawler_script = CrawlerScript::new(script, false).unwrap();

        assert_eq!(crawler_script.rule, Rule::value_access);
        assert_eq!(crawler_script.commands.len(), 3);

        match &crawler_script.commands[0] {
            Command::Selector(selector_str) => {
                assert_eq!(*selector_str, Param::StaticStr("div.content".to_owned()));
            }
            _ => panic!("Unexpected first command type"),
        }

        match &crawler_script.commands[1] {
            Command::Parent(index) => {
                assert_eq!(*index, 1);
            }
            _ => panic!("Unexpected second command type"),
        }

        match &crawler_script.commands[2] {
            Command::Attr(param) => {
                assert_eq!(param.to_string(), "href");
            }
            _ => panic!("Unexpected third command type"),
        }
    }

    #[test]
    fn test_new_value_access() {
        let script = r#"selector("div.content").val().uppercase()"#;
        let crawler_script = CrawlerScript::new(script, false).unwrap();

        assert_eq!(crawler_script.rule, Rule::value_access);
        assert_eq!(crawler_script.commands.len(), 3);

        match &crawler_script.commands[0] {
            Command::Selector(selector_str) => {
                assert_eq!(*selector_str, Param::StaticStr("div.content".to_owned()));
            }
            _ => panic!("Unexpected first command type"),
        }

        match &crawler_script.commands[1] {
            Command::Val => {}
            _ => panic!("Unexpected second command type"),
        }

        match &crawler_script.commands[2] {
            Command::Uppercase => {}
            _ => panic!("Unexpected third command type"),
        }
    }

    #[test]
    fn test_new_text_access() {
        let script = r#"replace("old", "new").uppercase()"#;
        let crawler_script = CrawlerScript::new(script, true).unwrap();

        assert_eq!(crawler_script.rule, Rule::text_access);
        assert_eq!(crawler_script.commands.len(), 2);

        match &crawler_script.commands[0] {
            Command::Replace(from, to) => {
                assert_eq!(from.to_string(), "old");
                assert_eq!(to.to_string(), "new");
            }
            _ => panic!("Unexpected first command type"),
        }

        match &crawler_script.commands[1] {
            Command::Uppercase => {}
            _ => panic!("Unexpected second command type"),
        }
    }

    #[test]
    fn test_new_invalid_text_script_for_element_access() {
        let script = r#"replace("old", "new")"#;
        let result = CrawlerScript::new(script, false);

        assert!(matches!(result, Err(CrawlerErr::CharProcessAlone)));
    }

    #[test]
    fn test_new_complex_access_with_conditions() {
        let script = r#"selector("div.content").html().replace("old", "new").equals("result")"#;
        let crawler_script = CrawlerScript::new(script, false).unwrap();

        assert_eq!(crawler_script.rule, Rule::element_access);
        assert_eq!(crawler_script.commands.len(), 4);

        match &crawler_script.commands[0] {
            Command::Selector(selector_str) => {
                assert_eq!(*selector_str, Param::StaticStr("div.content".to_owned()));
            }
            _ => panic!("Unexpected first command type"),
        }

        match &crawler_script.commands[1] {
            Command::Html => {}
            _ => panic!("Unexpected second command type"),
        }

        match &crawler_script.commands[2] {
            Command::Replace(from, to) => {
                assert_eq!(from.to_string(), "old");
                assert_eq!(to.to_string(), "new");
            }
            _ => panic!("Unexpected third command type"),
        }

        match &crawler_script.commands[3] {
            Command::Equals(param) => {
                assert_eq!(param.to_string(), "result");
            }
            _ => panic!("Unexpected fourth command type"),
        }
    }
}
