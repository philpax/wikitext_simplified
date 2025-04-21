use serde::{Deserialize, Serialize};

use parse_wiki_text_2 as pwt;
use wikitext_util::{nodes_inner_text, nodes_wikitext, NodeMetadata};

#[cfg(feature = "wasm")]
use tsify_next::Tsify;

/// Errors that can occur during simplification of wikitext nodes
#[derive(Debug)]
pub enum SimplificationError {
    /// An unknown node type was encountered
    UnknownNode {
        /// The type of node that caused the error
        node_type: &'static str,
        /// The context of where the error occurred
        context: SimplificationErrorContext,
    },
    /// Error occurred due to invalid node structure
    InvalidNodeStructure {
        /// The specific type of structural error
        kind: NodeStructureError,
        /// The context of where the error occurred
        context: SimplificationErrorContext,
    },
}
impl std::fmt::Display for SimplificationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SimplificationError::UnknownNode { node_type, context } => {
                write!(
                    f,
                    "Unknown node type '{}' at position {}-{}: '{}'",
                    node_type, context.start, context.end, context.content
                )
            }
            SimplificationError::InvalidNodeStructure { kind, context } => {
                write!(
                    f,
                    "Invalid node structure: {} at position {}-{}: '{}'",
                    kind, context.start, context.end, context.content
                )
            }
        }
    }
}
impl std::error::Error for SimplificationError {}

