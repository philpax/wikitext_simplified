//! A library for simplifying wikitext into a more manageable AST structure.
//!
//! This library provides functionality to parse wikitext and convert it into a simplified
//! abstract syntax tree (AST) that's easier to work with. It handles various wikitext
//! elements like templates, links, formatting, and more.

#![deny(missing_docs)]

pub use parse_wiki_text_2;

use serde::{Deserialize, Serialize};

use parse_wiki_text_2 as pwt;
use wikitext_util::{nodes_inner_text, nodes_wikitext, NodeMetadata};

#[cfg(feature = "wasm")]
use tsify_next::Tsify;
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

/// Errors that can occur during simplification of wikitext nodes
#[derive(Debug)]
pub enum WikitextError {
    /// Error occurred during simplification of wikitext nodes
    SimplificationError {
        /// The type of node that caused the error
        node_type: String,
        /// The context of where the error occurred
        context: WikitextErrorContext,
    },
    /// Error occurred due to invalid node structure
    InvalidNodeStructure {
        /// The specific type of structural error
        kind: NodeStructureError,
        /// The context of where the error occurred
        context: WikitextErrorContext,
    },
}
impl std::fmt::Display for WikitextError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WikitextError::SimplificationError { node_type, context } => {
                write!(
                    f,
                    "Simplification error: Unknown node type '{}' at position {}-{}: '{}'",
                    node_type, context.start, context.end, context.content
                )
            }
            WikitextError::InvalidNodeStructure { kind, context } => {
                write!(
                    f,
                    "Invalid node structure: {} at position {}-{}: '{}'",
                    kind, context.start, context.end, context.content
                )
            }
        }
    }
}
impl std::error::Error for WikitextError {}

/// Context information for errors that occur at specific positions in the wikitext
#[derive(Debug)]
pub struct WikitextErrorContext {
    /// The problematic content from the wikitext
    pub content: String,
    /// The start position of the problematic content
    pub start: usize,
    /// The end position of the problematic content
    pub end: usize,
}
impl WikitextErrorContext {
    /// Creates a new error context from a node's metadata
    pub fn from_node_metadata(wikitext: &str, metadata: &NodeMetadata) -> Self {
        Self {
            content: wikitext[metadata.start..metadata.end].to_string(),
            start: metadata.start,
            end: metadata.end,
        }
    }
}

/// Specific types of node structure errors that can occur
#[derive(Debug)]
pub enum NodeStructureError {
    /// Attempted to pop from, or access the last element of, an empty stack
    StackUnderflow,
    /// Attempted to push to a full stack (if we ever implement a size limit)
    StackOverflow,
    /// Attempted to access children of a node that has no children
    NoChildren,
    /// Found a bold-italic node without a corresponding bold node
    MissingBoldLayer,
    /// Found an unclosed formatting node
    UnclosedFormatting,
    /// Found an unexpected node type in the current context
    UnexpectedNodeType(String),
}
impl std::fmt::Display for NodeStructureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeStructureError::StackUnderflow => write!(f, "Stack underflow"),
            NodeStructureError::StackOverflow => write!(f, "Stack overflow"),
            NodeStructureError::NoChildren => write!(f, "Node has no children"),
            NodeStructureError::MissingBoldLayer => {
                write!(f, "Bold-italic found without a bold layer")
            }
            NodeStructureError::UnclosedFormatting => write!(f, "Unclosed formatting node"),
            NodeStructureError::UnexpectedNodeType(ty) => write!(f, "Unexpected node type: {}", ty),
        }
    }
}

/// Errors that can occur during parsing of wikitext
#[derive(Debug)]
pub enum ParseAndSimplifyWikitextError<'a> {
    /// Error occurred during parsing of wikitext
    ParseError(pwt::ParseError<'a>),
    /// Error occurred during simplification of wikitext nodes
    SimplificationError(WikitextError),
}

impl<'a> std::fmt::Display for ParseAndSimplifyWikitextError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseAndSimplifyWikitextError::ParseError(e) => write!(f, "Parse error: {:?}", e),
            ParseAndSimplifyWikitextError::SimplificationError(e) => write!(f, "{}", e),
        }
    }
}
impl<'a> std::error::Error for ParseAndSimplifyWikitextError<'a> {}

