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
