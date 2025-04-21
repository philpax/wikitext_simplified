use super::*;

use std::sync::LazyLock;

use wikitext_util::wikipedia_pwt_configuration;
use WikitextSimplifiedNode as WSN;

static PWT_CONFIGURATION: LazyLock<pwt::Configuration> = LazyLock::new(wikipedia_pwt_configuration);

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

#[test]
fn will_simplify_nested_template_parameters() {
    let wikitext = r#"{{{description|{{{file_name}}}}}}"#;
    let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
    assert_eq!(
        simplified,
        vec![WSN::TemplateParameterUse {
            name: "description".into(),
            default: Some(Box::new(WSN::TemplateParameterUse {
                name: "file_name".into(),
                default: None,
            })),
        }]
    );
}

#[test]
fn will_simplify_template_parameter_inside_html_tag() {
    let wikitext = r#"<span style="color:#505050;font-size:80%">{{{1}}}</span>"#;
    let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
    assert_eq!(
        simplified,
        vec![WSN::Tag {
            name: "span".into(),
            children: vec![WSN::TemplateParameterUse {
                name: "1".into(),
                default: None,
            }],
        }]
    );
}

#[test]
fn can_parse_heading() {
    let wikitext = r#"==Heading=="#;
    let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
    assert_eq!(
        simplified,
        vec![WSN::Heading {
            level: 2,
            children: vec![WSN::Text {
                text: "Heading".into(),
            }],
        }]
    );
}

#[test]
fn test_basic_text() {
    let wikitext = "Hello, world!";
    let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
    assert_eq!(
        simplified,
        vec![WSN::Text {
            text: "Hello, world!".into()
        }]
    );
}

#[test]
fn test_bold_text() {
    let wikitext = "'''bold text'''";
    let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
    assert_eq!(
        simplified,
        vec![WSN::Bold {
            children: vec![WSN::Text {
                text: "bold text".into()
            }]
        }]
    );
}

#[test]
fn test_italic_text() {
    let wikitext = "''italic text''";
    let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
    assert_eq!(
        simplified,
        vec![WSN::Italic {
            children: vec![WSN::Text {
                text: "italic text".into()
            }]
        }]
    );
}

#[test]
fn test_bold_italic_text() {
    let wikitext = "'''''bold italic text'''''";
    let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
    assert_eq!(
        simplified,
        vec![WSN::Bold {
            children: vec![WSN::Italic {
                children: vec![WSN::Text {
                    text: "bold italic text".into()
                }]
            }]
        }]
    );
}

#[test]
fn test_mixed_formatting() {
    let wikitext = "This is '''bold''', this is ''italic'', and this is '''''bold italic'''''";
    let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
    assert_eq!(
        simplified,
        vec![
            WSN::Text {
                text: "This is ".into()
            },
            WSN::Bold {
                children: vec![WSN::Text {
                    text: "bold".into()
                }]
            },
            WSN::Text {
                text: ", this is ".into()
            },
            WSN::Italic {
                children: vec![WSN::Text {
                    text: "italic".into()
                }]
            },
            WSN::Text {
                text: ", and this is ".into()
            },
            WSN::Bold {
                children: vec![WSN::Italic {
                    children: vec![WSN::Text {
                        text: "bold italic".into()
                    }]
                }]
            }
        ]
    );
}

#[test]
fn test_internal_link() {
    let wikitext = "[[Main Page]]";
    let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
    assert_eq!(
        simplified,
        vec![WSN::Link {
            text: "Main Page".into(),
            title: "Main Page".into()
        }]
    );
}

#[test]
fn test_internal_link_with_text() {
    let wikitext = "[[Main Page|Home]]";
    let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
    assert_eq!(
        simplified,
        vec![WSN::Link {
            text: "Home".into(),
            title: "Main Page".into()
        }]
    );
}

#[test]
fn test_external_link() {
    let wikitext = "[https://example.com]";
    let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
    assert_eq!(
        simplified,
        vec![WSN::ExtLink {
            link: "https://example.com".into(),
            text: None
        }]
    );
}

#[test]
fn test_external_link_with_text() {
    let wikitext = "[https://example.com Example]";
    let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
    assert_eq!(
        simplified,
        vec![WSN::ExtLink {
            link: "https://example.com".into(),
            text: Some("Example".into())
        }]
    );
}

#[test]
fn test_simple_template() {
    let wikitext = "{{Template}}";
    let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
    assert_eq!(
        simplified,
        vec![WSN::Template {
            name: "Template".into(),
            children: vec![]
        }]
    );
}

#[test]
fn test_template_with_parameters() {
    let wikitext = "{{Template|param1=value1|param2=value2}}";
    let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
    assert_eq!(
        simplified,
        vec![WSN::Template {
            name: "Template".into(),
            children: vec![
                TemplateParameter {
                    name: "param1".into(),
                    value: "value1".into()
                },
                TemplateParameter {
                    name: "param2".into(),
                    value: "value2".into()
                }
            ]
        }]
    );
}

#[test]
fn test_template_with_unnamed_parameters() {
    let wikitext = "{{Template|value1|value2}}";
    let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
    assert_eq!(
        simplified,
        vec![WSN::Template {
            name: "Template".into(),
            children: vec![
                TemplateParameter {
                    name: "1".into(),
                    value: "value1".into()
                },
                TemplateParameter {
                    name: "2".into(),
                    value: "value2".into()
                }
            ]
        }]
    );
}

#[test]
fn test_html_tag() {
    let wikitext = "<span>Hello</span>";
    let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
    assert_eq!(
        simplified,
        vec![WSN::Tag {
            name: "span".into(),
            children: vec![WSN::Text {
                text: "Hello".into()
            }]
        }]
    );
}

