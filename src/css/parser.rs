use anyhow::{bail, Error, Result};
use simplecss::{AttributeOperator, Element, PseudoClass, StyleSheet};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::str::FromStr;

use super::properties::{parse_declarations, Align, FontFamily, Radius, Sides};
use super::rules::{Rule, Rules, RulesBuilder};

impl FromStr for FontFamily {
    type Err = Error;

    fn from_str(line: &str) -> Result<Self> {
        match line.to_lowercase().as_str() {
            "bold" => Ok(FontFamily::Bold),
            "regular" => Ok(FontFamily::Regular),
            "mono" => Ok(FontFamily::Mono),
            _ => bail!("Unknown font-family"),
        }
    }
}

impl FromStr for Sides {
    type Err = Error;

    fn from_str(line: &str) -> Result<Self> {
        let words = line
            .split_whitespace()
            .map(|word| Ok(word.parse()?))
            .collect::<Result<Vec<f32>>>()?;
        match words.len() {
            1 => Ok(Sides {
                top: words[0],
                right: words[0],
                bottom: words[0],
                left: words[0],
            }),
            2 => Ok(Sides {
                top: words[0],
                right: words[1],
                bottom: words[0],
                left: words[1],
            }),
            4 => Ok(Sides {
                top: words[0],
                right: words[1],
                bottom: words[2],
                left: words[3],
            }),
            _ => bail!("Parsing margin/padding failed. Expected 1, 2 or 4 values"),
        }
    }
}

impl FromStr for Radius {
    type Err = Error;

    fn from_str(line: &str) -> Result<Self> {
        let words = line
            .split_whitespace()
            .map(|word| Ok(word.parse()?))
            .collect::<Result<Vec<f32>>>()?;
        match words.len() {
            1 => Ok(Radius {
                nw: words[0],
                ne: words[0],
                sw: words[0],
                se: words[0],
            }),
            4 => Ok(Radius {
                nw: words[0],
                ne: words[1],
                se: words[2],
                sw: words[3],
            }),
            _ => bail!("Parsing margin/padding failed. Expected 1 or 4 values"),
        }
    }
}

impl FromStr for Align {
    type Err = Error;

    fn from_str(line: &str) -> Result<Self> {
        match line {
            "center" => Ok(Align::Center),
            "min" | "left" | "bottom" => Ok(Align::Min),
            "max" | "right" | "top" => Ok(Align::Max),
            _ => bail!("Invalid align value"),
        }
    }
}

// We use this element just early on during the parsing process
// to find variables to later on resolve the property valus.
struct VariableElement<'a> {
    classes: HashSet<&'a str>,
}

impl<'a> Element for VariableElement<'a> {
    fn has_local_name(&self, name: &str) -> bool {
        name == "variables"
    }

    fn attribute_matches(&self, attr: &str, operator: AttributeOperator<'_>) -> bool {
        if self.classes.is_empty() {
            // As if that variables element had all the attributes and such.
            // Useful to find all the variable rules.
            return true;
        }
        match (attr, operator) {
            ("class", AttributeOperator::Contains(class)) => self.classes.contains(class),
            _ => false,
        }
    }

    fn parent_element(&self) -> Option<Self> {
        None
    }

    fn prev_sibling_element(&self) -> Option<Self> {
        None
    }

    fn pseudo_class_matches(&self, _class: PseudoClass<'_>) -> bool {
        false
    }
}

pub fn parse_css(path: &str, system_classes: &[&str]) -> Result<Rules> {
    let classes = system_classes.iter().copied().collect();
    let var_elt = VariableElement {
        classes: HashSet::new(),
    };
    let system_var_elt = VariableElement { classes };

    let source = std::fs::read_to_string(Path::new(path))?;
    let rules = RulesBuilder {
        source,
        rules_builder: |source: &String| {
            let stylesheet = StyleSheet::parse(source);

            let mut variables = HashMap::new();

            stylesheet
                .rules
                .iter()
                .filter(|r| r.selector.matches(&system_var_elt))
                .for_each(|rule| {
                    for declaration in &rule.declarations {
                        variables.insert(declaration.name.into(), declaration.value.into());
                    }
                });

            stylesheet
                .rules
                .into_iter()
                .filter(|r| {
                    // Ditch `variable {}` rules.
                    // Need to check the spec too, otherwise `* {}` will be ditched too.
                    let spec = r.selector.specificity();
                    !r.selector.matches(&var_elt) || spec[2] == 0
                })
                .map(|r| Rule {
                    selector: r.selector,
                    properties: parse_declarations(&r.declarations, &variables),
                })
                .collect()
        },
    }
    .build();
    Ok(rules)
}
