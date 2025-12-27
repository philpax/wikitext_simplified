//! Template evaluation for wikitext_simplified AST nodes.
//!
//! This crate provides template instantiation capabilities for wikitext templates,
//! supporting nested templates, magic variables, and table cell reparsing.

use std::{collections::HashMap, error::Error, fmt};

pub use async_trait::async_trait;

use parse_wiki_text_2::Configuration;
use wikitext_simplified::{Span, Spanned, TemplateParameter, WikitextSimplifiedNode};

#[cfg(test)]
mod tests;

/// Error type for template operations.
#[derive(Debug)]
pub enum TemplateError {
    /// Template was not found in the loader.
    TemplateNotFound { name: String, key: String },
    /// Failed to load template content from storage.
    LoadFailed {
        name: String,
        path: String,
        source: std::io::Error,
    },
    /// Failed to parse template wikitext.
    ParseFailed { name: String, message: String },
    /// Failed to scan template directory.
    DirectoryScanFailed {
        path: String,
        source: std::io::Error,
    },
    /// User-provided error from custom context implementations.
    User(Box<dyn Error + Send + Sync>),
}
impl fmt::Display for TemplateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TemplateNotFound { name, key } => {
                write!(f, "Template not found: {name} (key: {key})")
            }
            Self::LoadFailed { name, path, source } => {
                write!(f, "Failed to load template '{name}' from {path}: {source}")
            }
            Self::ParseFailed { name, message } => {
                write!(f, "Failed to parse template '{name}': {message}")
            }
            Self::DirectoryScanFailed { path, source } => {
                write!(f, "Failed to scan template directory {path}: {source}")
            }
            Self::User(e) => write!(f, "{e}"),
        }
    }
}
impl Error for TemplateError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::LoadFailed { source, .. } => Some(source),
            Self::DirectoryScanFailed { source, .. } => Some(source),
            Self::User(e) => Some(e.as_ref()),
            _ => None,
        }
    }
}

/// Trait for providing template context during instantiation.
///
/// Implementors provide both magic variable resolution (like `{{{SUBPAGENAME}}}`)
/// and template loading capabilities.
#[async_trait]
pub trait TemplateContext: Send + Sync {
    /// Get the parser configuration for parsing wikitext.
    fn configuration(&self) -> &Configuration;

    /// Resolve a magic variable by name (e.g., "subpagename").
    ///
    /// Returns `Some(value)` if the variable is known, `None` otherwise.
    fn resolve_magic_variable(&self, name: &str) -> Option<String>;

    /// Load a template by name, returning its wikitext content.
    ///
    /// This is async to support web-based template fetching.
    async fn load_template(&self, name: &str) -> Result<String, TemplateError>;
}

/// Specifies what to instantiate: either a template by name or an already-parsed node.
#[derive(Clone, Debug)]
pub enum TemplateToInstantiate<'a> {
    /// Load and instantiate a template by name.
    Name(&'a str),
    /// Instantiate an already-parsed node.
    Node(WikitextSimplifiedNode),
}

/// Template instantiation engine.
///
/// Caches parsed templates and handles recursive template expansion.
pub struct TemplateEvaluator<'a> {
    context: &'a dyn TemplateContext,
    templates: HashMap<String, WikitextSimplifiedNode>,
}
impl<'a> TemplateEvaluator<'a> {
    /// Create a new template engine with the given context.
    pub fn new(context: &'a dyn TemplateContext) -> Self {
        Self {
            context,
            templates: HashMap::new(),
        }
    }

    /// Reparse text content in table cells that contains wikitext markup.
    async fn reparse_table_cells(&mut self, node: &mut WikitextSimplifiedNode) {
        use WikitextSimplifiedNode as WSN;

        match node {
            WSN::Table { rows, .. } => {
                for row in rows {
                    for cell in &mut row.cells {
                        let cell_wikitext = WSN::Fragment {
                            children: cell.content.clone(),
                        }
                        .to_wikitext();

                        // Check if cell content contains wikitext markup or templates
                        let has_markup = cell_wikitext.contains("[[")
                            || cell_wikitext.contains("'''")
                            || cell_wikitext.contains("''")
                            || cell_wikitext.contains("{{");

                        if has_markup
                            && let Ok(parsed) = wikitext_simplified::parse_and_simplify_wikitext(
                                &cell_wikitext,
                                self.context.configuration(),
                            )
                            && !parsed.is_empty()
                        {
                            // Compute span from original cell content
                            let original_span = Span {
                                start: cell.content.first().map(|s| s.span.start).unwrap_or(0),
                                end: cell.content.last().map(|s| s.span.end).unwrap_or(0),
                            };

                            // After reparsing, we may have new templates to instantiate
                            let reparsed = WSN::Fragment { children: parsed };
                            let instantiated = Box::pin(
                                self.instantiate(TemplateToInstantiate::Node(reparsed), &[]),
                            )
                            .await;

                            // Extract children from the result
                            match instantiated {
                                WSN::Fragment { children } => {
                                    cell.content = children;
                                }
                                other => {
                                    cell.content = vec![Spanned {
                                        value: other,
                                        span: original_span,
                                    }];
                                }
                            }
                        }
                    }
                }
            }
            WSN::Fragment { children } => {
                for child in children {
                    // Recursive call needs boxing
                    let fut = self.reparse_table_cells(&mut child.value);
                    Box::pin(fut).await;
                }
            }
            _ => {}
        }
    }