#[test]
fn test_html_tag_with_attributes() {
    let wikitext = r#"<span style="color:red">Red text</span>"#;
    let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
    assert_eq!(
        simplified,
        vec![WSN::Tag {
            name: "span".into(),
            children: vec![WSN::Text {
                text: "Red text".into()
            }]
        }]
    );
}

#[test]
fn test_blockquote() {
    let wikitext = "<blockquote>Quoted text</blockquote>";
    let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
    assert_eq!(
        simplified,
        vec![WSN::Blockquote {
            children: vec![WSN::Text {
                text: "Quoted text".into()
            }]
        }]
    );
}

#[test]
fn test_superscript() {
    let wikitext = "<sup>superscript</sup>";
    let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
    assert_eq!(
        simplified,
        vec![WSN::Superscript {
            children: vec![WSN::Text {
                text: "superscript".into()
            }]
        }]
    );
}

#[test]
fn test_subscript() {
    let wikitext = "<sub>subscript</sub>";
    let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
    assert_eq!(
        simplified,
        vec![WSN::Subscript {
            children: vec![WSN::Text {
                text: "subscript".into()
            }]
        }]
    );
}

#[test]
fn test_small_text() {
    let wikitext = "<small>small text</small>";
    let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
    assert_eq!(
        simplified,
        vec![WSN::Small {
            children: vec![WSN::Text {
                text: "small text".into()
            }]
        }]
    );
}

#[test]
fn test_preformatted() {
    let wikitext = "<pre>preformatted text</pre>";
    let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
    assert_eq!(
        simplified,
        vec![WSN::Preformatted {
            children: vec![WSN::Text {
                text: "preformatted text".into()
            }]
        }]
    );
}

#[test]
fn test_paragraph_breaks() {
    let wikitext = "Paragraph 1\n\nParagraph 2";
    let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
    assert_eq!(
        simplified,
        vec![
            WSN::Text {
                text: "Paragraph 1".into()
            },
            WSN::ParagraphBreak,
            WSN::Text {
                text: "Paragraph 2".into()
            }
        ]
    );
}

#[test]
fn test_nested_formatting() {
    let wikitext = "'''bold with ''italic'' inside'''";
    let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
    assert_eq!(
        simplified,
        vec![WSN::Bold {
            children: vec![
                WSN::Text {
                    text: "bold with ".into()
                },
                WSN::Italic {
                    children: vec![WSN::Text {
                        text: "italic".into()
                    }]
                },
                WSN::Text {
                    text: " inside".into()
                }
            ]
        }]
    );
}

#[test]
fn test_template_in_link() {
    let wikitext = "[[Page|{{Template}}]]";
    let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
    assert_eq!(
        simplified,
        vec![WSN::Link {
            text: "{{Template}}".into(),
            title: "Page".into()
        }]
    );
}

#[test]
fn test_formatting_in_template() {
    let wikitext = "{{Template|param='''bold'''}}";
    let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
    assert_eq!(
        simplified,
        vec![WSN::Template {
            name: "Template".into(),
            children: vec![TemplateParameter {
                name: "param".into(),
                value: "'''bold'''".into()
            }]
        }]
    );
}

#[test]
fn test_mismatched_tags() {
    let wikitext = "<span>text</div>";
    let result = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION);
    assert!(result.is_err());
    if let Err(ParseAndSimplifyWikitextError::SimplificationError(
        SimplificationError::InvalidNodeStructure { kind, .. },
    )) = result
    {
        assert!(matches!(
            kind,
            NodeStructureError::TagClosureMismatch { .. }
        ));
    } else {
        panic!("Expected TagClosureMismatch error");
    }
}

#[test]
fn test_table_conversion() {
    let wikitext = r#"{| class="wikitable"
|+ Caption
|-
! Header 1 !! Header 2
|-
| Cell 1 || Cell 2
|-
| Cell 3 || Cell 4
|}"#;
    let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
    assert_eq!(
        simplified,
        vec![WSN::Table {
            attributes: "class=\"wikitable\"".into(),
            captions: vec![WikitextSimplifiedTableCaption {
                attributes: None,
                content: vec![WSN::Text {
                    text: "Caption".into()
                }]
            }],
            rows: vec![
                WikitextSimplifiedTableRow {
                    attributes: None,
                    cells: vec![
                        WikitextSimplifiedTableCell {
                            attributes: None,
                            content: vec![WSN::Text {
                                text: "Header 1".into()
                            }]
                        },
                        WikitextSimplifiedTableCell {
                            attributes: None,
                            content: vec![WSN::Text {
                                text: "Header 2".into()
                            }]
                        }
                    ]
                },
                WikitextSimplifiedTableRow {
                    attributes: None,
                    cells: vec![
                        WikitextSimplifiedTableCell {
                            attributes: None,
                            content: vec![WSN::Text {
                                text: "Cell 1".into()
                            }]
                        },
                        WikitextSimplifiedTableCell {
                            attributes: None,
                            content: vec![WSN::Text {
                                text: "Cell 2".into()
                            }]
                        }
                    ]
                },
                WikitextSimplifiedTableRow {
                    attributes: None,
                    cells: vec![
                        WikitextSimplifiedTableCell {
                            attributes: None,
                            content: vec![WSN::Text {
                                text: "Cell 3".into()
                            }]
                        },
                        WikitextSimplifiedTableCell {
                            attributes: None,
                            content: vec![WSN::Text {
                                text: "Cell 4".into()
                            }]
                        }
                    ]
                }
            ]
        }]
    );
}
