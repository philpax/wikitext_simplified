use crate::simplification::{
    DefinitionListItemType, Spanned, TemplateParameter, WikitextSimplifiedDefinitionListItem,
    WikitextSimplifiedListItem, WikitextSimplifiedNode as WSN,
};

use super::*;

use std::sync::LazyLock;

use wikitext_util::wikipedia_pwt_configuration;

static PWT_CONFIGURATION: LazyLock<pwt::Configuration> = LazyLock::new(wikipedia_pwt_configuration);

// Helper function to create a Spanned node with specific span
fn sp(value: WSN, start: usize, end: usize) -> Spanned<WSN> {
    Spanned {
        value,
        span: Span { start, end },
    }
}

// Helper function to create a Spanned node with a dummy span (for tests that don't care about spans)
#[allow(dead_code)]
fn spanned(value: WSN) -> Spanned<WSN> {
    Spanned {
        value,
        span: Span { start: 0, end: 0 },
    }
}

// Helper macro to wrap all nodes in a vec with dummy Spanned (for tests that don't care about spans)
macro_rules! spanned_vec {
    [$($node:expr),* $(,)?] => {
        vec![$(spanned($node)),*]
    };
}

#[test]
fn test_s_after_link() {
    let wikitext = "cool [[thing]]s by cool [[Person|person]]s";
    let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
    assert_eq!(
        simplified,
        vec![
            sp(WSN::Text {
                text: "cool ".into()
            }, 0, 5),
            sp(WSN::Link {
                text: "things".into(),
                title: "thing".into()
            }, 5, 15),
            sp(WSN::Text {
                text: " by cool ".into()
            }, 15, 24),
            sp(WSN::Link {
                text: "persons".into(),
                title: "Person".into()
            }, 24, 42),
        ]
    )
}

#[test]
fn can_parse_wikitext_in_link() {
    let wikitext = r#"[[Time signature|{{music|time|4|4}}]]"#;
    let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
    assert_eq!(
        simplified,
        spanned_vec![WSN::Link {
            text: "{{music|time|4|4}}".into(),
            title: "Time signature".into()
        }]
    )
}

#[test]
fn will_gracefully_ignore_refs() {
    let wikitext = r#"<ref name=bigtakeover>{{cite web|author=Kristen Sollee|title=Japanese Rock on NPR|work=[[The Big Takeover]]|date=2006-06-25|url=http://www.bigtakeover.com/news/japanese-rock-on-npr|access-date=2013-06-07|quote=It's a style of dress, there's a lot of costuming and make up and it's uniquely Japanese because it goes back to ancient Japan. Men would often wear women's clothing...}}</ref>"#;
    let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
    assert_eq!(simplified, spanned_vec![]);
}

#[test]
fn will_simplify_nested_template_parameters() {
    let wikitext = r#"{{{description|{{{file_name}}}}}}"#;
    let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
    assert_eq!(
        simplified,
        spanned_vec![WSN::TemplateParameterUse {
            name: "description".into(),
            default: Some(spanned_vec![WSN::TemplateParameterUse {
                name: "file_name".into(),
                default: None,
            }]),
        }]
    );
}