/// Context information for simplification errors that occur at specific
/// positions in the wikitext
#[derive(Debug)]
pub struct SimplificationErrorContext {
    /// The problematic content from the wikitext
    pub content: String,
    /// The start position of the problematic content
    pub start: usize,
    /// The end position of the problematic content
    pub end: usize,
}
impl SimplificationErrorContext {
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
    NoChildren {
        /// The type of node that has no children
        parent_node_type: &'static str,
    },
    /// Found a bold-italic node without a corresponding bold node
    MissingBoldLayer,
    /// Found an unclosed formatting node
    UnclosedFormatting,
    /// Found a tag closure mismatch, where the closing tag does not match the opening tag
    TagClosureMismatch {
        /// The expected tag name
        expected: String,
        /// The actual tag name
        actual: String,
    },
}
impl std::fmt::Display for NodeStructureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeStructureError::StackUnderflow => write!(f, "Stack underflow"),
            NodeStructureError::StackOverflow => write!(f, "Stack overflow"),
            NodeStructureError::NoChildren { parent_node_type } => {
                write!(f, "Node of type '{}' has no children", parent_node_type)
            }
            NodeStructureError::MissingBoldLayer => {
                write!(f, "Bold-italic found without a bold layer")
            }
            NodeStructureError::UnclosedFormatting => write!(f, "Unclosed formatting node"),
            NodeStructureError::TagClosureMismatch { expected, actual } => {
                write!(
                    f,
                    "Tag closure mismatch: {} (expected {})",
                    actual, expected
                )
            }
        }
    }
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
    /// A use of a parameter within a template
    TemplateParameterUse {
        /// The name of the parameter
        name: String,
        /// Default, if available
        default: Option<Vec<WikitextSimplifiedNode>>,
    },
    /// A heading node
    Heading {
        /// The level of the heading
        level: u8,
        /// The content within the heading
        children: Vec<WikitextSimplifiedNode>,
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
    /// An arbitrary tag.
    Tag {
        /// The name of the tag
        name: String,
        /// The content within the tag
        children: Vec<WikitextSimplifiedNode>,
    },
    /// Plain text content
    Text {
        /// The text content
        text: String,
    },
    /// A table
    Table {
        /// The HTML attributes of the table
        attributes: String,
        /// The captions of the table
        captions: Vec<WikitextSimplifiedTableCaption>,
        /// The rows of the table
        rows: Vec<WikitextSimplifiedTableRow>,
    },
    /// A redirect node
    Redirect {
        /// The target page of the redirect
        target: String,
    },
    /// A horizontal divider
    HorizontalDivider,
    /// A paragraph break
    ParagraphBreak,
    /// A line break
    Newline,
}
/// A caption for a table
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct WikitextSimplifiedTableCaption {
    /// The HTML attributes of the caption
    pub attributes: Option<String>,
    /// The content of the caption
    pub content: Vec<WikitextSimplifiedNode>,
}
/// A row in a table
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct WikitextSimplifiedTableRow {
    /// The HTML attributes of the row
    pub attributes: Option<String>,
    /// The cells in the row
    pub cells: Vec<WikitextSimplifiedTableCell>,
}
/// A cell in a table
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct WikitextSimplifiedTableCell {
    /// The HTML attributes of the cell
    pub attributes: Option<String>,
    /// The content of the cell
    pub content: Vec<WikitextSimplifiedNode>,
}
impl WikitextSimplifiedNode {
    /// Returns the type of this node.
    pub fn node_type(&self) -> &'static str {
        match self {
            Self::Fragment { .. } => "fragment",
            Self::Template { .. } => "template",
            Self::TemplateParameterUse { .. } => "template-parameter-use",
            Self::Heading { .. } => "heading",
            Self::Link { .. } => "link",
            Self::ExtLink { .. } => "ext-link",
            Self::Bold { .. } => "bold",
            Self::Italic { .. } => "italic",
            Self::Blockquote { .. } => "blockquote",
            Self::Superscript { .. } => "superscript",
            Self::Subscript { .. } => "subscript",
            Self::Small { .. } => "small",
            Self::Preformatted { .. } => "preformatted",
            Self::Tag { .. } => "tag",
            Self::Text { .. } => "text",
            Self::Table { .. } => "table",
            Self::Redirect { .. } => "redirect",
            Self::HorizontalDivider => "horizontal-divider",
            Self::ParagraphBreak => "paragraph-break",
            Self::Newline => "newline",
        }
    }

    /// Returns a reference to the children of this node, if it has any.
    ///
    /// This method returns `Some` for node types that can contain children (like `Fragment`,
    /// `Bold`, `Italic`, etc.) and `None` for leaf nodes (like `Text`, `ParagraphBreak`).
    pub fn children(&self) -> Option<&[WikitextSimplifiedNode]> {
        match self {
            Self::Fragment { children } => Some(children),
            Self::Heading { children, .. } => Some(children),
            Self::Bold { children } => Some(children),
            Self::Italic { children } => Some(children),
            Self::Blockquote { children } => Some(children),
            Self::Superscript { children } => Some(children),
            Self::Subscript { children } => Some(children),
            Self::Small { children } => Some(children),
            Self::Preformatted { children } => Some(children),
            Self::Tag { children, .. } => Some(children),

            Self::Template { .. }
            | Self::TemplateParameterUse { .. }
            | Self::Link { .. }
            | Self::ExtLink { .. }
            | Self::Text { .. }
            | Self::Table { .. }
            | Self::Redirect { .. }
            | Self::HorizontalDivider
            | Self::ParagraphBreak
            | Self::Newline => None,
        }
    }

    /// Returns a mutable reference to the children of this node, if it has any.
    ///
    /// This method returns `Some` for node types that can contain children (like `Fragment`,
    /// `Bold`, `Italic`, etc.) and `None` for leaf nodes (like `Text`, `ParagraphBreak`).
    pub fn children_mut(&mut self) -> Option<&mut Vec<WikitextSimplifiedNode>> {
        match self {
            Self::Fragment { children } => Some(children),
            Self::Heading { children, .. } => Some(children),
            Self::Bold { children } => Some(children),
            Self::Italic { children } => Some(children),
            Self::Blockquote { children } => Some(children),
            Self::Superscript { children } => Some(children),
            Self::Subscript { children } => Some(children),
            Self::Small { children } => Some(children),
            Self::Preformatted { children } => Some(children),
            Self::Tag { children, .. } => Some(children),

            Self::Template { .. }
            | Self::TemplateParameterUse { .. }
            | Self::Link { .. }
            | Self::ExtLink { .. }
            | Self::Text { .. }
            | Self::Table { .. }
            | Self::Redirect { .. }
            | Self::HorizontalDivider
            | Self::ParagraphBreak
            | Self::Newline => None,
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
pub fn simplify_wikitext_nodes(
    wikitext: &str,
    nodes: &[pwt::Node],
) -> Result<Vec<WikitextSimplifiedNode>, SimplificationError> {
    use WikitextSimplifiedNode as WSN;
    let mut root_stack = RootStack::new(wikitext);

    // Awful hack to deal with templates: special-case single start/end tags and preserve them as texts
    if nodes.len() == 1 {
        match &nodes[0] {
            pwt::Node::StartTag { .. } => {
                return Ok(vec![WSN::Text {
                    text: nodes_wikitext(wikitext, nodes),
                }]);
            }
            pwt::Node::EndTag { .. } => {
                return Ok(vec![WSN::Text {
                    text: nodes_wikitext(wikitext, nodes),
                }]);
            }
            _ => {}
        }
    }

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
                        return Err(SimplificationError::InvalidNodeStructure {
                            kind: NodeStructureError::MissingBoldLayer,
                            context: SimplificationErrorContext::from_node_metadata(
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
                assert_tag_closure_matches(name, "blockquote")?;
                root_stack.add_to_children(blockquote)?;
            }
            pwt::Node::StartTag { name, .. } if name == "sup" => {
                root_stack.push_layer(WSN::Superscript { children: vec![] });
            }
            pwt::Node::EndTag { name, .. } if name == "sup" => {
                let superscript = root_stack.pop_layer()?;
                assert_tag_closure_matches(name, "sup")?;
                root_stack.add_to_children(superscript)?;
            }
            pwt::Node::StartTag { name, .. } if name == "sub" => {
                root_stack.push_layer(WSN::Subscript { children: vec![] });
            }
            pwt::Node::EndTag { name, .. } if name == "sub" => {
                let subscript = root_stack.pop_layer()?;
                assert_tag_closure_matches(name, "sub")?;
                root_stack.add_to_children(subscript)?;
            }
            pwt::Node::StartTag { name, .. } if name == "small" => {
                root_stack.push_layer(WSN::Small { children: vec![] });
            }
            pwt::Node::EndTag { name, .. } if name == "small" => {
                let small = root_stack.pop_layer()?;
                assert_tag_closure_matches(name, "small")?;
                root_stack.add_to_children(small)?;
            }
            pwt::Node::StartTag { name, .. } if name == "pre" => {
                root_stack.push_layer(WSN::Preformatted { children: vec![] });
            }
            pwt::Node::EndTag { name, .. } if name == "pre" => {
                let preformatted = root_stack.pop_layer()?;
                assert_tag_closure_matches(name, "pre")?;
                root_stack.add_to_children(preformatted)?;
            }
            pwt::Node::StartTag { name, .. } => {
                root_stack.push_layer(WSN::Tag {
                    name: name.to_string(),
                    children: vec![],
                });
            }
            pwt::Node::EndTag { name, .. } => {
                let tag = root_stack.pop_layer()?;
                if let WSN::Tag { name: tag_name, .. } = &tag {
                    assert_tag_closure_matches(name, &tag_name)?;
                } else {
                    return Err(SimplificationError::InvalidNodeStructure {
                        kind: NodeStructureError::TagClosureMismatch {
                            expected: name.to_string(),
                            actual: tag.node_type().to_string(),
                        },
                        context: SimplificationErrorContext::from_node_metadata(
                            wikitext,
                            &NodeMetadata::for_node(node),
                        ),
                    });
                }
                root_stack.add_to_children(tag)?;
            }
            other => {
                if let Some(simplified_node) = simplify_wikitext_node(wikitext, other)? {
                    root_stack.add_to_children(simplified_node)?;
                }
            }
        }
    }

    fn assert_tag_closure_matches(
        end_tag_name: &str,
        last_node_name: &str,
    ) -> Result<(), SimplificationError> {
        match last_node_name {
            name if name == end_tag_name => Ok(()),
            _ => Err(SimplificationError::InvalidNodeStructure {
                kind: NodeStructureError::TagClosureMismatch {
                    expected: end_tag_name.to_string(),
                    actual: last_node_name.to_string(),
                },
                context: SimplificationErrorContext {
                    // Filling this in requires us to have the original bounds and wikitext,
                    // which we don't have here. This can be fixed, but it would require a more
                    // significant refactor.
                    content: "TODO: Fill in context for tag closure mismatch".into(),
                    start: 0,
                    end: 0,
                },
            }),
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
) -> Result<Option<WikitextSimplifiedNode>, SimplificationError> {
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
        pwt::Node::Heading { level, nodes, .. } => {
            return Ok(Some(WSN::Heading {
                level: *level,
                children: simplify_wikitext_nodes(wikitext, nodes)?,
            }));
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
        pwt::Node::Table {
            attributes,
            captions,
            rows,
            ..
        } => {
            // Convert table attributes to a string
            let attributes_str = nodes_wikitext(wikitext, attributes);

            // Convert captions
            let mut simplified_captions = Vec::new();
            for caption in captions {
                let caption_attributes = caption
                    .attributes
                    .as_ref()
                    .map(|attrs| nodes_wikitext(wikitext, attrs));
                let caption_content = simplify_wikitext_nodes(wikitext, &caption.content)?;
                simplified_captions.push(WikitextSimplifiedTableCaption {
                    attributes: caption_attributes,
                    content: caption_content,
                });
            }

            // Convert rows
            let mut simplified_rows = Vec::new();
            for row in rows {
                let row_attributes = if !row.attributes.is_empty() {
                    Some(nodes_wikitext(wikitext, &row.attributes))
                } else {
                    None
                };

                let mut cells = Vec::new();
                for cell in &row.cells {
                    let cell_attributes = cell
                        .attributes
                        .as_ref()
                        .map(|attrs| nodes_wikitext(wikitext, attrs));
                    let cell_content = simplify_wikitext_nodes(wikitext, &cell.content)?;
                    cells.push(WikitextSimplifiedTableCell {
                        attributes: cell_attributes,
                        content: cell_content,
                    });
                }

                simplified_rows.push(WikitextSimplifiedTableRow {
                    attributes: row_attributes,
                    cells,
                });
            }

            return Ok(Some(WSN::Table {
                attributes: attributes_str,
                captions: simplified_captions,
                rows: simplified_rows,
            }));
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
        pwt::Node::Tag { name, nodes, .. } => {
            return Ok(Some(WSN::Tag {
                name: name.to_string(),
                children: simplify_wikitext_nodes(wikitext, nodes)?,
            }));
        }
        pwt::Node::Preformatted { nodes, .. } => {
            return Ok(Some(WSN::Preformatted {
                children: simplify_wikitext_nodes(wikitext, nodes)?,
            }));
        }
        pwt::Node::Parameter { name, default, .. } => {
            return Ok(Some(WSN::TemplateParameterUse {
                name: nodes_inner_text(name),
                default: default
                    .as_deref()
                    .map(|nodes| simplify_wikitext_nodes(wikitext, nodes))
                    .transpose()?,
            }));
        }
        pwt::Node::Redirect { target, .. } => {
            return Ok(Some(WSN::Redirect {
                target: target.to_string(),
            }));
        }
        pwt::Node::HorizontalDivider { .. } => {
            return Ok(Some(WSN::HorizontalDivider));
        }
        _ => {}
    }
    let metadata = NodeMetadata::for_node(node);
    Err(SimplificationError::UnknownNode {
        node_type: metadata.name,
        context: SimplificationErrorContext::from_node_metadata(wikitext, &metadata),
    })
}

struct RootStack<'a> {
    stack: Vec<WikitextSimplifiedNode>,
    wikitext: &'a str,
    current_node: Option<&'a pwt::Node<'a>>,
}
impl<'a> RootStack<'a> {
    fn new(wikitext: &'a str) -> Self {
        Self {
            stack: vec![WikitextSimplifiedNode::Fragment { children: vec![] }],
            wikitext,
            current_node: None,
        }
    }

    fn push_layer(&mut self, node: WikitextSimplifiedNode) {
        self.stack.push(node);
    }

    fn pop_layer(&mut self) -> Result<WikitextSimplifiedNode, SimplificationError> {
        self.stack
            .pop()
            .ok_or_else(|| SimplificationError::InvalidNodeStructure {
                kind: NodeStructureError::StackUnderflow,
                context: Self::error_context_for_current_node(self.wikitext, self.current_node),
            })
    }

    fn last_layer(&self) -> &WikitextSimplifiedNode {
        self.stack.last().unwrap()
    }

    fn add_to_children(&mut self, node: WikitextSimplifiedNode) -> Result<(), SimplificationError> {
        let last_layer =
            self.stack
                .last_mut()
                .ok_or_else(|| SimplificationError::InvalidNodeStructure {
                    kind: NodeStructureError::StackUnderflow,
                    context: Self::error_context_for_current_node(self.wikitext, self.current_node),
                })?;
        let parent_node_type = last_layer.node_type();

        last_layer
            .children_mut()
            .ok_or_else(|| SimplificationError::InvalidNodeStructure {
                kind: NodeStructureError::NoChildren { parent_node_type },
                context: Self::error_context_for_current_node(self.wikitext, self.current_node),
            })?
            .push(node);

        Ok(())
    }

    fn unwind(mut self) -> Result<Vec<WikitextSimplifiedNode>, SimplificationError> {
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
    ) -> SimplificationErrorContext {
        current_node
            .map(|node| {
                SimplificationErrorContext::from_node_metadata(
                    wikitext,
                    &NodeMetadata::for_node(node),
                )
            })
            .unwrap_or_else(|| SimplificationErrorContext {
                content: "No current node".into(),
                start: 0,
                end: 0,
            })
    }
}
