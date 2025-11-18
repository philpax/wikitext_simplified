use serde::{Deserialize, Serialize};

use parse_wiki_text_2 as pwt;
use wikitext_util::{nodes_inner_text, nodes_wikitext, NodeMetadata, NodeMetadataType};

#[cfg(feature = "wasm")]
use tsify_next::Tsify;

/// A span representing a range in the source text
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct Span {
    /// The start position in the source text
    pub start: usize,
    /// The end position in the source text
    pub end: usize,
}

/// A helper type that wraps a value with its span in the source text
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct Spanned<T> {
    /// The wrapped value
    pub value: T,
    /// The span of this value in the source text
    pub span: Span,
}

/// Errors that can occur during simplification of wikitext nodes
#[derive(Debug)]
pub enum SimplificationError {
    /// An unknown node type was encountered
    UnknownNode {
        /// The type of node that caused the error
        node_type: NodeMetadataType,
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
                    "Unknown node type '{:?}' at position {}-{}: '{}'",
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
                write!(f, "Node of type '{parent_node_type}' has no children")
            }
            NodeStructureError::MissingBoldLayer => {
                write!(f, "Bold-italic found without a bold layer")
            }
            NodeStructureError::UnclosedFormatting => write!(f, "Unclosed formatting node"),
            NodeStructureError::TagClosureMismatch { expected, actual } => {
                write!(f, "Tag closure mismatch: {actual} (expected {expected})")
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
        children: Vec<Spanned<WikitextSimplifiedNode>>,
    },
    /// A template node, representing a wikitext template
    Template {
        /// The name of the template
        name: String,
        /// The parameters passed to the template
        parameters: Vec<TemplateParameter>,
    },
    /// A use of a parameter within a template
    TemplateParameterUse {
        /// The name of the parameter
        name: String,
        /// Default, if available
        default: Option<Vec<Spanned<WikitextSimplifiedNode>>>,
    },
    /// A heading node
    Heading {
        /// The level of the heading
        level: u8,
        /// The content within the heading
        children: Vec<Spanned<WikitextSimplifiedNode>>,
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
        children: Vec<Spanned<WikitextSimplifiedNode>>,
    },
    /// Italic text formatting
    Italic {
        /// The content within the italic formatting
        children: Vec<Spanned<WikitextSimplifiedNode>>,
    },
    /// Blockquote formatting
    Blockquote {
        /// The content within the blockquote
        children: Vec<Spanned<WikitextSimplifiedNode>>,
    },
    /// Superscript text formatting
    Superscript {
        /// The content within the superscript formatting
        children: Vec<Spanned<WikitextSimplifiedNode>>,
    },
    /// Subscript text formatting
    Subscript {
        /// The content within the subscript formatting
        children: Vec<Spanned<WikitextSimplifiedNode>>,
    },
    /// Small text formatting
    Small {
        /// The content within the small text formatting
        children: Vec<Spanned<WikitextSimplifiedNode>>,
    },
    /// Preformatted text
    Preformatted {
        /// The content within the preformatted block
        children: Vec<Spanned<WikitextSimplifiedNode>>,
    },
    /// An arbitrary tag.
    Tag {
        /// The name of the tag
        name: String,
        /// The HTML attributes of the tag
        attributes: Option<String>,
        /// The content within the tag
        children: Vec<Spanned<WikitextSimplifiedNode>>,
    },
    /// Plain text content
    Text {
        /// The text content
        text: String,
    },
    /// A table
    Table {
        /// The HTML attributes of the table
        attributes: Vec<Spanned<WikitextSimplifiedNode>>,
        /// The captions of the table
        captions: Vec<WikitextSimplifiedTableCaption>,
        /// The rows of the table
        rows: Vec<WikitextSimplifiedTableRow>,
    },
    /// An ordered list
    OrderedList {
        /// The items in the list
        items: Vec<WikitextSimplifiedListItem>,
    },
    /// An unordered list
    UnorderedList {
        /// The items in the list
        items: Vec<WikitextSimplifiedListItem>,
    },
    /// A definition list
    DefinitionList {
        /// The items in the list
        items: Vec<WikitextSimplifiedDefinitionListItem>,
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
    pub attributes: Option<Vec<Spanned<WikitextSimplifiedNode>>>,
    /// The content of the caption
    pub content: Vec<Spanned<WikitextSimplifiedNode>>,
}
/// A row in a table
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct WikitextSimplifiedTableRow {
    /// The HTML attributes of the row
    pub attributes: Vec<Spanned<WikitextSimplifiedNode>>,
    /// The cells in the row
    pub cells: Vec<WikitextSimplifiedTableCell>,
}
/// A cell in a table
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct WikitextSimplifiedTableCell {
    /// Whether this cell is a header cell (`!` syntax)
    pub is_header: bool,
    /// The HTML attributes of the cell
    pub attributes: Option<Vec<Spanned<WikitextSimplifiedNode>>>,
    /// The content of the cell
    pub content: Vec<Spanned<WikitextSimplifiedNode>>,
}
/// A list item
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct WikitextSimplifiedListItem {
    /// The content of the list item
    pub content: Vec<Spanned<WikitextSimplifiedNode>>,
}
/// A list item in a definition list
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct WikitextSimplifiedDefinitionListItem {
    /// The type of list item
    pub type_: DefinitionListItemType,
    /// The content of the list item
    pub content: Vec<Spanned<WikitextSimplifiedNode>>,
}
/// The type of a definition list item
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub enum DefinitionListItemType {
    /// A term item (;)
    Term,
    /// A details item (:)
    Details,
}
impl std::fmt::Display for DefinitionListItemType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Term => write!(f, ";"),
            Self::Details => write!(f, ":"),
        }
    }
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
            Self::OrderedList { .. } => "ordered-list",
            Self::UnorderedList { .. } => "unordered-list",
            Self::DefinitionList { .. } => "definition-list",
            Self::Redirect { .. } => "redirect",
            Self::HorizontalDivider => "horizontal-divider",
            Self::ParagraphBreak => "paragraph-break",
            Self::Newline => "newline",
        }
    }

    /// Returns a reference to the immediate children of this node, if it has any.
    /// This does not include "deep" children in tables, lists, etc.
    ///
    /// This method returns `Some` for node types that can contain children (like `Fragment`,
    /// `Bold`, `Italic`, etc.) and `None` for leaf nodes (like `Text`, `ParagraphBreak`).
    pub fn children(&self) -> Option<&[Spanned<WikitextSimplifiedNode>]> {
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
            | Self::OrderedList { .. }
            | Self::UnorderedList { .. }
            | Self::DefinitionList { .. }
            | Self::Redirect { .. }
            | Self::HorizontalDivider
            | Self::ParagraphBreak
            | Self::Newline => None,
        }
    }

    /// Returns a mutable reference to the immediate children of this node, if it has any.
    /// This does not include "deep" children in tables, lists, etc.
    ///
    /// This method returns `Some` for node types that can contain children (like `Fragment`,
    /// `Bold`, `Italic`, etc.) and `None` for leaf nodes (like `Text`, `ParagraphBreak`).
    pub fn children_mut(&mut self) -> Option<&mut Vec<Spanned<WikitextSimplifiedNode>>> {
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
            | Self::OrderedList { .. }
            | Self::UnorderedList { .. }
            | Self::DefinitionList { .. }
            | Self::Redirect { .. }
            | Self::HorizontalDivider
            | Self::ParagraphBreak
            | Self::Newline => None,
        }
    }

    /// Returns `true` if this node is a block-level node.
    ///
    /// Block-level nodes are nodes that can contain other nodes, such as headings, tables, lists, etc.
    pub fn is_block_type(&self) -> bool {
        matches!(
            self,
            Self::Heading { .. }
                | Self::Table { .. }
                | Self::OrderedList { .. }
                | Self::UnorderedList { .. }
                | Self::DefinitionList { .. }
        )
    }

    /// Converts this node and its children back into wikitext format.
    pub fn to_wikitext(&self) -> String {
        fn nodes_to_wikitext(nodes: &[Spanned<WikitextSimplifiedNode>]) -> String {
            let mut output = String::new();
            for node in nodes {
                if node.value.is_block_type() {
                    output.push('\n');
                }
                output.push_str(&node.value.to_wikitext());
            }
            output
        }

        match self {
            Self::Fragment { children } => nodes_to_wikitext(children),
            Self::Template { name, parameters } => {
                let params = parameters
                    .iter()
                    .map(|param| {
                        if param.name == "1" {
                            param.value.clone()
                        } else if param.name.parse::<usize>().is_ok() {
                            // For numeric parameters, just use the value
                            param.value.clone()
                        } else {
                            format!("{}={}", param.name, param.value)
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("|");
                format!(
                    "{{{{{}}}}}",
                    if params.is_empty() {
                        name.clone()
                    } else {
                        format!("{name}|{params}")
                    }
                )
            }
            Self::TemplateParameterUse { name, default } => {
                let mut result = format!("{{{{{name}}}}}");
                if let Some(default_nodes) = default {
                    result.push('|');
                    result.push_str(&nodes_to_wikitext(default_nodes));
                }
                result
            }
            Self::Heading { level, children } => {
                let equals = "=".repeat(*level as usize);
                format!("{} {} {}", equals, nodes_to_wikitext(children), equals)
            }
            Self::Link { text, title } => {
                if text == title {
                    format!("[[{title}]]")
                } else {
                    format!("[[{title}|{text}]]")
                }
            }
            Self::ExtLink { link, text } => {
                if let Some(text) = text {
                    format!("[{link} {text}]")
                } else {
                    format!("[{link}]")
                }
            }
            Self::Bold { children } => {
                format!("'''{}'''", nodes_to_wikitext(children))
            }
            Self::Italic { children } => {
                format!("''{}''", nodes_to_wikitext(children))
            }
            Self::Blockquote { children } => {
                format!("<blockquote>{}</blockquote>", nodes_to_wikitext(children))
            }
            Self::Superscript { children } => {
                format!("<sup>{}</sup>", nodes_to_wikitext(children))
            }
            Self::Subscript { children } => {
                format!("<sub>{}</sub>", nodes_to_wikitext(children))
            }
            Self::Small { children } => {
                format!("<small>{}</small>", nodes_to_wikitext(children))
            }
            Self::Preformatted { children } => {
                format!("<pre>{}</pre>", nodes_to_wikitext(children))
            }
            Self::Tag {
                name,
                attributes,
                children,
            } => {
                let attrs = attributes.as_deref().unwrap_or("");
                let space = if attrs.is_empty() { "" } else { " " };
                format!(
                    "<{}{}{}>{}</{}>",
                    name,
                    space,
                    attrs,
                    nodes_to_wikitext(children),
                    name
                )
            }
            Self::Text { text } => text.replace('\u{a0}', "&nbsp;"),
            Self::Table {
                attributes,
                captions,
                rows,
            } => {
                let mut result = format!("{{|{}\n", nodes_to_wikitext(attributes));

                // Add captions
                for caption in captions {
                    result.push_str("|+");
                    if let Some(attrs) = &caption.attributes {
                        result.push_str(&format!(" {}", nodes_to_wikitext(attrs)));
                    }
                    result.push_str(&nodes_to_wikitext(&caption.content));
                    result.push_str("\n|-\n");
                }

                // Add rows
                for (row_idx, row) in rows.iter().enumerate() {
                    if row_idx > 0 {
                        result.push_str("|-\n");
                    }
                    if !row.attributes.is_empty() {
                        result.push_str(&format!("|- {}\n", nodes_to_wikitext(&row.attributes)));
                    }

                    for (idx, cell) in row.cells.iter().enumerate() {
                        if cell.is_header {
                            result.push('!');
                        } else {
                            result.push('|');
                        }
                        if let Some(attrs) = &cell.attributes {
                            result.push_str(&nodes_to_wikitext(attrs));
                            result.push('|');
                        }
                        result.push_str(&nodes_to_wikitext(&cell.content));
                        if idx < row.cells.len() - 1 {
                            let next_is_header = row.cells[idx + 1].is_header;
                            if cell.is_header != next_is_header {
                                result.push('\n');
                            }
                        }
                    }
                    result.push('\n');
                }

                result.push_str("|}\n");
                result
            }
            Self::OrderedList { items } => {
                let mut result = String::new();
                for item in items {
                    result.push('#');
                    result.push_str(&nodes_to_wikitext(&item.content));
                    result.push('\n');
                }
                result
            }
            Self::UnorderedList { items } => {
                let mut result = String::new();
                for item in items {
                    result.push('*');
                    result.push_str(&nodes_to_wikitext(&item.content));
                    result.push('\n');
                }
                result
            }
            Self::DefinitionList { items } => {
                let mut result = String::new();
                for item in items {
                    result.push_str(&item.type_.to_string());
                    result.push_str(&nodes_to_wikitext(&item.content));
                    result.push('\n');
                }
                result
            }
            Self::Redirect { target } => {
                format!("#REDIRECT [[{target}]]")
            }
            Self::HorizontalDivider => "----".to_string(),
            Self::ParagraphBreak => "<br/>".to_string(),
            Self::Newline => "\n".to_string(),
        }
    }
}
// Visitors
macro_rules! visit_children_impl {
    ($self:expr, $visitor:expr, $visit_method:ident, $iter_method:ident) => {
        match $self {
            Self::Fragment { children }
            | Self::Heading { children, .. }
            | Self::Bold { children }
            | Self::Italic { children }
            | Self::Blockquote { children }
            | Self::Superscript { children }
            | Self::Subscript { children }
            | Self::Small { children }
            | Self::Preformatted { children }
            | Self::Tag { children, .. } => {
                for child in children {
                    child.value.$visit_method($visitor);
                }
            }

            Self::TemplateParameterUse { default, .. } => {
                if let Some(default) = default {
                    for child in default {
                        child.value.$visit_method($visitor);
                    }
                }
            }
            Self::Table {
                attributes,
                captions,
                rows,
                ..
            } => {
                for attr in attributes.$iter_method() {
                    attr.value.$visit_method($visitor);
                }
                for caption in captions.$iter_method().flat_map(|c| {
                    c.content
                        .$iter_method()
                        .chain(c.attributes.$iter_method().flat_map(|a| a.$iter_method()))
                }) {
                    caption.value.$visit_method($visitor);
                }
                for row in rows.$iter_method() {
                    for cell in row.cells.$iter_method().flat_map(|c| {
                        c.content
                            .$iter_method()
                            .chain(c.attributes.$iter_method().flat_map(|a| a.$iter_method()))
                    }) {
                        cell.value.$visit_method($visitor);
                    }
                }
            }
            Self::OrderedList { items } => {
                for item in items.$iter_method().flat_map(|i| i.content.$iter_method()) {
                    item.value.$visit_method($visitor);
                }
            }
            Self::UnorderedList { items } => {
                for item in items.$iter_method().flat_map(|i| i.content.$iter_method()) {
                    item.value.$visit_method($visitor);
                }
            }
            Self::DefinitionList { items } => {
                for item in items.$iter_method() {
                    for child in item.content.$iter_method() {
                        child.value.$visit_method($visitor);
                    }
                }
            }
            Self::Template { .. }
            | Self::Link { .. }
            | Self::ExtLink { .. }
            | Self::Text { .. }
            | Self::Redirect { .. }
            | Self::HorizontalDivider
            | Self::ParagraphBreak
            | Self::Newline => {}
        }
    };
}
impl WikitextSimplifiedNode {
    /// Visits this node and all its children recursively with the given visitor function,
    /// including "deep" children in tables, lists, and more.
    ///
    /// The visitor function is called on each node in depth-first order, starting with
    /// this node and then visiting all its children.
    pub fn visit(&self, visitor: &mut impl FnMut(&Self)) {
        visitor(self);
        visit_children_impl!(self, visitor, visit, iter);
    }

    /// Visits this node and all its children recursively with the given visitor function,
    /// including "deep" children in tables, lists, and more.
    ///
    /// The visitor function is called on each node in depth-first order, starting with
    /// this node and then visiting all its children.
    pub fn visit_mut(&mut self, visitor: &mut impl FnMut(&mut Self)) {
        visitor(self);
        visit_children_impl!(self, visitor, visit_mut, iter_mut);
    }

    /// Visits this node and all its children recursively with the given visitor function,
    /// replacing the node with the result of the visitor function.
    ///
    /// The visitor function is called on the children of each node first, and then on the node itself.
    pub fn visit_and_replace_mut(&mut self, visitor: &mut impl FnMut(&Self) -> Self) {
        visit_children_impl!(self, visitor, visit_and_replace_mut, iter_mut);
        *self = visitor(self);
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
) -> Result<Vec<Spanned<WikitextSimplifiedNode>>, SimplificationError> {
    use WikitextSimplifiedNode as WSN;
    let mut root_stack = RootStack::new(wikitext);

    // Awful hack to deal with templates: special-case single start/end tags and preserve them as texts
    if nodes.len() == 1 {
        match &nodes[0] {
            pwt::Node::StartTag { start, end, .. } => {
                return Ok(vec![Spanned {
                    value: WSN::Text {
                        text: nodes_wikitext(wikitext, nodes),
                    },
                    span: Span { start: *start, end: *end },
                }]);
            }
            pwt::Node::EndTag { start, end, .. } => {
                return Ok(vec![Spanned {
                    value: WSN::Text {
                        text: nodes_wikitext(wikitext, nodes),
                    },
                    span: Span { start: *start, end: *end },
                }]);
            }
            _ => {}
        }
    }

    /// Tags that look like tags but are actually inline elements and should
    /// not be considered for stack-based tag closure matching.
    const FAKE_TAGS: [&str; 4] = ["br/", "hr/", "br", "hr"];

    let mut text_start_override = None;
    for node in nodes {
        root_stack.set_current_node(node);
        match node {
            pwt::Node::Bold { start, end } => {
                if matches!(root_stack.last_layer(), WSN::Bold { .. }) {
                    let bold = root_stack.pop_layer(*end)?;
                    root_stack.add_to_children(bold)?;
                } else {
                    root_stack.push_layer(WSN::Bold { children: vec![] }, *start);
                }
            }
            pwt::Node::Italic { start, end } => {
                if matches!(root_stack.last_layer(), WSN::Italic { .. }) {
                    let italic = root_stack.pop_layer(*end)?;
                    root_stack.add_to_children(italic)?;
                } else {
                    root_stack.push_layer(WSN::Italic { children: vec![] }, *start);
                }
            }
            pwt::Node::BoldItalic { start, end } => {
                if matches!(root_stack.last_layer(), WSN::Italic { .. }) {
                    let italic = root_stack.pop_layer(*end)?;
                    if matches!(root_stack.last_layer(), WSN::Bold { .. }) {
                        let mut bold = root_stack.pop_layer(*end)?;
                        bold.value.children_mut().unwrap().push(italic);
                        root_stack.add_to_children(bold)?;
                    } else {
                        return Err(SimplificationError::InvalidNodeStructure {
                            kind: NodeStructureError::MissingBoldLayer,
                            context: SimplificationErrorContext {
                                content: wikitext[*start..*end].to_string(),
                                start: *start,
                                end: *end,
                            },
                        });
                    }
                } else {
                    root_stack.push_layer(WSN::Bold { children: vec![] }, *start);
                    root_stack.push_layer(WSN::Italic { children: vec![] }, *start);
                }
            }
            pwt::Node::StartTag { name, start, .. } if name == "blockquote" => {
                root_stack.push_layer(WSN::Blockquote { children: vec![] }, *start);
            }
            pwt::Node::EndTag { name, end, start } if name == "blockquote" => {
                let blockquote = root_stack.pop_layer(*end)?;
                assert_tag_closure_matches(wikitext, name, "blockquote", *start, *end)?;
                root_stack.add_to_children(blockquote)?;
            }
            pwt::Node::StartTag { name, start, .. } if name == "sup" => {
                root_stack.push_layer(WSN::Superscript { children: vec![] }, *start);
            }
            pwt::Node::EndTag { name, end, start } if name == "sup" => {
                let superscript = root_stack.pop_layer(*end)?;
                assert_tag_closure_matches(wikitext, name, "sup", *start, *end)?;
                root_stack.add_to_children(superscript)?;
            }
            pwt::Node::StartTag { name, start, .. } if name == "sub" => {
                root_stack.push_layer(WSN::Subscript { children: vec![] }, *start);
            }
            pwt::Node::EndTag { name, end, start } if name == "sub" => {
                let subscript = root_stack.pop_layer(*end)?;
                assert_tag_closure_matches(wikitext, name, "sub", *start, *end)?;
                root_stack.add_to_children(subscript)?;
            }
            pwt::Node::StartTag { name, start, .. } if name == "small" => {
                root_stack.push_layer(WSN::Small { children: vec![] }, *start);
            }
            pwt::Node::EndTag { name, end, start } if name == "small" => {
                let small = root_stack.pop_layer(*end)?;
                assert_tag_closure_matches(wikitext, name, "small", *start, *end)?;
                root_stack.add_to_children(small)?;
            }
            pwt::Node::StartTag { name, start, .. } if name == "pre" => {
                root_stack.push_layer(WSN::Preformatted { children: vec![] }, *start);
            }
            pwt::Node::EndTag { name, end, start } if name == "pre" => {
                let preformatted = root_stack.pop_layer(*end)?;
                assert_tag_closure_matches(wikitext, name, "pre", *start, *end)?;
                root_stack.add_to_children(preformatted)?;
            }
            pwt::Node::StartTag {
                name, start, end, ..
            } if !FAKE_TAGS.contains(&name.as_ref()) => {
                // Extract attributes from the tag content, e.g. <div class="foo"> -> class="foo"
                let tag_content = &wikitext[*start..*end];
                let closing_bracket_pos = tag_content.find('>').unwrap_or(tag_content.len());
                let opening_tag = &tag_content[..closing_bracket_pos];

                root_stack.push_layer(WSN::Tag {
                    name: name.to_string(),
                    attributes: extract_tag_attributes(opening_tag),
                    children: vec![],
                }, *start);
            }
            pwt::Node::EndTag { name, start, end } if !FAKE_TAGS.contains(&name.as_ref()) => {
                let tag = root_stack.pop_layer(*end)?;
                if let WSN::Tag { name: tag_name, .. } = &tag.value {
                    assert_tag_closure_matches(wikitext, name, tag_name, *start, *end)?;
                } else {
                    return Err(SimplificationError::InvalidNodeStructure {
                        kind: NodeStructureError::TagClosureMismatch {
                            expected: name.to_string(),
                            actual: tag.value.node_type().to_string(),
                        },
                        context: SimplificationErrorContext {
                            content: wikitext[*start..*end].to_string(),
                            start: *start,
                            end: *end,
                        },
                    });
                }
                root_stack.add_to_children(tag)?;
            }
            other => {
                if let Some(simplified_node) =
                    simplify_wikitext_node(wikitext, other, text_start_override)?
                {
                    // HACK: deal with `link_trail` by preserving the end of the link and forcing the next
                    // text to start at the end of the link
                    text_start_override = match &simplified_node.value {
                        WSN::Link { .. } => Some(simplified_node.span.end),
                        _ => None,
                    };

                    root_stack.add_to_children(simplified_node)?;
                }
            }
        }
    }

    fn assert_tag_closure_matches(
        wikitext: &str,
        end_tag_name: &str,
        last_node_name: &str,
        start: usize,
        end: usize,
    ) -> Result<(), SimplificationError> {
        if last_node_name == end_tag_name {
            return Ok(());
        }
        Err(SimplificationError::InvalidNodeStructure {
            kind: NodeStructureError::TagClosureMismatch {
                expected: end_tag_name.to_string(),
                actual: last_node_name.to_string(),
            },
            context: SimplificationErrorContext {
                content: wikitext[start..end].to_string(),
                start,
                end,
            },
        })
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
    text_start_override: Option<usize>,
) -> Result<Option<Spanned<WikitextSimplifiedNode>>, SimplificationError> {
    use WikitextSimplifiedNode as WSN;
    match node {
        pwt::Node::Template {
            name,
            parameters,
            start,
            end,
        } => {
            let mut unnamed_parameter_index = 1;
            let mut new_parameters = vec![];
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

                new_parameters.push(TemplateParameter { name, value });
            }

            return Ok(Some(Spanned {
                value: WSN::Template {
                    name: nodes_inner_text(name),
                    parameters: new_parameters,
                },
                span: Span { start: *start, end: *end },
            }));
        }
        pwt::Node::MagicWord { .. } => {
            // Making the current assumption that we don't care about these
            return Ok(None);
        }
        pwt::Node::Heading {
            level,
            nodes,
            start,
            end,
        } => {
            return Ok(Some(Spanned {
                value: WSN::Heading {
                    level: *level,
                    children: simplify_wikitext_nodes(wikitext, nodes)?,
                },
                span: Span { start: *start, end: *end },
            }));
        }
        pwt::Node::Bold { .. } | pwt::Node::BoldItalic { .. } | pwt::Node::Italic { .. } => {
            // We can't do anything at this level
            return Ok(None);
        }
        pwt::Node::Link {
            target,
            text,
            start,
            end,
        } => {
            return Ok(Some(Spanned {
                value: WSN::Link {
                    text: nodes_wikitext(wikitext, text),
                    title: target.to_string(),
                },
                span: Span { start: *start, end: *end },
            }));
        }
        pwt::Node::ExternalLink { nodes, start, end } => {
            let inner = nodes_wikitext(wikitext, nodes);
            let (link, text) = inner
                .split_once(' ')
                .map(|(l, t)| (l, Some(t)))
                .unwrap_or((&inner, None));
            return Ok(Some(Spanned {
                value: WSN::ExtLink {
                    link: link.to_string(),
                    text: text.map(|s| s.to_string()),
                },
                span: Span { start: *start, end: *end },
            }));
        }
        pwt::Node::Text { value, start, end } => {
            let text_start = text_start_override.unwrap_or(*start);
            let text_start_offset = text_start.saturating_sub(*start);
            let text = &value[text_start_offset..];
            if text.is_empty() {
                return Ok(None);
            }
            return Ok(Some(Spanned {
                value: WSN::Text {
                    text: text.to_string(),
                },
                span: Span { start: text_start, end: *end },
            }));
        }
        pwt::Node::CharacterEntity {
            character,
            start,
            end,
        } => {
            return Ok(Some(Spanned {
                value: WSN::Text {
                    text: character.to_string(),
                },
                span: Span { start: *start, end: *end },
            }));
        }
        pwt::Node::ParagraphBreak { start, end } => {
            return Ok(Some(Spanned {
                value: WSN::ParagraphBreak,
                span: Span { start: *start, end: *end },
            }));
        }
        pwt::Node::Category { .. } | pwt::Node::Comment { .. } | pwt::Node::Image { .. } => {
            // Don't care
            return Ok(None);
        }
        pwt::Node::Table {
            attributes,
            captions,
            rows,
            start,
            end,
        } => {
            // Convert captions
            let mut simplified_captions = vec![];
            for caption in captions {
                let caption_content = simplify_wikitext_nodes(wikitext, &caption.content)?;
                simplified_captions.push(WikitextSimplifiedTableCaption {
                    attributes: caption
                        .attributes
                        .as_deref()
                        .map(|attrs| simplify_wikitext_nodes(wikitext, attrs))
                        .transpose()?,
                    content: caption_content,
                });
            }

            // Convert rows
            let mut simplified_rows = vec![];
            for row in rows {
                let mut cells = vec![];
                for cell in &row.cells {
                    let cell_content = simplify_wikitext_nodes(wikitext, &cell.content)?;
                    cells.push(WikitextSimplifiedTableCell {
                        is_header: cell.type_ == pwt::TableCellType::Heading,
                        attributes: cell
                            .attributes
                            .as_deref()
                            .map(|attrs| simplify_wikitext_nodes(wikitext, attrs))
                            .transpose()?,
                        content: cell_content,
                    });
                }

                simplified_rows.push(WikitextSimplifiedTableRow {
                    attributes: simplify_wikitext_nodes(wikitext, &row.attributes)?,
                    cells,
                });
            }

            return Ok(Some(Spanned {
                value: WSN::Table {
                    attributes: simplify_wikitext_nodes(wikitext, attributes)?,
                    captions: simplified_captions,
                    rows: simplified_rows,
                },
                span: Span { start: *start, end: *end },
            }));
        }
        pwt::Node::OrderedList { items, start, end } => {
            let mut simplified_items = vec![];
            for item in items {
                let content = simplify_wikitext_nodes(wikitext, &item.nodes)?;
                simplified_items.push(WikitextSimplifiedListItem { content });
            }
            return Ok(Some(Spanned {
                value: WSN::OrderedList {
                    items: simplified_items,
                },
                span: Span { start: *start, end: *end },
            }));
        }
        pwt::Node::UnorderedList { items, start, end } => {
            let mut simplified_items = vec![];
            for item in items {
                let content = simplify_wikitext_nodes(wikitext, &item.nodes)?;
                simplified_items.push(WikitextSimplifiedListItem { content });
            }
            return Ok(Some(Spanned {
                value: WSN::UnorderedList {
                    items: simplified_items,
                },
                span: Span { start: *start, end: *end },
            }));
        }
        pwt::Node::DefinitionList { items, start, end } => {
            let mut simplified_items = vec![];
            for item in items {
                let content = simplify_wikitext_nodes(wikitext, &item.nodes)?;
                simplified_items.push(WikitextSimplifiedDefinitionListItem {
                    type_: match item.type_ {
                        pwt::DefinitionListItemType::Term => DefinitionListItemType::Term,
                        pwt::DefinitionListItemType::Details => DefinitionListItemType::Details,
                    },
                    content,
                });
            }
            return Ok(Some(Spanned {
                value: WSN::DefinitionList {
                    items: simplified_items,
                },
                span: Span { start: *start, end: *end },
            }));
        }
        pwt::Node::Tag {
            name,
            nodes,
            start,
            end,
        } => {
            // Special handling for ref tags - ignore them
            if name == "ref" || name == "references" || name == "gallery" || name == "nowiki" {
                return Ok(None);
            }

            // Extract attributes from the opening tag content
            let tag_content = &wikitext[*start..*end];
            let closing_bracket_pos = tag_content.find('>').unwrap_or(tag_content.len());
            let opening_tag = &tag_content[..closing_bracket_pos];

            return Ok(Some(Spanned {
                value: WSN::Tag {
                    name: name.to_string(),
                    attributes: extract_tag_attributes(opening_tag),
                    children: simplify_wikitext_nodes(wikitext, nodes)?,
                },
                span: Span { start: *start, end: *end },
            }));
        }
        pwt::Node::Preformatted { nodes, start, end } => {
            return Ok(Some(Spanned {
                value: WSN::Preformatted {
                    children: simplify_wikitext_nodes(wikitext, nodes)?,
                },
                span: Span { start: *start, end: *end },
            }));
        }
        pwt::Node::Parameter {
            name,
            default,
            start,
            end,
        } => {
            return Ok(Some(Spanned {
                value: WSN::TemplateParameterUse {
                    name: nodes_inner_text(name),
                    default: default
                        .as_deref()
                        .map(|nodes| simplify_wikitext_nodes(wikitext, nodes))
                        .transpose()?,
                },
                span: Span { start: *start, end: *end },
            }));
        }
        pwt::Node::Redirect { target, start, end } => {
            return Ok(Some(Spanned {
                value: WSN::Redirect {
                    target: target.to_string(),
                },
                span: Span { start: *start, end: *end },
            }));
        }
        pwt::Node::HorizontalDivider { start, end } => {
            return Ok(Some(Spanned {
                value: WSN::HorizontalDivider,
                span: Span { start: *start, end: *end },
            }));
        }
        pwt::Node::StartTag { name, start, end } if name == "hr" || name == "hr/" => {
            return Ok(Some(Spanned {
                value: WSN::HorizontalDivider,
                span: Span { start: *start, end: *end },
            }));
        }
        pwt::Node::StartTag { name, start, end } if name == "br" || name == "br/" => {
            return Ok(Some(Spanned {
                value: WSN::ParagraphBreak,
                span: Span { start: *start, end: *end },
            }));
        }
        _ => {}
    }
    let metadata = NodeMetadata::for_node(node);
    Err(SimplificationError::UnknownNode {
        node_type: metadata.ty,
        context: SimplificationErrorContext::from_node_metadata(wikitext, &metadata),
    })
}

struct RootStack<'a> {
    stack: Vec<(WikitextSimplifiedNode, usize)>,
    wikitext: &'a str,
    current_node: Option<&'a pwt::Node<'a>>,
}
impl<'a> RootStack<'a> {
    fn new(wikitext: &'a str) -> Self {
        Self {
            stack: vec![(WikitextSimplifiedNode::Fragment { children: vec![] }, 0)],
            wikitext,
            current_node: None,
        }
    }

    fn push_layer(&mut self, node: WikitextSimplifiedNode, start: usize) {
        self.stack.push((node, start));
    }

    fn pop_layer(&mut self, end: usize) -> Result<Spanned<WikitextSimplifiedNode>, SimplificationError> {
        let (node, start) = self.stack
            .pop()
            .ok_or_else(|| SimplificationError::InvalidNodeStructure {
                kind: NodeStructureError::StackUnderflow,
                context: Self::error_context_for_current_node(self.wikitext, self.current_node),
            })?;
        Ok(Spanned {
            value: node,
            span: Span { start, end },
        })
    }

    fn last_layer(&self) -> &WikitextSimplifiedNode {
        &self.stack.last().unwrap().0
    }

    fn add_to_children(&mut self, node: Spanned<WikitextSimplifiedNode>) -> Result<(), SimplificationError> {
        let last_layer =
            self.stack
                .last_mut()
                .ok_or_else(|| SimplificationError::InvalidNodeStructure {
                    kind: NodeStructureError::StackUnderflow,
                    context: Self::error_context_for_current_node(self.wikitext, self.current_node),
                })?;
        let parent_node_type = last_layer.0.node_type();

        last_layer.0
            .children_mut()
            .ok_or_else(|| SimplificationError::InvalidNodeStructure {
                kind: NodeStructureError::NoChildren { parent_node_type },
                context: Self::error_context_for_current_node(self.wikitext, self.current_node),
            })?
            .push(node);

        Ok(())
    }

    fn unwind(mut self) -> Result<Vec<Spanned<WikitextSimplifiedNode>>, SimplificationError> {
        // This is a disgusting hack, but Wikipedia implicitly closes these, so we need to as well...
        // Use the end of wikitext as the end position for implicitly closed tags
        let wikitext_end = self.wikitext.len();
        while self.stack.len() > 1 {
            let popped = self.pop_layer(wikitext_end)?;
            self.add_to_children(popped)?;
        }
        Ok(self.stack[0].0.children().unwrap().to_vec())
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

/// Helper function to extract attributes from an HTML tag's opening content
fn extract_tag_attributes(opening_tag: &str) -> Option<String> {
    opening_tag.find(char::is_whitespace).map(|attr_start| {
        let attr_str = opening_tag[attr_start..].trim();
        if let Some(stripped) = attr_str.strip_suffix('>') {
            let trimmed = stripped.trim();
            // Ensure the attribute string ends with a quote if it starts with one
            if trimmed.starts_with('"') && !trimmed.ends_with('"') {
                format!("{trimmed}\"")
            } else {
                trimmed.to_string()
            }
        } else {
            attr_str.to_string()
        }
    })
}