#[test]
fn will_simplify_template_parameter_inside_html_tag() {
    let wikitext = r#"<span style="color:#505050;font-size:80%">{{{1}}}</span>"#;
    let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
    assert_eq!(
        simplified,
        spanned_vec![WSN::Tag {
            name: "span".into(),
            attributes: Some(r#"style="color:#505050;font-size:80%""#.into()),
            children: spanned_vec![WSN::TemplateParameterUse {
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
        vec![sp(WSN::Heading {
            level: 2,
            children: vec![sp(WSN::Text {
                text: "Heading".into(),
            }, 2, 9)],
        }, 0, 11)]
    );
}

#[test]
fn test_basic_text() {
    let wikitext = "Hello, world!";
    let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
    assert_eq!(
        simplified,
        vec![sp(WSN::Text {
            text: "Hello, world!".into()
        }, 0, 13)]
    );
}

#[test]
fn test_bold_text() {
    let wikitext = "'''bold text'''";
    let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
    assert_eq!(
        simplified,
        vec![sp(WSN::Bold {
            children: vec![sp(WSN::Text {
                text: "bold text".into()
            }, 3, 12)]
        }, 0, 15)]
    );
}

#[test]
fn test_italic_text() {
    let wikitext = "''italic text''";
    let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
    assert_eq!(
        simplified,
        vec![sp(WSN::Italic {
            children: vec![sp(WSN::Text {
                text: "italic text".into()
            }, 2, 13)]
        }, 0, 15)]
    );
}

#[test]
fn test_bold_italic_text() {
    let wikitext = "'''''bold italic text'''''";
    let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
    assert_eq!(
        simplified,
        vec![sp(WSN::Bold {
            children: vec![sp(WSN::Italic {
                children: vec![sp(WSN::Text {
                    text: "bold italic text".into()
                }, 5, 21)]
            }, 0, 26)]
        }, 0, 26)]
    );
}

#[test]
fn test_mixed_formatting() {
    let wikitext = "This is '''bold''', this is ''italic'', and this is '''''bold italic'''''";
    let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
    assert_eq!(
        simplified,
        spanned_vec![
            WSN::Text {
                text: "This is ".into()
            },
            WSN::Bold {
                children: spanned_vec![WSN::Text {
                    text: "bold".into()
                }]
            },
            WSN::Text {
                text: ", this is ".into()
            },
            WSN::Italic {
                children: spanned_vec![WSN::Text {
                    text: "italic".into()
                }]
            },
            WSN::Text {
                text: ", and this is ".into()
            },
            WSN::Bold {
                children: spanned_vec![WSN::Italic {
                    children: spanned_vec![WSN::Text {
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
        vec![sp(WSN::Link {
            text: "Main Page".into(),
            title: "Main Page".into()
        }, 0, 13)]
    );
}

#[test]
fn test_internal_link_with_text() {
    let wikitext = "[[Main Page|Home]]";
    let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
    assert_eq!(
        simplified,
        vec![sp(WSN::Link {
            text: "Home".into(),
            title: "Main Page".into()
        }, 0, 18)]
    );
}

#[test]
fn test_external_link() {
    let wikitext = "[https://example.com]";
    let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
    assert_eq!(
        simplified,
        vec![sp(WSN::ExtLink {
            link: "https://example.com".into(),
            text: None
        }, 0, 21)]
    );
}

#[test]
fn test_external_link_with_text() {
    let wikitext = "[https://example.com Example]";
    let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
    assert_eq!(
        simplified,
        vec![sp(WSN::ExtLink {
            link: "https://example.com".into(),
            text: Some("Example".into())
        }, 0, 29)]
    );
}

#[test]
fn test_simple_template() {
    let wikitext = "{{Template}}";
    let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
    assert_eq!(
        simplified,
        vec![sp(WSN::Template {
            name: "Template".into(),
            parameters: vec![]
        }, 0, 12)]
    );
}

#[test]
fn test_template_with_parameters() {
    let wikitext = "{{Template|param1=value1|param2=value2}}";
    let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
    assert_eq!(
        simplified,
        vec![sp(WSN::Template {
            name: "Template".into(),
            parameters: vec![
                TemplateParameter {
                    name: "param1".into(),
                    value: "value1".into()
                },
                TemplateParameter {
                    name: "param2".into(),
                    value: "value2".into()
                }
            ]
        }, 0, 40)]
    );
}

#[test]
fn test_template_with_unnamed_parameters() {
    let wikitext = "{{Template|value1|value2}}";
    let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
    assert_eq!(
        simplified,
        vec![sp(WSN::Template {
            name: "Template".into(),
            parameters: vec![
                TemplateParameter {
                    name: "1".into(),
                    value: "value1".into()
                },
                TemplateParameter {
                    name: "2".into(),
                    value: "value2".into()
                }
            ]
        }, 0, 26)]
    );
}

#[test]
fn test_html_tag() {
    let wikitext = "<span>Hello</span>";
    let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
    assert_eq!(
        simplified,
        spanned_vec![WSN::Tag {
            name: "span".into(),
            attributes: None,
            children: spanned_vec![WSN::Text {
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
        spanned_vec![WSN::Tag {
            name: "span".into(),
            attributes: Some("style=\"color:red\"".into()),
            children: spanned_vec![WSN::Text {
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
        spanned_vec![WSN::Blockquote {
            children: spanned_vec![WSN::Text {
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
        spanned_vec![WSN::Superscript {
            children: spanned_vec![WSN::Text {
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
        spanned_vec![WSN::Subscript {
            children: spanned_vec![WSN::Text {
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
        spanned_vec![WSN::Small {
            children: spanned_vec![WSN::Text {
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
        spanned_vec![WSN::Preformatted {
            children: spanned_vec![WSN::Text {
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
            sp(WSN::Text {
                text: "Paragraph 1".into()
            }, 0, 11),
            sp(WSN::ParagraphBreak, 11, 13),
            sp(WSN::Text {
                text: "Paragraph 2".into()
            }, 13, 24)
        ]
    );
}

#[test]
fn test_nested_formatting() {
    let wikitext = "'''bold with ''italic'' inside'''";
    let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
    assert_eq!(
        simplified,
        spanned_vec![WSN::Bold {
            children: spanned_vec![
                WSN::Text {
                    text: "bold with ".into()
                },
                WSN::Italic {
                    children: spanned_vec![WSN::Text {
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
        spanned_vec![WSN::Link {
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
        spanned_vec![WSN::Template {
            name: "Template".into(),
            parameters: vec![TemplateParameter {
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
        spanned_vec![WSN::Table {
            attributes: spanned_vec![WSN::Text {
                text: "class=\"wikitable\"".into()
            }],
            captions: vec![WikitextSimplifiedTableCaption {
                attributes: None,
                content: spanned_vec![WSN::Text {
                    text: "Caption".into()
                }]
            }],
            rows: vec![
                WikitextSimplifiedTableRow {
                    attributes: vec![],
                    cells: vec![
                        WikitextSimplifiedTableCell {
                            attributes: None,
                            content: spanned_vec![WSN::Text {
                                text: "Header 1".into()
                            }],
                            is_header: true,
                        },
                        WikitextSimplifiedTableCell {
                            attributes: None,
                            content: spanned_vec![WSN::Text {
                                text: "Header 2".into()
                            }],
                            is_header: true,
                        }
                    ]
                },
                WikitextSimplifiedTableRow {
                    attributes: vec![],
                    cells: vec![
                        WikitextSimplifiedTableCell {
                            attributes: None,
                            content: spanned_vec![WSN::Text {
                                text: "Cell 1".into()
                            }],
                            is_header: false,
                        },
                        WikitextSimplifiedTableCell {
                            attributes: None,
                            content: spanned_vec![WSN::Text {
                                text: "Cell 2".into()
                            }],
                            is_header: false,
                        }
                    ]
                },
                WikitextSimplifiedTableRow {
                    attributes: vec![],
                    cells: vec![
                        WikitextSimplifiedTableCell {
                            attributes: None,
                            content: spanned_vec![WSN::Text {
                                text: "Cell 3".into()
                            }],
                            is_header: false,
                        },
                        WikitextSimplifiedTableCell {
                            attributes: None,
                            content: spanned_vec![WSN::Text {
                                text: "Cell 4".into()
                            }],
                            is_header: false,
                        }
                    ]
                }
            ]
        }]
    );
}

#[test]
fn test_redirect() {
    let wikitext = "#REDIRECT [[Target Page]]";
    let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
    assert_eq!(
        simplified,
        vec![sp(WSN::Redirect {
            target: "Target Page".into()
        }, 0, 25)]
    );
}

#[test]
fn can_handle_nested_defaults_in_template_parameters() {
    let wikitext = r#"[[Lua/{{{1}}}/{{{2}}}/Functions/{{{3}}}|{{{4|{{{2}}}:{{{3}}}}}}]]"#;
    let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
    // This is really a bit busted, but I can see the argument for parsing like this:
    // it doesn't make sense to resolve the outer tags when the parameters are unhygenic
    // text replacements. I suspect the best way to handle this is to apply the parameter
    // substitutions and then reparse the result.
    assert_eq!(
        simplified,
        spanned_vec![
            WSN::Text {
                text: "[[Lua/".to_string()
            },
            WSN::TemplateParameterUse {
                name: "1".into(),
                default: None
            },
            WSN::Text { text: "/".into() },
            WSN::TemplateParameterUse {
                name: "2".into(),
                default: None
            },
            WSN::Text {
                text: "/Functions/".into()
            },
            WSN::TemplateParameterUse {
                name: "3".into(),
                default: None
            },
            WSN::Text { text: "|".into() },
            WSN::TemplateParameterUse {
                name: "4".into(),
                default: Some(spanned_vec![
                    WSN::TemplateParameterUse {
                        name: "2".into(),
                        default: None
                    },
                    WSN::Text { text: ":".into() },
                    WSN::TemplateParameterUse {
                        name: "3".into(),
                        default: None
                    }
                ])
            },
            WSN::Text { text: "]]".into() }
        ]
    );
}

#[test]
fn can_handle_conventional_tags() {
    let wikitext = r#"<syntaxhighlight line>
effects = {}

-- Make sure to clean up everything on ModuleUnload.
Events:Subscribe("ModuleUnload", function()
	for index, effect in ipairs(effects) do
		effect:Remove()
	end
end)
</syntaxhighlight>"#;
    let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
    assert_eq!(
        simplified,
        spanned_vec![WSN::Tag {
            name: "syntaxhighlight".into(),
            attributes: Some("line".into()),
            children: spanned_vec![
                WSN::Text {
                    text: "\neffects = {}\n\n-- Make sure to clean up everything on ModuleUnload.\nEvents:Subscribe(\"ModuleUnload\", function()\n\tfor index, effect in ipairs(effects) do\n\t\teffect:Remove()\n\tend\nend)\n".into(),
                }
            ]
        }]
    );
}

#[test]
fn can_handle_horizontal_divider() {
    let wikitext = "----";
    let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
    assert_eq!(simplified, vec![sp(WSN::HorizontalDivider, 0, 4)]);
}

#[test]
fn returns_verbatim_texts_for_unclosed_single_tags() {
    {
        let wikitext = r#"<font size="3">"#;
        let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
        assert_eq!(
            simplified,
            spanned_vec![WSN::Text {
                text: r#"<font size="3">"#.into()
            }]
        );
    }
    {
        let wikitext = r#"</font>"#;
        let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
        assert_eq!(
            simplified,
            spanned_vec![WSN::Text {
                text: r#"</font>"#.into()
            }]
        );
    }
}

#[test]
fn can_handle_lists_underneath_headers() {
    let wikitext = r#"==0.1.4a (Available on the publicbeta branch)==

====New features====

* Shared
** Overhauled the logging system to support unicode (the first of many unicode additions to come)
** Added console command for profiling Lua modules; usage: profiler_sample {{Arg|number_of_seconds}}"#;
    let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
    assert_eq!(
        simplified,
        spanned_vec![
            WSN::Heading {
                level: 2,
                children: spanned_vec![WSN::Text {
                    text: "0.1.4a (Available on the publicbeta branch)".into()
                }]
            },
            WSN::Heading {
                level: 4,
                children: spanned_vec![WSN::Text {
                    text: "New features".into()
                }]
            },
            WSN::UnorderedList {
                items: vec![WikitextSimplifiedListItem {
                    content: spanned_vec![
                        WSN::Text { text: "Shared".into() },
                        WSN::UnorderedList {
                            items: vec![
                                WikitextSimplifiedListItem {
                                    content: spanned_vec![WSN::Text {
                                        text: "Overhauled the logging system to support unicode (the first of many unicode additions to come)".into()
                                    }]
                                },
                                WikitextSimplifiedListItem {
                                    content: spanned_vec![
                                        WSN::Text {
                                            text: "Added console command for profiling Lua modules; usage: profiler_sample ".into()
                                        },
                                        WSN::Template {
                                            name: "Arg".into(),
                                            parameters: vec![TemplateParameter {
                                                name: "1".into(),
                                                value: "number_of_seconds".into()
                                            }]
                                        }
                                    ]
                                }
                            ]
                        }
                    ]
                }]
            }
        ]
    );
}

#[test]
fn test_to_wikitext_basic() {
    let node = WSN::Text {
        text: "Hello, world!".into(),
    };
    assert_eq!(node.to_wikitext(), "Hello, world!");
}

#[test]
fn test_to_wikitext_bold() {
    let node = WSN::Bold {
        children: spanned_vec![WSN::Text {
            text: "bold text".into(),
        }],
    };
    assert_eq!(node.to_wikitext(), "'''bold text'''");
}

#[test]
fn test_to_wikitext_italic() {
    let node = WSN::Italic {
        children: spanned_vec![WSN::Text {
            text: "italic text".into(),
        }],
    };
    assert_eq!(node.to_wikitext(), "''italic text''");
}

#[test]
fn test_to_wikitext_bold_italic() {
    let node = WSN::Bold {
        children: spanned_vec![WSN::Italic {
            children: spanned_vec![WSN::Text {
                text: "bold italic text".into(),
            }],
        }],
    };
    assert_eq!(node.to_wikitext(), "'''''bold italic text'''''");
}

#[test]
fn test_to_wikitext_link() {
    let node = WSN::Link {
        text: "Main Page".into(),
        title: "Main Page".into(),
    };
    assert_eq!(node.to_wikitext(), "[[Main Page]]");

    let node = WSN::Link {
        text: "Home".into(),
        title: "Main Page".into(),
    };
    assert_eq!(node.to_wikitext(), "[[Main Page|Home]]");
}

#[test]
fn test_to_wikitext_ext_link() {
    let node = WSN::ExtLink {
        link: "https://example.com".into(),
        text: None,
    };
    assert_eq!(node.to_wikitext(), "[https://example.com]");

    let node = WSN::ExtLink {
        link: "https://example.com".into(),
        text: Some("Example".into()),
    };
    assert_eq!(node.to_wikitext(), "[https://example.com Example]");
}

#[test]
fn test_to_wikitext_template() {
    let node = WSN::Template {
        name: "Template".into(),
        parameters: vec![],
    };
    assert_eq!(node.to_wikitext(), "{{Template}}");

    let node = WSN::Template {
        name: "Template".into(),
        parameters: vec![
            TemplateParameter {
                name: "param1".into(),
                value: "value1".into(),
            },
            TemplateParameter {
                name: "param2".into(),
                value: "value2".into(),
            },
        ],
    };
    assert_eq!(
        node.to_wikitext(),
        "{{Template|param1=value1|param2=value2}}"
    );

    let node = WSN::Template {
        name: "Template".into(),
        parameters: vec![
            TemplateParameter {
                name: "1".into(),
                value: "value1".into(),
            },
            TemplateParameter {
                name: "2".into(),
                value: "value2".into(),
            },
        ],
    };
    assert_eq!(node.to_wikitext(), "{{Template|value1|value2}}");
}

#[test]
fn test_to_wikitext_heading() {
    let node = WSN::Heading {
        level: 2,
        children: spanned_vec![WSN::Text {
            text: "Heading".into(),
        }],
    };
    assert_eq!(node.to_wikitext(), "== Heading ==");
}

#[test]
fn test_to_wikitext_tag() {
    let node = WSN::Tag {
        name: "span".into(),
        attributes: None,
        children: spanned_vec![WSN::Text {
            text: "Hello".into(),
        }],
    };
    assert_eq!(node.to_wikitext(), "<span>Hello</span>");

    let node = WSN::Tag {
        name: "span".into(),
        attributes: Some("style=\"color:red\"".into()),
        children: spanned_vec![WSN::Text {
            text: "Red text".into(),
        }],
    };
    assert_eq!(
        node.to_wikitext(),
        "<span style=\"color:red\">Red text</span>"
    );
}

#[test]
fn test_to_wikitext_table() {
    let expected = r#"
{|class="wikitable"
|+Caption
|-
|Cell 1|Cell 2
|}
"#
    .trim_start();

    let node = WSN::Table {
        attributes: spanned_vec![WSN::Text {
            text: "class=\"wikitable\"".into(),
        }],
        captions: vec![WikitextSimplifiedTableCaption {
            attributes: None,
            content: spanned_vec![WSN::Text {
                text: "Caption".into(),
            }],
        }],
        rows: vec![WikitextSimplifiedTableRow {
            attributes: vec![],
            cells: vec![
                WikitextSimplifiedTableCell {
                    attributes: None,
                    content: spanned_vec![WSN::Text {
                        text: "Cell 1".into(),
                    }],
                    is_header: false,
                },
                WikitextSimplifiedTableCell {
                    attributes: None,
                    content: spanned_vec![WSN::Text {
                        text: "Cell 2".into(),
                    }],
                    is_header: false,
                },
            ],
        }],
    };
    assert_eq!(node.to_wikitext(), expected);
}

#[test]
fn test_to_wikitext_table_representative() {
    let expected = r#"
{|
!width="120" align="right"|<font size="3">Returns</font> &nbsp;&nbsp;
|<font size="3">None</font>
|}
"#
    .trim_start();

    let node = WSN::Table {
        attributes: vec![],
        captions: vec![],
        rows: vec![WikitextSimplifiedTableRow {
            attributes: vec![],
            cells: vec![
                WikitextSimplifiedTableCell {
                    attributes: Some(spanned_vec![WSN::Text {
                        text: "width=\"120\" align=\"right\"".into(),
                    }]),
                    content: spanned_vec![
                        WSN::Tag {
                            name: "font".into(),
                            attributes: Some("size=\"3\"".into()),
                            children: spanned_vec![WSN::Text {
                                text: "Returns".into(),
                            }],
                        },
                        WSN::Text { text: " ".into() },
                        WSN::Text {
                            text: "\u{a0}".into(),
                        },
                        WSN::Text {
                            text: "\u{a0}".into(),
                        },
                    ],
                    is_header: true,
                },
                WikitextSimplifiedTableCell {
                    attributes: None,
                    content: spanned_vec![WSN::Tag {
                        name: "font".into(),
                        attributes: Some("size=\"3\"".into()),
                        children: spanned_vec![WSN::Text {
                            text: "None".into(),
                        }],
                    }],
                    is_header: false,
                },
            ],
        }],
    };
    assert_eq!(node.to_wikitext(), expected);
}

#[test]
fn test_to_wikitext_list() {
    let node = WSN::OrderedList {
        items: vec![
            WikitextSimplifiedListItem {
                content: spanned_vec![WSN::Text {
                    text: "Item 1".into(),
                }],
            },
            WikitextSimplifiedListItem {
                content: spanned_vec![WSN::Text {
                    text: "Item 2".into(),
                }],
            },
        ],
    };
    assert_eq!(node.to_wikitext(), "#Item 1\n#Item 2\n");

    let node = WSN::UnorderedList {
        items: vec![
            WikitextSimplifiedListItem {
                content: spanned_vec![WSN::Text {
                    text: "Item 1".into(),
                }],
            },
            WikitextSimplifiedListItem {
                content: spanned_vec![WSN::Text {
                    text: "Item 2".into(),
                }],
            },
        ],
    };
    assert_eq!(node.to_wikitext(), "*Item 1\n*Item 2\n");
}

#[test]
fn test_to_wikitext_redirect() {
    let node = WSN::Redirect {
        target: "Target Page".into(),
    };
    assert_eq!(node.to_wikitext(), "#REDIRECT [[Target Page]]");
}

#[test]
fn test_to_wikitext_special_nodes() {
    assert_eq!(WSN::HorizontalDivider.to_wikitext(), "----");
    assert_eq!(WSN::ParagraphBreak.to_wikitext(), "<br/>");
    assert_eq!(WSN::Newline.to_wikitext(), "\n");
}

#[test]
fn test_to_wikitext_nested() {
    let node = WSN::Fragment {
        children: spanned_vec![
            WSN::Text {
                text: "This is ".into(),
            },
            WSN::Bold {
                children: spanned_vec![WSN::Text {
                    text: "bold".into(),
                }],
            },
            WSN::Text {
                text: ", this is ".into(),
            },
            WSN::Italic {
                children: spanned_vec![WSN::Text {
                    text: "italic".into(),
                }],
            },
            WSN::Text {
                text: ", and this is ".into(),
            },
            WSN::Bold {
                children: spanned_vec![WSN::Italic {
                    children: spanned_vec![WSN::Text {
                        text: "bold italic".into(),
                    }],
                }],
            },
        ],
    };
    assert_eq!(
        node.to_wikitext(),
        "This is '''bold''', this is ''italic'', and this is '''''bold italic'''''"
    );
}

#[test]
fn test_multiline_wikitext_roundtrip() {
    let sample = r#"----
{|
!width="120" align="right"|<font size="3">Returns</font> &nbsp;&nbsp;
|<font size="3">[[Lua/Server/CellID|CellID]]</font>
|-
!width="120" align="right"|<font size="3">Prototype</font> &nbsp;&nbsp;
|<font size="3">StreamableObject:GetCellId()</font>
|-
!width="120" align="right"|<font size="3">Description</font> &nbsp;&nbsp;
|<font size="3">No description</font>
|}
<br/>"#;
    let simplified = parse_and_simplify_wikitext(sample, &PWT_CONFIGURATION).unwrap();
    assert_eq!(
        WSN::Fragment {
            children: simplified
        }
        .to_wikitext(),
        sample
    );
}

#[test]
fn test_warning_box_instantiated_table() {
    let sample = r#"<center>
{|border="1"
|- style="background:#e02020; color:white"
!width="800" height="50"|<br/><font size="3">Please note: This documentation is a major work in progress.<br/>Expect it to be greatly improved over time.</font>
|}
</center>"#;
    let simplified = parse_and_simplify_wikitext(sample, &PWT_CONFIGURATION).unwrap();
    assert_eq!(
        WSN::Fragment {
            children: simplified
        }
        .to_wikitext(),
        sample
    );
}

#[test]
fn test_definition_list() {
    let wikitext = r#";Term 1
:Definition 1
;Term 2
:Definition 2"#;
    let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
    assert_eq!(
        simplified,
        spanned_vec![WSN::DefinitionList {
            items: vec![
                WikitextSimplifiedDefinitionListItem {
                    type_: DefinitionListItemType::Term,
                    content: spanned_vec![WSN::Text {
                        text: "Term 1".into()
                    }]
                },
                WikitextSimplifiedDefinitionListItem {
                    type_: DefinitionListItemType::Details,
                    content: spanned_vec![WSN::Text {
                        text: "Definition 1".into()
                    }]
                },
                WikitextSimplifiedDefinitionListItem {
                    type_: DefinitionListItemType::Term,
                    content: spanned_vec![WSN::Text {
                        text: "Term 2".into()
                    }]
                },
                WikitextSimplifiedDefinitionListItem {
                    type_: DefinitionListItemType::Details,
                    content: spanned_vec![WSN::Text {
                        text: "Definition 2".into()
                    }]
                }
            ]
        }]
    );
}

#[test]
fn test_definition_list_to_wikitext() {
    let node = WSN::DefinitionList {
        items: vec![
            WikitextSimplifiedDefinitionListItem {
                type_: DefinitionListItemType::Term,
                content: spanned_vec![WSN::Text {
                    text: "Term 1".into(),
                }],
            },
            WikitextSimplifiedDefinitionListItem {
                type_: DefinitionListItemType::Details,
                content: spanned_vec![WSN::Text {
                    text: "Definition 1".into(),
                }],
            },
        ],
    };
    assert_eq!(node.to_wikitext(), ";Term 1\n:Definition 1\n");
}

#[test]
fn test_definition_list_with_formatting() {
    let wikitext = r#";'''Bold Term'''
:''Italic Definition''"#;
    let simplified = parse_and_simplify_wikitext(wikitext, &PWT_CONFIGURATION).unwrap();
    assert_eq!(
        simplified,
        spanned_vec![WSN::DefinitionList {
            items: vec![
                WikitextSimplifiedDefinitionListItem {
                    type_: DefinitionListItemType::Term,
                    content: spanned_vec![WSN::Bold {
                        children: spanned_vec![WSN::Text {
                            text: "Bold Term".into()
                        }]
                    }]
                },
                WikitextSimplifiedDefinitionListItem {
                    type_: DefinitionListItemType::Details,
                    content: spanned_vec![WSN::Italic {
                        children: spanned_vec![WSN::Text {
                            text: "Italic Definition".into()
                        }]
                    }]
                }
            ]
        }]
    );
}