    /// Get a cached template or load and parse it.
    async fn get(&mut self, name: &str) -> Result<WikitextSimplifiedNode, TemplateError> {
        let key = name.to_lowercase().replace(" ", "_");

        if !self.templates.contains_key(&key) {
            let content = self.context.load_template(name).await?;
            let simplified = wikitext_simplified::parse_and_simplify_wikitext(
                &content,
                self.context.configuration(),
            )
            .map_err(|e| TemplateError::ParseFailed {
                name: name.to_string(),
                message: format!("{e:?}"),
            })?;
            self.templates.insert(
                key.clone(),
                WikitextSimplifiedNode::Fragment {
                    children: simplified,
                },
            );
        }

        Ok(self.templates[&key].clone())
    }

    /// Replace templates and parameters in the AST once.
    async fn replace_once(
        &mut self,
        template: &mut WikitextSimplifiedNode,
        parameters: &[TemplateParameter],
    ) {
        use WikitextSimplifiedNode as WSN;

        // Collect template calls first, then process them
        let mut template_calls: Vec<(String, Vec<TemplateParameter>)> = Vec::new();

        // First pass: identify what needs to be replaced and replace parameters
        template.visit_and_replace_mut(&mut |node| match node {
            WSN::Template {
                name,
                parameters: template_params,
            } => {
                template_calls.push((name.clone(), template_params.clone()));
                // Placeholder - will be replaced
                WSN::Text {
                    text: format!("__TEMPLATE_PLACEHOLDER_{}__", template_calls.len() - 1),
                }
            }
            WSN::TemplateParameterUse { name, default } => {
                let parameter = parameters
                    .iter()
                    .find(|p| p.name == *name)
                    .map(|p| p.value.clone())
                    .or_else(|| self.context.resolve_magic_variable(name));
                if let Some(parameter) = parameter {
                    WSN::Text { text: parameter }
                } else if let Some(default) = default {
                    WSN::Text {
                        text: WSN::Fragment {
                            children: default.clone(),
                        }
                        .to_wikitext(),
                    }
                } else {
                    WSN::Text {
                        text: String::new(),
                    }
                }
            }
            _ => node.clone(),
        });

        // Second pass: instantiate templates and collect results
        let mut results = Vec::new();
        for (name, params) in template_calls {
            let result =
                Box::pin(self.instantiate(TemplateToInstantiate::Name(&name), &params)).await;
            // Flatten single-child fragments
            let result = match result {
                WSN::Fragment { children } if children.len() == 1 => {
                    children.into_iter().next().unwrap().value
                }
                _ => result,
            };
            results.push(result);
        }

        // Third pass: replace placeholders with actual results
        for (idx, result) in results.into_iter().enumerate() {
            let placeholder = format!("__TEMPLATE_PLACEHOLDER_{idx}__");
            template.visit_and_replace_mut(&mut |node| {
                if let WSN::Text { text } = node
                    && text == &placeholder
                {
                    return result.clone();
                }
                node.clone()
            });
        }
    }

    /// Instantiate a template by replacing all template parameter uses with their values,
    /// instantiating nested templates, converting back to wikitext, and repeating until
    /// no more template parameter uses or nested templates are found.
    pub async fn instantiate(
        &mut self,
        template: TemplateToInstantiate<'_>,
        parameters: &[TemplateParameter],
    ) -> WikitextSimplifiedNode {
        use WikitextSimplifiedNode as WSN;

        let mut template = match template {
            TemplateToInstantiate::Name(name) => {
                // Check for magic template names
                if let Some(value) = self.context.resolve_magic_variable(name) {
                    return WSN::Text { text: value };
                }
                match self.get(name).await {
                    Ok(t) => t,
                    Err(e) => {
                        // Return error as text for now - could be improved
                        return WSN::Text {
                            text: format!("{{{{Template error: {e}}}}}"),
                        };
                    }
                }
            }
            TemplateToInstantiate::Node(node) => node,
        };

        // Check if we're done
        let mut further_instantiation_required = false;
        template.visit(&mut |node| {
            further_instantiation_required |= matches!(
                node,
                WSN::TemplateParameterUse { .. } | WSN::Template { .. }
            );
        });
        if !further_instantiation_required {
            return template;
        }

        // Do one round of replacement first
        self.replace_once(&mut template, parameters).await;

        // Check if we have tables - this catches tables created by template expansion
        let contains_table = {
            let mut found = false;
            template.visit(&mut |node| {
                if matches!(node, WSN::Table { .. }) {
                    found = true;
                }
            });
            found
        };

        if contains_table {
            // For templates containing tables, recursively replace until no more changes
            loop {
                let before = template.to_wikitext();
                self.replace_once(&mut template, parameters).await;
                let after = template.to_wikitext();

                if before == after {
                    break;
                }
            }

            // After template expansion, reparse text content in table cells
            self.reparse_table_cells(&mut template).await;

            template
        } else {
            // For non-table templates, roundtrip through wikitext
            let template_wikitext = template.to_wikitext();
            let roundtripped_template = wikitext_simplified::parse_and_simplify_wikitext(
                &template_wikitext,
                self.context.configuration(),
            )
            .unwrap_or_else(|e| {
                panic!("Failed to parse and simplify template {template_wikitext}: {e:?}")
            });

            Box::pin(self.instantiate(
                TemplateToInstantiate::Node(WikitextSimplifiedNode::Fragment {
                    children: roundtripped_template,
                }),
                parameters,
            ))
            .await
        }
    }
}
