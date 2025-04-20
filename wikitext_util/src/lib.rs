//! A utility library for working with wikitext, providing functions to parse and process Wikipedia-style markup.
//!
//! This library provides utilities for working with the `parse-wiki-text-2` crate, including functions
//! to extract text content from wikitext nodes and handle various wikitext elements.

#![deny(missing_docs)]
use parse_wiki_text_2 as pwt;

/// Metadata about a wikitext node, including its type, position in the source text, and child nodes.
pub struct NodeMetadata<'a> {
    /// The name/type of the node (e.g. "bold", "link", "template")
    pub name: &'static str,
    /// The starting position of the node in the source text
    pub start: usize,
    /// The ending position of the node in the source text
    pub end: usize,
    /// Optional child nodes contained within this node
    pub children: Option<&'a [pwt::Node<'a>]>,
}
impl<'a> NodeMetadata<'a> {
    fn new(
        name: &'static str,
        start: usize,
        end: usize,
        children: Option<&'a [pwt::Node<'a>]>,
    ) -> Self {
        Self {
            name,
            start,
            end,
            children,
        }
    }

    /// Creates a [`NodeMetadata`] instance from a wikitext node.
    ///
    /// This function extracts metadata about a node's type, position, and children
    /// from a [`parse_wiki_text_2`] node.
    pub fn for_node(node: &'a pwt::Node) -> NodeMetadata<'a> {
        use NodeMetadata as NM;
        match node {
            pwt::Node::Bold { end, start } => NM::new("bold", *start, *end, None),
            pwt::Node::BoldItalic { end, start } => NM::new("bold_italic", *start, *end, None),
            pwt::Node::Category { end, start, .. } => NM::new("category", *start, *end, None),
            pwt::Node::CharacterEntity { end, start, .. } => {
                NM::new("character_entity", *start, *end, None)
            }
            pwt::Node::Comment { end, start } => NM::new("comment", *start, *end, None),
            pwt::Node::DefinitionList {
                end,
                start,
                items: _,
            } => NM::new("definition_list", *start, *end, None),
            pwt::Node::EndTag { end, start, .. } => NM::new("end_tag", *start, *end, None),
            pwt::Node::ExternalLink { end, nodes, start } => {
                NM::new("external_link", *start, *end, Some(nodes))
            }
            pwt::Node::Heading {
                end, start, nodes, ..
            } => NM::new("heading", *start, *end, Some(nodes)),
            pwt::Node::HorizontalDivider { end, start } => {
                NM::new("horizontal_divider", *start, *end, None)
            }
            pwt::Node::Image {
                end, start, text, ..
            } => NM::new("image", *start, *end, Some(text)),
            pwt::Node::Italic { end, start } => NM::new("italic", *start, *end, None),
            pwt::Node::Link {
                end, start, text, ..
            } => NM::new("link", *start, *end, Some(text)),
            pwt::Node::MagicWord { end, start } => NM::new("magic_word", *start, *end, None),
            pwt::Node::OrderedList {
                end,
                start,
                items: _,
            } => NM::new("ordered_list", *start, *end, None),
            pwt::Node::ParagraphBreak { end, start } => {
                NM::new("paragraph_break", *start, *end, None)
            }
            pwt::Node::Parameter { end, start, .. } => NM::new("parameter", *start, *end, None),
            pwt::Node::Preformatted { end, start, nodes } => {
                NM::new("preformatted", *start, *end, Some(nodes))
            }
            pwt::Node::Redirect { end, start, .. } => NM::new("redirect", *start, *end, None),
            pwt::Node::StartTag { end, start, .. } => NM::new("start_tag", *start, *end, None),
            pwt::Node::Table {
                end,
                start,
                rows: _,
                ..
            } => NM::new("table", *start, *end, None),
            pwt::Node::Tag {
                end, start, nodes, ..
            } => NM::new("tag", *start, *end, Some(nodes.as_slice())),
            pwt::Node::Template { end, start, .. } => NM::new("template", *start, *end, None),
            pwt::Node::Text { end, start, .. } => NM::new("text", *start, *end, None),
            pwt::Node::UnorderedList {
                end,
                start,
                items: _,
            } => NM::new("unordered_list", *start, *end, None),
        }
    }
}

/// Configuration options for extracting inner text from wikitext nodes.
#[derive(Default)]
pub struct InnerTextConfig {
    /// Whether to stop processing after encountering a `<br>` tag.
    pub stop_after_br: bool,
}

/// Extracts the raw wikitext content from a sequence of nodes.
///
/// Unlike [`nodes_inner_text`], this retrieves the raw wikitext for each node,
/// preserving the original formatting.
pub fn nodes_wikitext(original_wikitext: &str, nodes: &[pwt::Node]) -> String {
    let mut result = String::new();
    for node in nodes {
        let metadata = NodeMetadata::for_node(node);
        result.push_str(&original_wikitext[metadata.start..metadata.end]);
    }
    result
}

