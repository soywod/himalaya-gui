//! Element are light object identifying a UI toolkit widget. For now, they are
//! only used to be matched against the CSS stylesheet.

use std::collections::HashSet;

use crate::css::{
    AttributeOperator, ComputedProperties as CSSProps, Element as CSSElement, PseudoClass, Rules,
};

#[derive(Debug, Clone)]
pub struct Element {
    parent: Option<Box<Element>>,
    local: &'static str,
    id: Option<&'static str>,
    classes: HashSet<&'static str>,
    hover: bool,
    active: bool,
    focus: bool,
}

#[derive(Clone)]
pub struct ComputedElement(pub(crate) CSSProps);

impl ComputedElement {
    pub fn props(&self) -> &CSSProps {
        &self.0
    }

    pub fn props_mut(&mut self) -> &mut CSSProps {
        &mut self.0
    }
}

impl Element {
    pub(crate) fn new(local: &'static str) -> Element {
        Element {
            local,
            id: None,
            classes: HashSet::new(),
            hover: false,
            active: false,
            focus: false,
            parent: None,
        }
    }

    pub fn id(mut self, id: &'static str) -> Element {
        self.id = Some(id);
        self
    }

    pub fn classes(mut self, classes: &'static str) -> Element {
        self.classes = classes.split_whitespace().collect();
        self
    }

    pub fn hover(mut self, hover: bool) -> Element {
        self.hover = hover;
        self
    }

    pub fn active(mut self, active: bool) -> Element {
        self.active = active;
        self
    }

    pub fn focus(mut self, focus: bool) -> Element {
        self.focus = focus;
        self
    }

    pub fn parent(mut self, parent: Element) -> Element {
        self.parent = Some(Box::new(parent));
        self
    }

    pub fn add_class(&mut self, class: &'static str) {
        self.classes.insert(class);
    }

    pub fn remove_class(&mut self, class: &'static str) {
        self.classes.remove(class);
    }

    pub fn attach_parent(&mut self, parent: Element) {
        self.parent = Some(Box::new(parent));
    }

    pub fn toggle_class(&mut self, class: &'static str, on: bool) {
        if on {
            self.add_class(class);
        } else {
            self.remove_class(class);
        }
    }

    pub fn compute(&self, rules: &Rules) -> ComputedElement {
        let props = rules.solve(self);
        ComputedElement(props)
    }
}

impl std::fmt::Display for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(parent) = &self.parent {
            write!(f, "{} > ", parent)?;
        }
        write!(f, "{}", self.local)?;
        if let Some(id) = self.id {
            write!(f, "#{}", id)?;
        }
        for class in &self.classes {
            write!(f, ".{}", class)?;
        }
        if self.hover {
            write!(f, ":hover")?;
        }
        if self.active {
            write!(f, ":active")?;
        }
        if self.focus {
            write!(f, ":focus")?;
        }

        Ok(())
    }
}

impl CSSElement for Element {
    fn parent_element(&self) -> Option<Self> {
        self.parent.as_ref().map(|elt| *elt.clone())
    }

    fn prev_sibling_element(&self) -> Option<Self> {
        None
    }

    fn has_local_name(&self, name: &str) -> bool {
        self.local == name
    }

    fn attribute_matches(&self, attr: &str, operator: AttributeOperator<'_>) -> bool {
        match (attr, operator) {
            ("class", AttributeOperator::Contains(class)) => self.classes.contains(class),
            ("id", AttributeOperator::Matches(id)) => {
                self.id.map_or(false, |local_id| local_id == id)
            }
            _ => false,
        }
    }

    fn pseudo_class_matches(&self, class: PseudoClass<'_>) -> bool {
        match class {
            PseudoClass::Hover => self.hover,
            PseudoClass::Active => self.active,
            PseudoClass::Focus => self.focus,
            _ => false,
        }
    }
}

pub fn window() -> Element {
    Element::new("window")
}

pub fn native() -> Element {
    Element::new("native")
}

pub fn label() -> Element {
    Element::new("label")
}

pub fn vbox() -> Element {
    Element::new("vbox")
}

pub fn hbox() -> Element {
    Element::new("hbox")
}

pub fn panel() -> Element {
    Element::new("panel")
}
