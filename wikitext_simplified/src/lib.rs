use std::sync::LazyLock;

use serde::{Deserialize, Serialize};

use parse_wiki_text_2 as pwt;
use wikitext_util::{
    nodes_inner_text, nodes_inner_wikitext, wikipedia_pwt_configuration, InnerTextConfig,
    NodeMetadata,
};

#[cfg(feature = "wasm")]
use tsify_next::Tsify;
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[serde(tag = "type", rename_all = "kebab-case")]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub enum WikitextSimplifiedNode {
    Fragment {
        children: Vec<WikitextSimplifiedNode>,
    },
    Template {
        name: String,
        children: Vec<TemplateParameter>,
    },
    Link {
        text: String,
        title: String,
    },
    ExtLink {
        link: String,
        text: Option<String>,
    },
    Bold {
        children: Vec<WikitextSimplifiedNode>,
    },
    Italic {
        children: Vec<WikitextSimplifiedNode>,
    },
    Blockquote {
        children: Vec<WikitextSimplifiedNode>,
    },
    Superscript {
        children: Vec<WikitextSimplifiedNode>,
    },
    Subscript {
        children: Vec<WikitextSimplifiedNode>,
    },
    Small {
        children: Vec<WikitextSimplifiedNode>,
    },
    Preformatted {
        children: Vec<WikitextSimplifiedNode>,
    },
    Text {
        text: String,
    },
    ParagraphBreak,
    Newline,
}
impl WikitextSimplifiedNode {
    pub fn children(&self) -> Option<&[WikitextSimplifiedNode]> {
        match self {
            Self::Fragment { children } => Some(children),
            Self::Bold { children } => Some(children),
            Self::Italic { children } => Some(children),
            Self::Blockquote { children } => Some(children),
            Self::Superscript { children } => Some(children),
            Self::Subscript { children } => Some(children),
            Self::Small { children } => Some(children),
            Self::Preformatted { children } => Some(children),
            _ => None,
        }
    }
    pub fn children_mut(&mut self) -> Option<&mut Vec<WikitextSimplifiedNode>> {
        match self {
            Self::Fragment { children } => Some(children),
            Self::Bold { children } => Some(children),
            Self::Italic { children } => Some(children),
            Self::Blockquote { children } => Some(children),
            Self::Superscript { children } => Some(children),
            Self::Subscript { children } => Some(children),
            Self::Small { children } => Some(children),
            Self::Preformatted { children } => Some(children),
            _ => None,
        }
    }
    pub fn visit_mut(&mut self, visitor: &mut impl FnMut(&mut Self)) {
        visitor(self);
        if let Some(children) = self.children_mut() {
            for child in children {
                child.visit_mut(visitor);
            }
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct TemplateParameter {
    pub name: String,
    pub value: String,
}

/// Parses wikitext and simplifies it into a nested AST using Wikipedia's PWT configuration.
///
/// Will panic on failure.
pub fn parse_and_simplify_wikitext(wikitext: &str) -> Vec<WikitextSimplifiedNode> {
    static PWT_CONFIGURATION: LazyLock<pwt::Configuration> =
        LazyLock::new(wikipedia_pwt_configuration);

    let output = PWT_CONFIGURATION.parse(wikitext).unwrap();
    simplify_wikitext_nodes(wikitext, &output.nodes)
}

pub fn simplify_wikitext_nodes(wikitext: &str, nodes: &[pwt::Node]) -> Vec<WikitextSimplifiedNode> {
    use WikitextSimplifiedNode as WSN;
    struct RootStack {
        stack: Vec<WSN>,
    }
    impl RootStack {
        fn new() -> Self {
            Self {
                stack: vec![WSN::Fragment { children: vec![] }],
            }
        }
        fn push_layer(&mut self, node: WSN) {
            self.stack.push(node);
        }
        fn pop_layer(&mut self) -> WSN {
            self.stack.pop().unwrap()
        }
        fn last_layer(&self) -> &WSN {
            self.stack.last().unwrap()
        }
        fn add_to_children(&mut self, node: WSN) {
            self.stack
                .last_mut()
                .unwrap()
                .children_mut()
                .unwrap()
                .push(node);
        }
        fn unwind(mut self) -> Vec<WSN> {
            // This is a disgusting hack, but Wikipedia implicitly closes these, so we need to as well...
            while self.stack.len() > 1 {
                let popped = self.pop_layer();
                self.add_to_children(popped);
            }
            self.stack[0].children().unwrap().to_vec()
        }
    }
    let mut root_stack = RootStack::new();

    for node in nodes {
        match node {
            pwt::Node::Bold { .. } => {
                if matches!(root_stack.last_layer(), WSN::Bold { .. }) {
                    let bold = root_stack.pop_layer();
                    root_stack.add_to_children(bold);
                } else {
                    root_stack.push_layer(WSN::Bold { children: vec![] });
                }
            }
            pwt::Node::Italic { .. } => {
                if matches!(root_stack.last_layer(), WSN::Italic { .. }) {
                    let italic = root_stack.pop_layer();
                    root_stack.add_to_children(italic);
                } else {
                    root_stack.push_layer(WSN::Italic { children: vec![] });
                }
            }
            pwt::Node::BoldItalic { .. } => {
                if matches!(root_stack.last_layer(), WSN::Italic { .. }) {
                    let italic = root_stack.pop_layer();
                    if matches!(root_stack.last_layer(), WSN::Bold { .. }) {
                        let mut bold = root_stack.pop_layer();
                        bold.children_mut().unwrap().push(italic);
                        root_stack.add_to_children(bold);
                    } else {
                        panic!("BoldItalic found without a bold layer");
                    }
                } else {
                    root_stack.push_layer(WSN::Bold { children: vec![] });
                    root_stack.push_layer(WSN::Italic { children: vec![] });
                }
            }
            pwt::Node::StartTag { name, .. } if name == "blockquote" => {
                root_stack.push_layer(WSN::Blockquote { children: vec![] });
            }
            pwt::Node::EndTag { name, .. } if name == "blockquote" => {
                let blockquote = root_stack.pop_layer();
                root_stack.add_to_children(blockquote);
            }
            pwt::Node::StartTag { name, .. } if name == "sup" => {
                root_stack.push_layer(WSN::Superscript { children: vec![] });
            }
            pwt::Node::EndTag { name, .. } if name == "sup" => {
                let superscript = root_stack.pop_layer();
                root_stack.add_to_children(superscript);
            }
            pwt::Node::StartTag { name, .. } if name == "sub" => {
                root_stack.push_layer(WSN::Subscript { children: vec![] });
            }
            pwt::Node::EndTag { name, .. } if name == "sub" => {
                let subscript = root_stack.pop_layer();
                root_stack.add_to_children(subscript);
            }
            pwt::Node::StartTag { name, .. } if name == "small" => {
                root_stack.push_layer(WSN::Small { children: vec![] });
            }
            pwt::Node::EndTag { name, .. } if name == "small" => {
                let small = root_stack.pop_layer();
                root_stack.add_to_children(small);
            }
            other => {
                if let Some(simplified_node) = simplify_wikitext_node(wikitext, other) {
                    root_stack.add_to_children(simplified_node);
                }
            }
        }
    }

    root_stack.unwind()
}

pub fn simplify_wikitext_node(wikitext: &str, node: &pwt::Node) -> Option<WikitextSimplifiedNode> {
    use WikitextSimplifiedNode as WSN;
    match node {
        pwt::Node::Template {
            name, parameters, ..
        } => {
            let mut unnamed_parameter_index = 1;
            let mut children = vec![];
            for parameter in parameters {
                let name = if let Some(parameter_name) = &parameter.name {
                    nodes_inner_text(parameter_name, &InnerTextConfig::default())
                } else {
                    let name = unnamed_parameter_index.to_string();
                    unnamed_parameter_index += 1;
                    name
                };

                let value_start = parameter
                    .value
                    .first()
                    .map(|v| NodeMetadata::for_node(v).start)
                    .unwrap_or_default();
                let value_end = parameter
                    .value
                    .last()
                    .map(|v| NodeMetadata::for_node(v).end)
                    .unwrap_or_default();
                let value = wikitext[value_start..value_end].to_string();

                children.push(TemplateParameter { name, value });
            }

            return Some(WSN::Template {
                name: nodes_inner_text(name, &InnerTextConfig::default()),
                children,
            });
        }
        pwt::Node::MagicWord { .. } => {
            // Making the current assumption that we don't care about these
            return None;
        }
        pwt::Node::Bold { .. } | pwt::Node::BoldItalic { .. } | pwt::Node::Italic { .. } => {
            // We can't do anything at this level
            return None;
        }
        pwt::Node::Link { target, text, .. } => {
            return Some(WSN::Link {
                text: nodes_inner_wikitext(wikitext, text),
                title: target.to_string(),
            });
        }
        pwt::Node::ExternalLink { nodes, .. } => {
            let inner = nodes_inner_wikitext(wikitext, nodes);
            let (link, text) = inner
                .split_once(' ')
                .map(|(l, t)| (l, Some(t)))
                .unwrap_or((&inner, None));
            return Some(WSN::ExtLink {
                link: link.to_string(),
                text: text.map(|s| s.to_string()),
            });
        }
        pwt::Node::Text { value, .. } => {
            return Some(WSN::Text {
                text: value.to_string(),
            });
        }
        pwt::Node::CharacterEntity { character, .. } => {
            return Some(WSN::Text {
                text: character.to_string(),
            });
        }
        pwt::Node::ParagraphBreak { .. } => {
            return Some(WSN::ParagraphBreak);
        }
        pwt::Node::Category { .. } | pwt::Node::Comment { .. } | pwt::Node::Image { .. } => {
            // Don't care
            return None;
        }
        pwt::Node::DefinitionList { .. }
        | pwt::Node::OrderedList { .. }
        | pwt::Node::UnorderedList { .. } => {
            // Temporarily ignore these
            return None;
        }
        pwt::Node::Tag { name, .. }
            if ["nowiki", "references", "gallery", "ref"].contains(&name.as_ref()) =>
        {
            // Don't care
            return None;
        }
        pwt::Node::StartTag { name, .. } if name == "br" => {
            return Some(WSN::Newline);
        }
        pwt::Node::Preformatted { nodes, .. } => {
            return Some(WSN::Preformatted {
                children: simplify_wikitext_nodes(wikitext, nodes),
            });
        }
        _ => {}
    }
    let metadata = NodeMetadata::for_node(node);
    panic!(
        "Unknown node type: {:?}: {:?}",
        node,
        &wikitext[metadata.start..metadata.end]
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use WikitextSimplifiedNode as WSN;

    #[test]
    fn test_s_after_link() {
        let wikitext = "cool [[thing]]s by cool [[Person|person]]s";
        let simplified = parse_and_simplify_wikitext(wikitext);
        assert_eq!(
            simplified,
            vec![
                WSN::Text {
                    text: "cool ".into()
                },
                WSN::Link {
                    text: "thing".into(),
                    title: "thing".into()
                },
                WSN::Text {
                    text: "s by cool ".into()
                },
                WSN::Link {
                    text: "person".into(),
                    title: "Person".into()
                },
                WSN::Text { text: "s".into() }
            ]
        )
    }

    #[test]
    fn can_parse_wikitext_in_link() {
        let wikitext = r#"[[Time signature|{{music|time|4|4}}]]"#;
        let simplified = parse_and_simplify_wikitext(wikitext);
        assert_eq!(
            simplified,
            vec![WSN::Link {
                text: "{{music|time|4|4}}".into(),
                title: "Time signature".into()
            }]
        )
    }

    #[test]
    fn will_gracefully_ignore_refs() {
        let wikitext = r#"<ref name=bigtakeover>{{cite web|author=Kristen Sollee|title=Japanese Rock on NPR|work=[[The Big Takeover]]|date=2006-06-25|url=http://www.bigtakeover.com/news/japanese-rock-on-npr|access-date=2013-06-07|quote=It's a style of dress, there's a lot of costuming and make up and it's uniquely Japanese because it goes back to ancient Japan. Men would often wear women's clothing...}}</ref>"#;
        let simplified = parse_and_simplify_wikitext(wikitext);
        assert_eq!(simplified, vec![]);
    }
}