/// Helper function that parses wikitext and converts it into a simplified AST structure.
///
/// # Errors
///
/// This function will return an error if the wikitext cannot be parsed or simplified.
pub fn parse_and_simplify_wikitext<'a>(
    wikitext: &'a str,
    pwt_configuration: &pwt::Configuration,
) -> Result<Vec<WikitextSimplifiedNode>, ParseAndSimplifyWikitextError<'a>> {
    let output = pwt_configuration
        .parse(wikitext)
        .map_err(ParseAndSimplifyWikitextError::ParseError)?;

    simplify_wikitext_nodes(wikitext, &output.nodes)
        .map_err(ParseAndSimplifyWikitextError::SimplificationError)
}

/// A simplified representation of a wikitext node.
///
/// This enum represents the various types of nodes that can appear in simplified wikitext.
/// It's designed to be more straightforward to work with than the raw [`parse_wiki_text_2`] nodes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[serde(tag = "type", rename_all = "kebab-case")]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub enum WikitextSimplifiedNode {
    /// A container node that can hold multiple child nodes
    Fragment {
        /// The child nodes contained within this fragment
        children: Vec<WikitextSimplifiedNode>,
    },
    /// A template node, representing a wikitext template
    Template {
        /// The name of the template
        name: String,
        /// The parameters passed to the template
        children: Vec<TemplateParameter>,
    },
    /// An internal wiki link
    Link {
        /// The display text of the link
        text: String,
        /// The target page of the link
        title: String,
    },
    /// An external link
    ExtLink {
        /// The URL of the external link
        link: String,
        /// Optional display text for the link
        text: Option<String>,
    },
    /// Bold text formatting
    Bold {
        /// The content within the bold formatting
        children: Vec<WikitextSimplifiedNode>,
    },
    /// Italic text formatting
    Italic {
        /// The content within the italic formatting
        children: Vec<WikitextSimplifiedNode>,
    },
    /// Blockquote formatting
    Blockquote {
        /// The content within the blockquote
        children: Vec<WikitextSimplifiedNode>,
    },
    /// Superscript text formatting
    Superscript {
        /// The content within the superscript formatting
        children: Vec<WikitextSimplifiedNode>,
    },
    /// Subscript text formatting
    Subscript {
        /// The content within the subscript formatting
        children: Vec<WikitextSimplifiedNode>,
    },
    /// Small text formatting
    Small {
        /// The content within the small text formatting
        children: Vec<WikitextSimplifiedNode>,
    },
    /// Preformatted text
    Preformatted {
        /// The content within the preformatted block
        children: Vec<WikitextSimplifiedNode>,
    },
    /// Plain text content
    Text {
        /// The text content
        text: String,
    },
    /// A paragraph break
    ParagraphBreak,
    /// A line break
    Newline,
}
impl WikitextSimplifiedNode {
    /// Returns a reference to the children of this node, if it has any.
    ///
    /// This method returns `Some` for node types that can contain children (like `Fragment`,
    /// `Bold`, `Italic`, etc.) and `None` for leaf nodes (like `Text`, `ParagraphBreak`).
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

    /// Returns a mutable reference to the children of this node, if it has any.
    ///
    /// This method returns `Some` for node types that can contain children (like `Fragment`,
    /// `Bold`, `Italic`, etc.) and `None` for leaf nodes (like `Text`, `ParagraphBreak`).
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

    /// Visits this node and all its children recursively with the given visitor function.
    ///
    /// The visitor function is called on each node in depth-first order, starting with
    /// this node and then visiting all its children.
    pub fn visit_mut(&mut self, visitor: &mut impl FnMut(&mut Self)) {
        visitor(self);
        if let Some(children) = self.children_mut() {
            for child in children {
                child.visit_mut(visitor);
            }
        }
    }
}

/// A parameter for a wikitext template
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct TemplateParameter {
    /// The name of the parameter
    pub name: String,
    /// The value of the parameter
    pub value: String,
}

