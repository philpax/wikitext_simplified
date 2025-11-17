//! A library for simplifying wikitext into a more manageable AST structure.
//!
//! This library provides functionality to parse wikitext and convert it into a simplified
//! abstract syntax tree (AST) that's easier to work with. It handles various wikitext
//! elements like templates, links, formatting, and more.

#![deny(missing_docs)]

pub use parse_wiki_text_2;
pub use wikitext_util;

use parse_wiki_text_2 as pwt;

mod simplification;
pub use simplification::{
    simplify_wikitext_node, simplify_wikitext_nodes, DefinitionListItemType, NodeStructureError,
    SimplificationError, SimplificationErrorContext, Span, Spanned, TemplateParameter,
    WikitextSimplifiedDefinitionListItem, WikitextSimplifiedNode, WikitextSimplifiedTableCaption,
    WikitextSimplifiedTableCell, WikitextSimplifiedTableRow,
};

#[cfg(test)]
mod tests;

/// Errors that can occur during parsing of wikitext
#[derive(Debug)]
pub enum ParseAndSimplifyWikitextError<'a> {
    /// Error occurred during parsing of wikitext
    ParseError(pwt::ParseError<'a>),
    /// Error occurred during simplification of wikitext nodes
    SimplificationError(SimplificationError),
}
impl std::fmt::Display for ParseAndSimplifyWikitextError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseAndSimplifyWikitextError::ParseError(e) => write!(f, "Parse error: {e:?}"),
            ParseAndSimplifyWikitextError::SimplificationError(e) => {
                write!(f, "Simplification error: {e:?}")
            }
        }
    }
}
impl std::error::Error for ParseAndSimplifyWikitextError<'_> {}

/// Helper function that parses wikitext and converts it into a simplified AST structure.
///
/// # Errors
///
/// This function will return an error if the wikitext cannot be parsed or simplified.
pub fn parse_and_simplify_wikitext<'a>(
    wikitext: &'a str,
    pwt_configuration: &pwt::Configuration,
) -> Result<Vec<Spanned<WikitextSimplifiedNode>>, ParseAndSimplifyWikitextError<'a>> {
    let output = pwt_configuration
        .parse(wikitext)
        .map_err(ParseAndSimplifyWikitextError::ParseError)?;

    simplify_wikitext_nodes(wikitext, &output.nodes)
        .map_err(ParseAndSimplifyWikitextError::SimplificationError)
}
