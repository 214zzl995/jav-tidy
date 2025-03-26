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
    Selector(&'static str, Option<Selector>),
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
    RegexMatch(&'static str, Regex),
    Equals(Param),
    Html,
    Attr(Param),
    Val,
    RegexExtract(&'static str, Regex),
    RegexReplace(&'static str, &'static str, Regex),
}

#[derive(Debug, Clone)]
enum Param {
    StaticStr(&'static str),
    DynamicStr(&'static str),
}

impl Param {
    pub(crate) fn get_value(
        &self,
        runtime_variable: &mut HashMap<String, Vec<String>>,
    ) -> Result<&'static str, CrawlerErr> {
        match self {
            Param::StaticStr(param) => Ok(param),
            Param::DynamicStr(param) => {
                let values = runtime_variable
                    .entry(param.to_string())
                    .or_default()
                    .clone();

                if values.is_empty() {
                    return Err(CrawlerErr::DynNoValidData(param.to_string()));
                }

                if values.len() > 1 {
                    return Err(CrawlerErr::DynMultipleResults(
                        param.to_string(),
                        values.join(","),
                    ));
                }

                Ok(Box::leak(values.first().unwrap().clone().into_boxed_str()))
            }
        }
    }
}

impl CrawlerScript {
    pub fn new(script: &str) -> Result<CrawlerScript, CrawlerErr> {
        let mut commands = Vec::new();
        let mut pairs = ScriptParser::parse(Rule::script, script)?;

        let script_pair = pairs.next().unwrap();
        let rule = script_pair.as_rule();

        for pair in script_pair.into_inner() {
            match pair.as_rule() {
                Rule::selector => {
                    let param = get_pair_string(pair);

                    let selector = match param {
                        "" => None,
                        _ => Some(
                            Selector::parse(param)
                                .map_err(|err| CrawlerErr::SelectorError(err.to_string()))?,
                        ),
                    };
                    commands.push(Command::Selector(param, selector));
                }
                Rule::parent => {
                    let index = pair.into_inner().as_str().parse::<usize>().unwrap_or(1);
                    commands.push(Command::Parent(index));
                }
                Rule::prev => {
                    let index = pair.into_inner().as_str().parse::<usize>().unwrap_or(1);
                    commands.push(Command::Prev(index));
                }
                Rule::nth => {
                    let index = pair.into_inner().as_str().parse::<usize>().unwrap_or(1);
                    commands.push(Command::Nth(index));
                }
                Rule::replace => {
                    let from = get_pair_param_with_index(&pair, 0);

                    let to = get_pair_param_with_index(&pair, 1);

                    commands.push(Command::Replace(from, to));
                }
                Rule::uppercase => {
                    commands.push(Command::Uppercase);
                }
                Rule::lowercase => {
                    commands.push(Command::Lowercase);
                }
                Rule::insert => {
                    let param = get_pair_param_with_index(&pair, 1);

                    let index = pair
                        .into_inner()
                        .next()
                        .unwrap()
                        .as_str()
                        .parse::<usize>()
                        .unwrap();
                    commands.push(Command::Insert(index, param));
                }
                Rule::prepend => {
                    commands.push(Command::Prepend(get_pair_param(&pair)));
                }
                Rule::append => {
                    commands.push(Command::Append(get_pair_param(&pair)));
                }
                Rule::delete => {
                    commands.push(Command::Delete(get_pair_param(&pair)));
                }
                Rule::regex_extract => {
                    let param = get_pair_string(pair);
                    let regex = Regex::new(param)?;
                    commands.push(Command::RegexExtract(param, regex));
                }
                Rule::regex_replace => {
                    let regex_str = get_pair_string_with_index(&pair, 0);
                    let replace_str = get_pair_string_with_index(&pair, 1);

                    let regex = Regex::new(regex_str)?;

                    commands.push(Command::RegexReplace(regex_str, replace_str, regex));
                }
                Rule::regex_match => {
                    let param = get_pair_string(pair);
                    let regex = Regex::new(param)?;
                    commands.push(Command::RegexMatch(param, regex));
                }
                Rule::equals => {
                    commands.push(Command::Equals(get_pair_param(&pair)));
                }
                Rule::html => {
                    commands.push(Command::Html);
                }
                Rule::attr => {
                    commands.push(Command::Attr(get_pair_param(&pair)));
                }
                Rule::val => {
                    commands.push(Command::Val);
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
                Command::Selector(_, selector) => match selector {
                    Some(selector) => {
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
                    None => continue,
                },
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
                        value.0 = value.1.value().attr(attr).unwrap_or("").to_string();
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
                        element_value.0 = element_value.0.replace(from, to);
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
                        value.insert_str(index, param);
                    });
                }
                Command::Prepend(param) => {
                    let param = param.get_value(runtime_variable)?;
                    element_values.iter_mut().for_each(|(value, _)| {
                        value.insert_str(0, param);
                    });
                }

                Command::Append(param) => {
                    let param = param.get_value(runtime_variable)?;

                    element_values.iter_mut().for_each(|(value, _)| {
                        value.push_str(param);
                    });
                }
                Command::Delete(param) => {
                    let param = param.get_value(runtime_variable)?;

                    element_values.iter_mut().for_each(|element_value| {
                        element_value.0 = element_value.0.replace(param, "");
                    });
                }
                Command::RegexExtract(_, regex) => {
                    element_values.iter_mut().for_each(|element_value| {
                        element_value.0 = regex
                            .find_iter(&element_value.0)
                            .map(|m| m.as_str())
                            .collect();
                    });
                }
                Command::RegexMatch(_, param) => {
                    element_values.retain(|value| param.is_match(&value.0));

                    if element_values.is_empty() {
                        return Ok(vec![]);
                    }
                }
                Command::RegexReplace(_, replace, regex) => {
                    element_values.iter_mut().for_each(|element_value| {
                        element_value.0 = regex.replace_all(&element_value.0, replace).to_string();
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
}

fn get_pair_string(pair: pest::iterators::Pair<Rule>) -> &'static str {
    let pair = match pair.into_inner().next() {
        Some(pair) => pair.into_inner(),
        None => return "",
    };
    Box::leak(
        pair.into_iter()
            .map(|p| p.as_str())
            .collect::<String>()
            .into_boxed_str(),
    )
}

fn get_pair_string_with_index(pair: &pest::iterators::Pair<Rule>, index: usize) -> &'static str {
    let pair = match pair.clone().into_inner().nth(index) {
        Some(pair) => pair.into_inner(),
        None => return "",
    };
    Box::leak(
        pair.into_iter()
            .map(|p| p.as_str())
            .collect::<String>()
            .into_boxed_str(),
    )
}

fn get_pair_param(pair: &pest::iterators::Pair<Rule>) -> Param {
    let pair = match pair.clone().into_inner().next() {
        Some(pair) => pair,
        None => return Param::StaticStr(""),
    };
    let pair_str = Box::leak(
        pair.clone()
            .into_inner()
            .map(|p| p.as_str())
            .collect::<String>()
            .into_boxed_str(),
    );

    match pair.as_rule() {
        Rule::param => Param::StaticStr(pair_str),
        Rule::dynamic_param => Param::DynamicStr(pair_str),
        _ => todo!(),
    }
}

fn get_pair_param_with_index(pair: &pest::iterators::Pair<Rule>, index: usize) -> Param {
    let pair = match pair.clone().into_inner().nth(index) {
        Some(pair) => pair,
        None => return Param::StaticStr(""),
    };
    let pair_str = Box::leak(
        pair.clone()
            .into_inner()
            .map(|p| p.as_str())
            .collect::<String>()
            .into_boxed_str(),
    );

    match pair.as_rule() {
        Rule::param => Param::StaticStr(pair_str),
        Rule::dynamic_param => Param::DynamicStr(pair_str),
        _ => todo!(),
    }
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
            Command::Selector(param, _) => write!(f, "selector({})", param),
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
            Command::RegexExtract(param, _) => write!(f, "regex_extract({})", param),
            Command::RegexMatch(param, _) => write!(f, "regex_match({})", param),
            Command::RegexReplace(param1, param2, _) => {
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