/// Converts a sequence of raw wikitext nodes into simplified nodes.
///
/// This function takes the original wikitext string and a sequence of nodes from
/// [`parse_wiki_text_2`] and converts them into the simplified node structure.
///
/// # Errors
///
/// This function will return an error if it encounters an unknown node type or if the stack
/// of nodes is not properly closed.
pub fn simplify_wikitext_nodes<'a>(
    wikitext: &'a str,
    nodes: &[pwt::Node],
) -> Result<Vec<WikitextSimplifiedNode>, WikitextError> {
    use WikitextSimplifiedNode as WSN;
    struct RootStack<'a> {
        stack: Vec<WSN>,
        wikitext: &'a str,
        current_node: Option<&'a pwt::Node<'a>>,
    }
    impl<'a> RootStack<'a> {
        fn new(wikitext: &'a str) -> Self {
            Self {
                stack: vec![WSN::Fragment { children: vec![] }],
                wikitext,
                current_node: None,
            }
        }
        fn push_layer(&mut self, node: WSN) {
            self.stack.push(node);
        }
        fn pop_layer(&mut self) -> Result<WSN, WikitextError> {
            self.stack
                .pop()
                .ok_or_else(|| WikitextError::InvalidNodeStructure {
                    kind: NodeStructureError::StackUnderflow,
                    context: Self::error_context_for_current_node(self.wikitext, self.current_node),
                })
        }
        fn last_layer(&self) -> &WSN {
            self.stack.last().unwrap()
        }
        fn add_to_children(&mut self, node: WSN) -> Result<(), WikitextError> {
            self.stack
                .last_mut()
                .ok_or_else(|| WikitextError::InvalidNodeStructure {
                    kind: NodeStructureError::StackUnderflow,
                    context: Self::error_context_for_current_node(self.wikitext, self.current_node),
                })?
                .children_mut()
                .ok_or_else(|| WikitextError::InvalidNodeStructure {
                    kind: NodeStructureError::NoChildren,
                    context: Self::error_context_for_current_node(self.wikitext, self.current_node),
                })?
                .push(node);
            Ok(())
        }
        fn unwind(mut self) -> Result<Vec<WSN>, WikitextError> {
            // This is a disgusting hack, but Wikipedia implicitly closes these, so we need to as well...
            while self.stack.len() > 1 {
                let popped = self.pop_layer()?;
                self.add_to_children(popped)?;
            }
            Ok(self.stack[0].children().unwrap().to_vec())
        }
        fn set_current_node(&mut self, node: &'a pwt::Node) {
            self.current_node = Some(node);
        }
        fn error_context_for_current_node(
            wikitext: &'a str,
            current_node: Option<&'a pwt::Node>,
        ) -> WikitextErrorContext {
            current_node
                .map(|node| {
                    WikitextErrorContext::from_node_metadata(
                        wikitext,
                        &NodeMetadata::for_node(node),
                    )
                })
                .unwrap_or_else(|| WikitextErrorContext {
                    content: "No current node".into(),
                    start: 0,
                    end: 0,
                })
        }
    }
    let mut root_stack = RootStack::new(wikitext);

    for node in nodes {
        root_stack.set_current_node(node);
        match node {
            pwt::Node::Bold { .. } => {
                if matches!(root_stack.last_layer(), WSN::Bold { .. }) {
                    let bold = root_stack.pop_layer()?;
                    root_stack.add_to_children(bold)?;
                } else {
                    root_stack.push_layer(WSN::Bold { children: vec![] });
                }
            }
            pwt::Node::Italic { .. } => {
                if matches!(root_stack.last_layer(), WSN::Italic { .. }) {
                    let italic = root_stack.pop_layer()?;
                    root_stack.add_to_children(italic)?;
                } else {
                    root_stack.push_layer(WSN::Italic { children: vec![] });
                }
            }
            pwt::Node::BoldItalic { .. } => {
                if matches!(root_stack.last_layer(), WSN::Italic { .. }) {
                    let italic = root_stack.pop_layer()?;
                    if matches!(root_stack.last_layer(), WSN::Bold { .. }) {
                        let mut bold = root_stack.pop_layer()?;
                        bold.children_mut().unwrap().push(italic);
                        root_stack.add_to_children(bold)?;
                    } else {
                        return Err(WikitextError::InvalidNodeStructure {
                            kind: NodeStructureError::MissingBoldLayer,
                            context: WikitextErrorContext::from_node_metadata(
                                wikitext,
                                &NodeMetadata::for_node(node),
                            ),
                        });
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
                let blockquote = root_stack.pop_layer()?;
                root_stack.add_to_children(blockquote)?;
            }
            pwt::Node::StartTag { name, .. } if name == "sup" => {
                root_stack.push_layer(WSN::Superscript { children: vec![] });
            }
            pwt::Node::EndTag { name, .. } if name == "sup" => {
                let superscript = root_stack.pop_layer()?;
                root_stack.add_to_children(superscript)?;
            }
            pwt::Node::StartTag { name, .. } if name == "sub" => {
                root_stack.push_layer(WSN::Subscript { children: vec![] });
            }
            pwt::Node::EndTag { name, .. } if name == "sub" => {
                let subscript = root_stack.pop_layer()?;
                root_stack.add_to_children(subscript)?;
            }
            pwt::Node::StartTag { name, .. } if name == "small" => {
                root_stack.push_layer(WSN::Small { children: vec![] });
            }
            pwt::Node::EndTag { name, .. } if name == "small" => {
                let small = root_stack.pop_layer()?;
                root_stack.add_to_children(small)?;
            }
            other => {
                if let Some(simplified_node) = simplify_wikitext_node(wikitext, other)? {
                    root_stack.add_to_children(simplified_node)?;
                }
            }
        }
    }

    root_stack.unwind()
}

/// Converts a single raw wikitext node into a simplified node.
///
/// This function handles the conversion of individual nodes from the [`parse_wiki_text_2`]
/// format into the simplified format. It handles various node types including templates,
/// links, text, and formatting nodes.
///
/// # Errors
///
/// This function will return an error if it encounters an unknown node type.
pub fn simplify_wikitext_node(
    wikitext: &str,
    node: &pwt::Node,
) -> Result<Option<WikitextSimplifiedNode>, WikitextError> {
    use WikitextSimplifiedNode as WSN;
    match node {
        pwt::Node::Template {
            name, parameters, ..
        } => {
            let mut unnamed_parameter_index = 1;
            let mut children = vec![];
            for parameter in parameters {
                let name = if let Some(parameter_name) = &parameter.name {
                    nodes_inner_text(parameter_name)
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

            return Ok(Some(WSN::Template {
                name: nodes_inner_text(name),
                children,
            }));
        }
        pwt::Node::MagicWord { .. } => {
            // Making the current assumption that we don't care about these
            return Ok(None);
        }
        pwt::Node::Bold { .. } | pwt::Node::BoldItalic { .. } | pwt::Node::Italic { .. } => {
            // We can't do anything at this level
            return Ok(None);
        }
        pwt::Node::Link { target, text, .. } => {
            return Ok(Some(WSN::Link {
                text: nodes_wikitext(wikitext, text),
                title: target.to_string(),
            }));
        }
        pwt::Node::ExternalLink { nodes, .. } => {
            let inner = nodes_wikitext(wikitext, nodes);
            let (link, text) = inner
                .split_once(' ')
                .map(|(l, t)| (l, Some(t)))
                .unwrap_or((&inner, None));
            return Ok(Some(WSN::ExtLink {
                link: link.to_string(),
                text: text.map(|s| s.to_string()),
            }));
        }
        pwt::Node::Text { value, .. } => {
            return Ok(Some(WSN::Text {
                text: value.to_string(),
            }));
        }
        pwt::Node::CharacterEntity { character, .. } => {
            return Ok(Some(WSN::Text {
                text: character.to_string(),
            }));
        }
        pwt::Node::ParagraphBreak { .. } => {
            return Ok(Some(WSN::ParagraphBreak));
        }
        pwt::Node::Category { .. } | pwt::Node::Comment { .. } | pwt::Node::Image { .. } => {
            // Don't care
            return Ok(None);
        }
        pwt::Node::DefinitionList { .. }
        | pwt::Node::OrderedList { .. }
        | pwt::Node::UnorderedList { .. } => {
            // Temporarily ignore these
            return Ok(None);
        }
        pwt::Node::Tag { name, .. }
            if ["nowiki", "references", "gallery", "ref"].contains(&name.as_ref()) =>
        {
            // Don't care
            return Ok(None);
        }
        pwt::Node::StartTag { name, .. } if name == "br" => {
            return Ok(Some(WSN::Newline));
        }
        pwt::Node::Preformatted { nodes, .. } => {
            return Ok(Some(WSN::Preformatted {
                children: simplify_wikitext_nodes(wikitext, nodes)?,
            }));
        }
        _ => {}
    }
    let metadata = NodeMetadata::for_node(node);
    Err(WikitextError::SimplificationError {
        node_type: "Unknown".into(),
        context: WikitextErrorContext::from_node_metadata(wikitext, &metadata),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::LazyLock;

    use wikitext_util::wikipedia_pwt_configuration;
    use WikitextSimplifiedNode as WSN;

    static PWT_CONFIGURATION: LazyLock<pwt::Configuration> =
        LazyLock::new(wikipedia_pwt_configuration);

    #[test]
    fn test_s_after_link() {
        let wikitext = "cool [[thing]]s by cool [[Person|person]]s";
        let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
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
        let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
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
        let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
        assert_eq!(simplified, vec![]);
    }
}