/// Extracts the text content from a sequence of wikitext nodes.
///
/// This function joins the text content of nodes together without spaces and trims the result.
/// Note that this behavior may not always be correct for all use cases.
pub fn nodes_inner_text(nodes: &[pwt::Node], config: &InnerTextConfig) -> String {
    let mut result = String::new();
    for node in nodes {
        if config.stop_after_br && matches!(node, pwt::Node::StartTag { name, .. } if name == "br")
        {
            break;
        }
        result.push_str(&node_inner_text(node, config));
    }
    result.trim().to_string()
}

/// Extracts the text content from a single wikitext node.
///
/// This function handles various node types and extracts their text content,
/// ignoring formatting. Note that this behavior may not always be correct for all use cases.
///
/// This function is allocation-heavy; there's room for optimization but it's not currently a priority.
pub fn node_inner_text(node: &pwt::Node, config: &InnerTextConfig) -> String {
    use pwt::Node;
    match node {
        Node::CharacterEntity { character, .. } => character.to_string(),
        // Node::DefinitionList { end, items, start } => nodes_inner_text(items, config),
        Node::Heading { nodes, .. } => nodes_inner_text(nodes, config),
        Node::Image { text, .. } => nodes_inner_text(text, config),
        Node::Link { text, .. } => nodes_inner_text(text, config),
        // Node::OrderedList { end, items, start } => nodes_inner_text(items, config),
        Node::Preformatted { nodes, .. } => nodes_inner_text(nodes, config),
        Node::Text { value, .. } => value.to_string(),
        // Node::UnorderedList { end, items, start } => nodes_inner_text(items, config),
        Node::Template {
            name, parameters, ..
        } => {
            let name = nodes_inner_text(name, config).to_ascii_lowercase();

            if name == "lang" {
                // hack: extract the text from the other-language template
                // the parameter is `|text=`, or the second paramter, so scan for both
                parameters
                    .iter()
                    .find(|p| {
                        p.name
                            .as_ref()
                            .is_some_and(|n| nodes_inner_text(n, config) == "text")
                    })
                    .or_else(|| parameters.iter().filter(|p| p.name.is_none()).nth(1))
                    .map(|p| nodes_inner_text(&p.value, config))
                    .unwrap_or_default()
            } else if name == "transliteration" || name == "tlit" || name == "transl" {
                // text is either the second or the third positional argument;
                // in the case of the latter, the second argument is the transliteration scheme,
                // so we want to select for the third first before the second

                let positional_args = parameters
                    .iter()
                    .filter(|p| p.name.is_none())
                    .collect::<Vec<_>>();
                if positional_args.len() >= 3 {
                    nodes_inner_text(&positional_args[2].value, config)
                } else {
                    nodes_inner_text(&positional_args[1].value, config)
                }
            } else {
                "".to_string()
            }
        }
        _ => "".to_string(),
    }
}

/// Creates a Wikipedia-compatible configuration for the `parse_wiki_text_2` parser.
///
/// This configuration includes Wikipedia-specific settings for:
/// - Category namespaces
/// - Extension tags
/// - File namespaces
/// - Magic words
/// - Protocols
/// - Redirect magic words
pub fn wikipedia_pwt_configuration() -> pwt::Configuration {
    pwt::Configuration::new(&pwt::ConfigurationSource {
        category_namespaces: &["category"],
        extension_tags: &[
            "categorytree",
            "ce",
            "charinsert",
            "chem",
            "gallery",
            "graph",
            "hiero",
            "imagemap",
            "indicator",
            "inputbox",
            "langconvert",
            "mapframe",
            "maplink",
            "math",
            "nowiki",
            "poem",
            "pre",
            "ref",
            "references",
            "score",
            "section",
            "source",
            "syntaxhighlight",
            "templatedata",
            "templatestyles",
            "timeline",
        ],
        file_namespaces: &["file", "image"],
        // Wikipedia sets this to "abcdefghijklmnopqrstuvwxyz", but it complicates link parsing
        // (by causing successor content to be double-parsed), and it's not relevant to what we're
        // doing, so I'm content with just disabling it.
        link_trail: "",
        magic_words: &[
            "disambig",
            "expected_unconnected_page",
            "expectunusedcategory",
            "forcetoc",
            "hiddencat",
            "index",
            "newsectionlink",
            "nocc",
            "nocontentconvert",
            "noeditsection",
            "nogallery",
            "noglobal",
            "noindex",
            "nonewsectionlink",
            "notc",
            "notitleconvert",
            "notoc",
            "staticredirect",
            "toc",
        ],
        protocols: &[
            "//",
            "bitcoin:",
            "ftp://",
            "ftps://",
            "geo:",
            "git://",
            "gopher://",
            "http://",
            "https://",
            "irc://",
            "ircs://",
            "magnet:",
            "mailto:",
            "mms://",
            "news:",
            "nntp://",
            "redis://",
            "sftp://",
            "sip:",
            "sips:",
            "sms:",
            "ssh://",
            "svn://",
            "tel:",
            "telnet://",
            "urn:",
            "worldwind://",
            "xmpp:",
        ],
        redirect_magic_words: &["redirect"],
    })
}
