use pulldown_cmark::{html, Event, Parser};

use regex::Regex;
pub type Tag = String;
pub type RenderedHtml = String;

pub fn parse_markdown(markdown_text: &str) -> (Vec<Tag>, RenderedHtml) {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"#[\w\d]+").unwrap();
    }

    let mut tags = vec![];
    let mut html_buf = String::new();

    let mut code_tag_level = 0;
    {
        let parser = Parser::new(markdown_text);

        let parser = parser.map(|event| match event {
            Event::Text(text) => {
                if code_tag_level == 0 {
                    for tag in RE.find_iter(&text) {
                        tags.push(tag.as_str()[1..].to_lowercase());
                    }
                }

                Event::Text(text)
            }
            Event::Start(::pulldown_cmark::Tag::Code) |
            Event::Start(::pulldown_cmark::Tag::CodeBlock(_)) 
                => {
                code_tag_level += 1;
                event
            }
            Event::End(::pulldown_cmark::Tag::Code) |

            Event::End(::pulldown_cmark::Tag::CodeBlock(_)) 
                => {
                assert!(code_tag_level >= 0);
                code_tag_level -= 1;
                event
            }
            _ => event,
        });

        html::push_html(&mut html_buf, parser);
    }

    tags.sort();
    tags.dedup();

    (tags, html_buf)
}

#[test]
fn simple() {
    let (tags, _rendered) = parse_markdown(
        r#"
Foo bar #X.
#foo
#BAR. #bAz
#CięŻarkiewicz #FOO;

* #list

    "#,
    );

    assert_eq!(tags, ["bar", "baz", "ciężarkiewicz", "foo", "list", "x"]);
}

#[test]
fn skip_code() {
    let (tags, rendered) = parse_markdown(
        r#"
Foo bar #X.

    #foo
    #BAR. #bAz;

#CięŻarkiewicz #FOO
    "#,
    );

    assert_eq!(tags, ["ciężarkiewicz", "foo", "x"]);
}
