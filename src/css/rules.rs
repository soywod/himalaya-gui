use log::warn;
use ouroboros::self_referencing;
use simplecss::{Element, Selector};

use super::properties::{ComputedProperties, OptionalProperties};

pub struct Rule<'a> {
    pub(crate) selector: Selector<'a>,
    pub(crate) properties: OptionalProperties,
}

// We use ouroboros because handling lifetimes for rules was becoming difficult.
// A self referencing struct solves the issue.
#[self_referencing(pub_extras)]
pub struct Rules {
    source: String,
    #[borrows(source)]
    #[covariant]
    rules: Vec<Rule<'this>>,
}

impl Rules {
    pub fn solve<E: Element + std::fmt::Display>(&self, element: &E) -> ComputedProperties {
        let mut prop = ComputedProperties::default();
        let mut matched = false;
        for rule in self.borrow_rules().iter() {
            if rule.selector.matches(element) {
                matched = true;
                prop.patch_from(&rule.properties);
            }
        }
        if !matched {
            warn!("Couldn't match any rules for element: {}", element);
        }
        prop
    }
}
