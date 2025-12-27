use std::collections::HashMap;

use parse_wiki_text_2::Configuration;

use crate::{
    TemplateContext, TemplateError, TemplateEvaluator, TemplateToInstantiate, async_trait,
};
use wikitext_simplified::WikitextSimplifiedNode;

/// In-memory template loader for testing.
struct MockContext {
    configuration: Configuration,
    templates: HashMap<String, String>,
    magic_variables: HashMap<String, String>,
}

impl MockContext {
    fn new() -> Self {
        Self {
            configuration: wikitext_simplified::wikitext_util::wikipedia_pwt_configuration(),
            templates: HashMap::new(),
            magic_variables: HashMap::new(),
        }
    }

    fn add_template(&mut self, name: &str, content: &str) {
        let key = name.to_lowercase().replace(" ", "_");
        self.templates.insert(key, content.to_string());
    }

    fn add_magic_variable(&mut self, name: &str, value: &str) {
        self.magic_variables
            .insert(name.to_lowercase(), value.to_string());
    }
}

#[async_trait]
impl TemplateContext for MockContext {
    fn configuration(&self) -> &Configuration {
        &self.configuration
    }

    fn resolve_magic_variable(&self, name: &str) -> Option<String> {
        self.magic_variables.get(&name.to_lowercase()).cloned()
    }

    async fn load_template(&self, name: &str) -> Result<String, TemplateError> {
        let key = name.to_lowercase().replace(" ", "_");
        self.templates
            .get(&key)
            .cloned()
            .ok_or_else(|| TemplateError::TemplateNotFound {
                name: name.to_string(),
                key,
            })
    }
}

fn block_on<F: std::future::Future>(f: F) -> F::Output {
    // Simple blocking executor for tests
    use std::task::{Context, Poll, Wake, Waker};
    struct NoopWaker;
    impl Wake for NoopWaker {
        fn wake(self: std::sync::Arc<Self>) {}
    }
    let waker = Waker::from(std::sync::Arc::new(NoopWaker));
    let mut cx = Context::from_waker(&waker);
    let mut f = std::pin::pin!(f);
    loop {
        match f.as_mut().poll(&mut cx) {
            Poll::Ready(v) => return v,
            Poll::Pending => {
                // For our sync tests, pending should never happen
                panic!("Future returned Pending in sync test context");
            }
        }
    }
}

#[test]
fn test_nested_table_template_instantiation() {
    let mut context = MockContext::new();

    // Create a simple cell attribute template
    context.add_template("lua/cellalign", r#"align="right""#);

    // Create a table template with nested template in cell attributes
    context.add_template(
        "lua/testtable",
        r#"{| class="wikitable"
!Returns
!Prototype
|-
|{{Lua/CellAlign}} | TypeA
|align="left" | FunctionA()
|-
|{{Lua/CellAlign}} | TypeB
|align="left" | FunctionB()
|}"#,
    );

    let mut evaluator = TemplateEvaluator::new(&context);

    // Instantiate the table template
    let result = block_on(evaluator.instantiate(TemplateToInstantiate::Name("Lua/TestTable"), &[]));

    // Verify the result is a table (possibly wrapped in a Fragment)
    let table_node = match &result {
        WikitextSimplifiedNode::Table { .. } => &result,
        WikitextSimplifiedNode::Fragment { children } => {
            &children
                .iter()
                .find(|node| matches!(node.value, WikitextSimplifiedNode::Table { .. }))
                .expect("Fragment should contain a Table node")
                .value
        }
        _ => panic!(
            "Expected Table or Fragment with Table node, got {:?}",
            result
        ),
    };

    match table_node {
        WikitextSimplifiedNode::Table { rows, .. } => {
            // Should have 3 rows (1 header + 2 data)
            assert_eq!(
                rows.len(),
                3,
                "Table should have 3 rows (1 header + 2 data)"
            );

            // Check first data row has 2 cells
            assert_eq!(rows[1].cells.len(), 2, "First data row should have 2 cells");

            // Verify the first cell has the correct attribute from the template
            if let Some(attrs) = &rows[1].cells[0].attributes {
                let attrs_node = WikitextSimplifiedNode::Fragment {
                    children: attrs.clone(),
                };
                let attrs_text = attrs_node.to_wikitext();
                assert!(
                    attrs_text.contains("right"),
                    "First cell should have 'align=right' attribute from template expansion"
                );
            }

            // Verify the first cell content is just "TypeA", not merged with second cell
            let cell_content = WikitextSimplifiedNode::Fragment {
                children: rows[1].cells[0].content.clone(),
            }
            .to_wikitext();
            assert!(
                !cell_content.contains("FunctionA"),
                "First cell should not contain content from second cell"
            );
            assert!(
                cell_content.contains("TypeA"),
                "First cell should contain TypeA"
            );

            // Verify the second cell exists and has correct content
            let cell2_content = WikitextSimplifiedNode::Fragment {
                children: rows[1].cells[1].content.clone(),
            }
            .to_wikitext();
            assert!(
                cell2_content.contains("FunctionA"),
                "Second cell should contain FunctionA"
            );
        }
        _ => panic!("Expected Table node, got {:?}", result),
    }
}

#[test]
fn test_non_table_template_uses_roundtrip() {
    let mut context = MockContext::new();

    // Create a template that expands to bold text
    context.add_template("boldtext", "'''important'''");

    let mut evaluator = TemplateEvaluator::new(&context);

    // Instantiate the template
    let result = block_on(evaluator.instantiate(TemplateToInstantiate::Name("BoldText"), &[]));

    // The result should be a Fragment containing a Bold node (due to roundtrip parsing)
    match result {
        WikitextSimplifiedNode::Fragment { children } => {
            assert!(
                children
                    .iter()
                    .any(|node| matches!(node.value, WikitextSimplifiedNode::Bold { .. })),
                "Template should be reparsed into Bold node through wikitext roundtrip"
            );
        }
        WikitextSimplifiedNode::Bold { .. } => {
            // Direct Bold node is also acceptable
        }
        _ => panic!("Expected Bold or Fragment with Bold node, got {:?}", result),
    }
}

#[test]
fn test_magic_variable_resolution() {
    let mut context = MockContext::new();
    context.add_magic_variable("subpagename", "TestPage");
    context.add_template("greet", "Hello, {{{subpagename}}}!");

    let mut evaluator = TemplateEvaluator::new(&context);

    let result = block_on(evaluator.instantiate(TemplateToInstantiate::Name("greet"), &[]));

    let text = result.to_wikitext();
    assert!(
        text.contains("TestPage"),
        "Magic variable should be resolved: {text}"
    );
}
