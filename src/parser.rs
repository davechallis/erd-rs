use crate::ast::{self, EntityOptions, GlobalOption, GlobalOptionType, HeaderOptions};
use std::collections::HashMap;
use nom::{IResult, branch::alt, InputTakeAtPosition, AsChar,
    error::{ParseError, ErrorKind},
    bytes::{
        complete::{is_not, tag, take_while, take_while1},
    },
    character::{
        complete::{
            alphanumeric1,
            char,
            line_ending,
            one_of,
            space0,
            not_line_ending,
            multispace0,
            multispace1,
        },
        is_alphanumeric
    }, combinator::{
        value,
        map,
        opt,
        all_consuming,
        eof,
    }, multi::{
        many0,
        separated_list0,
    },
    sequence::{
        delimited,
        separated_pair,
        pair,
        terminated,
        preceded,
    }};

pub fn parse_erd<'a>(i: &'a str) -> Result<ast::Erd, String> {
    let a = match parse::<'a, ErdParseError<&str>>(i) {
        Ok((_m, a)) => a,
        Err(err) => return Err(err.to_string()),
    };

    let mut entities = Vec::new();
    let mut relationships = Vec::new();
    let mut title_directive = HashMap::new();
    let mut header_directive = HashMap::new();
    let mut entity_directive = HashMap::new();
    let mut relationship_directive = HashMap::new();

    for o in a.into_iter() {
        match o {
            ast::Ast::Entity(mut e) => {
                e.options.merge_hashmap(&entity_directive)?;
                e.header_options.merge_hashmap(&header_directive)?;
                entities.push(e);
            },
            ast::Ast::Relation(mut r) => {
                r.options.merge_hashmap(&relationship_directive)?;
                relationships.push(r);
            },
            ast::Ast::Attribute(a) => {
                match entities.last_mut() {
                    Some(e) => e.add_attribute(a),
                    None => return Err(String::from("found attribute without a preceding entity to attach it to")),
                }
            },
            ast::Ast::GlobalOption(ast::GlobalOption { option_type, options }) => {
                use ast::GlobalOptionType::*;
                match option_type {
                    Title => title_directive.extend(options),
                    Header => header_directive.extend(options),
                    Entity => entity_directive.extend(options),
                    Relationship => relationship_directive.extend(options),
                }
            }
        }
    }

    let mut title_options = ast::TitleOptions::default();
    title_options.merge_hashmap(&title_directive)?;
    Ok(ast::Erd { entities, relationships, title_options })
}

fn parse<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&str, Vec<ast::Ast>, ErdParseError<&str>> {
    let (i, mut global_opts) = many0(
        delimited(
            blank_or_comment,
            map(global_option, |g| ast::Ast::GlobalOption(g)),
            blank_or_comment,
        )
    )(i)?;

    let (_, mut era) = all_consuming(
        delimited(
            blank_or_comment,

            many0(
                delimited(
                    blank_or_comment,
                    alt((
                        map(entity, |e| ast::Ast::Entity(e)),
                        map(relation, |r| ast::Ast::Relation(r)),
                        map(attribute, |a| ast::Ast::Attribute(a)),
                    )),
                    blank_or_comment,
                )
            ),

            blank_or_comment,
        )
    )(i)?;

    global_opts.append(&mut era);
    Ok((i, global_opts))
}

fn comment(i: &str) -> IResult<&str, &str, ErdParseError<&str>> {
    delimited(char('#'), not_line_ending, alt((line_ending, eof)))(i)
}

fn blank_or_comment(i: &str) -> IResult<&str, Vec<&str>, ErdParseError<&str>> {
    many0(alt((multispace1, comment)))(i)
}

fn multispace_comma0<T, E: ParseError<T>>(input: T) -> IResult<T, T, E>
where
  T: InputTakeAtPosition,
  <T as InputTakeAtPosition>::Item: AsChar + Clone,
{
  input.split_at_position_complete(|item| {
    let c = item.as_char();
    !(c == ' ' || c == '\t' || c == '\r' || c == '\n' || c == ',')
  })
}

fn multispace0_comment(i: &str) -> IResult<&str, (), ErdParseError<&str>> {
    value(
        (),
        delimited(
            multispace0,
            opt(comment),
            multispace0,
        )
    )(i)
}

fn eol_comment(i: &str) -> IResult<&str, (), ErdParseError<&str>> {
    value(
        (),
        delimited(
            space0, 
            alt((
                line_ending,
                comment,
            )),
            multispace0,
        )
    )(i)
}


fn entity(i: &str) -> IResult<&str, ast::Entity, ErdParseError<&str>> {
    let (i, name) = delimited(char('['), ident, char(']'))(i)?;
    let (i, opts) = trailing_options(i)?;

    let entity_options = match EntityOptions::from_hashmap(&opts) {
        Ok(o) => o,
        Err(e) => return Err(nom::Err::Error(ErdParseError::InvalidOption(e))),
    };

    let header_options = match HeaderOptions::from_hashmap(&opts) {
        Ok(o) => o,
        Err(e) => return Err(nom::Err::Error(ErdParseError::InvalidOption(e))),
    };

    Ok((i, ast::Entity {
        name: name.to_owned(),
        attribs: Vec::new(),
        options: entity_options,
        header_options: header_options,
     }))
}

fn attribute(i: &str) -> IResult<&str, ast::Attribute, ErdParseError<&str>> {
    let (i, key_types) = many0(one_of("*+ \t"))(i)?;

    let (i, field) = ident(i)?;
    let mut attr = ast::Attribute::with_field(field);
    for key_type in key_types {
        match key_type {
            '*' => attr.pk = true,
            '+' => attr.fk = true,
            ' ' | '\t' => {},
            _   => panic!("unhandled key type: {:?}", key_type)
        }
    }

    let (i, opts) = trailing_options(i)?;

    let options = match ast::AttributeOptions::from_hashmap(&opts) {
        Ok(o) => o,
        Err(e) => return Err(nom::Err::Error(ErdParseError::InvalidOption(e))),
    };

    attr.options = options;
    Ok((i, attr))
}

fn relation(i: &str) -> IResult<&str, ast::Relation, ErdParseError<&str>> {
    let (i, entity1) = ident(i)?;
    let (i, (card1, card2)) = separated_pair(
        cardinality,
        tag("--"),
        cardinality,
    )(i)?;
    let (i, entity2) = ident(i)?;
    let (i, opts) = trailing_options(i)?;

    let options = match ast::RelationshipOptions::from_hashmap(&opts) {
        Ok(o) => o,
        Err(e) => return Err(nom::Err::Error(ErdParseError::InvalidOption(e))),
    };

    let rel = ast::Relation {
        entity1: entity1.to_owned(), 
        entity2: entity2.to_owned(), 
        card1: card1.to_owned(), 
        card2: card2.to_owned(), 
        options,
    };
    Ok((i, rel))
}

fn cardinality(i: &str) -> IResult<&str, ast::Cardinality, ErdParseError<&str>> {
    let (i, op) = one_of("?1*+")(i)?;
    let c = match op {
        '?' => ast::Cardinality::ZeroOne,
        '1' => ast::Cardinality::One,
        '*' => ast::Cardinality::ZeroPlus,
        '+' => ast::Cardinality::OnePlus,
        _ => panic!("unhandled cardinality operand"),
    };
    Ok((i, c))
}

fn global_option(i: &str) -> IResult<&str, GlobalOption, ErdParseError<&str>> {
    let (i, name) = alt((
        tag("title"),
        tag("header"),
        tag("entity"),
        tag("relationship"),
    ))(i)?;

    let option_type = match name {
        "title" => GlobalOptionType::Title,
        "header" => GlobalOptionType::Header,
        "entity" => GlobalOptionType::Entity,
        "relationship" => GlobalOptionType::Relationship,
        _ => panic!("unhandled global optional type"),
    };

    let (i, options) = trailing_options(i)?;
    Ok((i, GlobalOption { option_type, options }))
}

fn option(i: &str) -> IResult<&str, (&str, &str), ErdParseError<&str>> {
    separated_pair(
        alphanumeric1, 
        delimited(space0, char(':'), space0),
        quoted
    )(i)
}

fn trailing_options(i: &str) ->IResult<&str, HashMap<String, String>, ErdParseError<&str>> {
    let (i, opts) = delimited(multispace0, opt(options), space0)(i)?;
    let opts: HashMap<String, String> = if let Some(o) = opts {
        o.into_iter().map(|(k, v)| (k.to_owned(), v.to_owned())).collect()
    } else {
        HashMap::new()
    };
    Ok((i, opts))
}

fn options(i: &str) -> IResult<&str, Vec<(&str, &str)>, ErdParseError<&str>> {
    delimited(
        preceded(char('{'), multispace0),

        separated_list0(
            delimited(
                multispace0_comment,
                char(','),
                multispace0_comment,
            ),
            option
        ),

        terminated(
            delimited(
                multispace_comma0,
                multispace0_comment,
                multispace_comma0,
            ),
            char('}'),
        ),
    )(i)
}

fn quoted(i: &str) -> IResult<&str, &str, ErdParseError<&str>> {
    delimited(char('"'), is_not("\""), char('"'))(i)
}

fn ident(i: &str) -> IResult<&str, &str, ErdParseError<&str>> {
    let (i, id) = delimited(space0, alt((
        ident_quoted,
        ident_no_space,
    )), space0)(i)?;
    Ok((i, id))
}

fn ident_quoted(i: &str) -> IResult<&str, &str, ErdParseError<&str>> {
    let (i, id) = alt((
        delimited(char('"'), take_while(|c: char| !c.is_control() && c != '"'), char('"')),
        delimited(char('\''), take_while(|c: char| !c.is_control() && c != '\''), char('\'')),
        delimited(char('`'), take_while(|c: char| !c.is_control() && c != '`'), char('`')),
    ))(i)?;
    Ok((i, id))
}

fn ident_no_space(i: &str) -> IResult<&str, &str, ErdParseError<&str>> {
    take_while1(|c| is_alphanumeric(c as u8) || c == '_')(i)
}

#[derive(Debug, PartialEq)]
pub enum ErdParseError<I> {
    InvalidOption(String),
    Nom(I, ErrorKind),
}

impl<I> nom::error::ParseError<I> for ErdParseError<I> {
    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        ErdParseError::Nom(input, kind)
    }

    fn append(_: I, _: ErrorKind, other: Self) -> Self {
        other
    }
}

#[cfg(test)]
mod tests {
    use std::include_str;

    use maplit::hashmap;

    use super::*;

    #[test]
    fn test_parse_empty() {
        let s = "";
        let e = parse_erd(s).unwrap();
        assert_eq!(e.entities.len(), 0);
        assert_eq!(e.relationships.len(), 0);
    }

    #[test]
    fn test_parse_single_comment() {
        let s = "# Comment.";
        let e = parse_erd(s).unwrap();
        assert_eq!(e.entities.len(), 0);
        assert_eq!(e.relationships.len(), 0);
    }

    #[test]
    fn test_parse_simple() {
        let s = include_str!("../examples/simple.er");
        let e = parse_erd(s).unwrap();
        assert_eq!(e.entities.len(), 2);
        assert_eq!(e.relationships.len(), 1);
    }

    #[test]
    fn test_parse_nfldb() {
        let s = include_str!("../examples/nfldb.er");
        let e = parse_erd(s).unwrap();
        assert_eq!(e.entities.len(), 7);
        assert_eq!(e.relationships.len(), 13);
    }

    #[test]
    fn test_blank_or_comment_empty() {
        blank_or_comment("").unwrap();
    }

    #[test]
    fn test_blank_or_comment_no_eol() {
        blank_or_comment("# comment").unwrap();
    }

    #[test]
    fn test_blank_or_comment_eol() {
        blank_or_comment("# comment\n").unwrap();
    }

    #[test]
    fn test_blank_or_comment_whitespace() {
        blank_or_comment("  # comment \n  ").unwrap();
    }

    #[test]
    fn test_comments() {
        assert_eq!(comment("# foo\r\n"), Ok(("", " foo")));
    }

    #[test]
    fn test_entity_simple() {
        let (i, e) = entity("[foo]").unwrap();
        assert!(i.is_empty());
        assert_eq!(e, ast::Entity::new("foo"));
    }

    #[test]
    fn test_entity_quoted() {
        let (i, e) = entity("[\"foo bar\"]").unwrap();
        assert!(i.is_empty());
        assert_eq!(e, ast::Entity::new("foo bar"));
    }

    #[test]
    fn test_entity_with_option() {
        let (i, e) = entity("[foo] {color: \"#1234AA\"}").unwrap();
        assert!(i.is_empty());
        let mut expected = ast::Entity::new("foo");
        let o = &hashmap!{"color".to_owned() => "#1234AA".to_owned()};
        expected.options = EntityOptions::from_hashmap(o).unwrap();
        expected.header_options = HeaderOptions::from_hashmap(o).unwrap();
        assert_eq!(e, expected);
    }

    #[test]
    fn test_entity_quoted_with_multiple_options() {
        let (i, e) = entity("[`foo - bar`] {size: \"10\", font: \"Equity\"}").unwrap();
        assert!(i.is_empty());
        let mut expected = ast::Entity::new("foo - bar");
        let o = &hashmap!{
            "size".to_owned() => "10".to_owned(),
            "font".to_owned() => "Equity".to_owned(),
        };
        expected.options = EntityOptions::from_hashmap(o).unwrap();
        expected.header_options = HeaderOptions::from_hashmap(o).unwrap();
        assert_eq!(e, expected);
    }

    #[test]
    fn test_attribute_simple() {
        let (i, attr) = attribute("foo").unwrap();
        assert_eq!(attr, ast::Attribute::with_field("foo"));
        assert!(i.is_empty());
    }

    #[test]
    fn test_attribute_pk() {
        let (i, attr) = attribute("*foo").unwrap();
        assert_eq!(attr, ast::Attribute {
            field: "foo".to_owned(),
            pk: true,
            ..Default::default()
        });
        assert!(i.is_empty());
    }

    #[test]
    fn test_attribute_multiple_pk() {
        let (i, attr) = attribute("***foo").unwrap();
        assert_eq!(attr, ast::Attribute {
            field: "foo".to_owned(),
            pk: true,
            ..Default::default()
        });
        assert!(i.is_empty());
    }

    #[test]
    fn test_attribute_fk() {
        let (i, attr) = attribute("+foo").unwrap();
        assert_eq!(attr, ast::Attribute {
            field: "foo".to_owned(),
            fk: true,
            ..Default::default()
        });
        assert!(i.is_empty());
    }

    #[test]
    fn test_attribute_pk_fk() {
        let (i, attr) = attribute("+*foo").unwrap();
        assert_eq!(attr, ast::Attribute {
            field: "foo".to_owned(),
            pk: true,
            fk: true,
            ..Default::default()
         });
        assert!(i.is_empty());
    }

    #[test]
    fn test_attribute_multiple_pk_fk() {
        let (i, attr) = attribute("***++*foo").unwrap();
        assert_eq!(attr, ast::Attribute {
            field: "foo".to_owned(),
            pk: true,
            fk: true,
            ..Default::default()
        });
        assert!(i.is_empty());
    }

    #[test]
    fn test_attribute_whitespace() {
        let (i, attr) = attribute("  \t foo").unwrap();
        assert_eq!(attr, ast::Attribute {
            field: "foo".to_owned(),
            ..Default::default()
        });
        assert!(i.is_empty());
    }

    #[test]
    fn test_attribute_with_options() {
        let (i, attr) = attribute("*foo {label:\"b\", border : \"3\"}").unwrap();
        assert_eq!(attr, ast::Attribute {
            field: "foo".to_owned(),
            pk: true,
            fk: false,
            options: ast::AttributeOptions::from_hashmap(&hashmap!{
                "label".to_owned() => "b".to_owned(),
                "border".to_owned() => "3".to_owned(),
            }).unwrap(),
        });
        assert!(i.is_empty());
    }

    #[test]
    fn test_attribute_with_multiline_options() {
        let (i, attr) = attribute(r#"*foo {
            label:"b",
            border : "3"
        }"#).unwrap();
        assert_eq!(attr, ast::Attribute {
            field: "foo".to_owned(),
            pk: true,
            fk: false,
            options: ast::AttributeOptions::from_hashmap(&hashmap!{
                "label".to_owned() => "b".to_owned(),
                "border".to_owned() => "3".to_owned(),
            }).unwrap(),
        });
        assert!(i.is_empty());
    }

    #[test]
    fn test_attribute_with_multiline_options_trailing_comments() {
        let (i, attr) = attribute(r#"*foo {
            label:"b",
            border : "3", # comment
        }"#).unwrap();
        assert_eq!(attr, ast::Attribute {
            field: "foo".to_owned(),
            pk: true,
            fk: false,
            options: ast::AttributeOptions::from_hashmap(&hashmap!{
                "label".to_owned() => "b".to_owned(),
                "border".to_owned() => "3".to_owned(),
            }).unwrap(),
        });
        assert!(i.is_empty());
    }

    #[test]
    fn test_relation_one_oneplus() {
        let (i, rel) = relation("E1 1--+ E2").unwrap();
        assert!(i.is_empty());
        assert_eq!(rel, ast::Relation {
            entity1: "E1".to_owned(),
            entity2: "E2".to_owned(),
            card1: ast::Cardinality::One,
            card2: ast::Cardinality::OnePlus,
            options: ast::RelationshipOptions::default(),
        });
    }

    #[test]
    fn test_relation_zeroplus_zeroone() {
        let (i, rel) = relation("`Entity 1` *--? 'Entity 2'").unwrap();
        assert!(i.is_empty());
        assert_eq!(rel, ast::Relation {
            entity1: "Entity 1".to_owned(),
            entity2: "Entity 2".to_owned(),
            card1: ast::Cardinality::ZeroPlus,
            card2: ast::Cardinality::ZeroOne,
            options: ast::RelationshipOptions::default(),
        });
    }

    #[test]
    fn test_relation_with_options() {
        let (i, rel) = relation(r##"E1 1--1 E2 {color:"#000000", size: "1"}"##).unwrap();
        assert!(i.is_empty());
        assert_eq!(rel, ast::Relation {
            entity1: "E1".to_owned(),
            entity2: "E2".to_owned(),
            card1: ast::Cardinality::One,
            card2: ast::Cardinality::One,
            options: ast::RelationshipOptions::from_hashmap(&hashmap!{
                "color".to_owned() => "#000000".to_owned(),
                "size".to_owned() => "1".to_owned(),
            }).unwrap(),
        });
    }

    #[test]
    fn test_ident_no_space() {
        let (i, id) = ident_no_space("foo").unwrap();
        assert!(i.is_empty());
        assert_eq!(id, "foo");

        let (i, id) = ident_no_space("foo_BAR").unwrap();
        assert!(i.is_empty());
        assert_eq!(id, "foo_BAR");
    }

    #[test]
    fn test_ident_quoted() {
        let (i, id) = ident_quoted("\"foo\"").unwrap();
        assert!(i.is_empty());
        assert_eq!(id, "foo");

        let (i, id) = ident_quoted("'foo'").unwrap();
        assert!(i.is_empty());
        assert_eq!(id, "foo");

        let (i, id) = ident_quoted("`foo`").unwrap();
        assert!(i.is_empty());
        assert_eq!(id, "foo");

        let (i, id) = ident_quoted("\"foo_BAR\"").unwrap();
        assert!(i.is_empty());
        assert_eq!(id, "foo_BAR");

        let (i, id) = ident_quoted("\"foo - 'foo@bar' BAR\"").unwrap();
        assert!(i.is_empty());
        assert_eq!(id, "foo - 'foo@bar' BAR");
    }

    #[test]
    fn test_ident() {
        let (i, id) = ident("\"foo\"").unwrap();
        assert!(i.is_empty());
        assert_eq!(id, "foo");

        let (i, id) = ident("'foo'").unwrap();
        assert!(i.is_empty());
        assert_eq!(id, "foo");

        let (i, id) = ident("`foo`").unwrap();
        assert!(i.is_empty());
        assert_eq!(id, "foo");

        let (i, id) = ident("\"foo_BAR\"").unwrap();
        assert!(i.is_empty());
        assert_eq!(id, "foo_BAR");

        let (i, id) = ident("\"foo - 'foo@bar' BAR\"").unwrap();
        assert_eq!(i, "");
        assert_eq!(id, "foo - 'foo@bar' BAR");

        let (i, id) = ident(" foo ").unwrap();
        assert!(i.is_empty());
        assert_eq!(id, "foo");

        let (i, id) = ident(" \t'foo'\t ").unwrap();
        assert!(i.is_empty());
        assert_eq!(id, "foo");

        let (i, id) = ident(" \t `foo \"and\" bar` \t ").unwrap();
        assert!(i.is_empty());
        assert_eq!(id, "foo \"and\" bar");
    }

    #[test]
    fn test_option() {
        let (i, (key, value)) = option(r#"foo: "bar""#).unwrap();
        assert!(i.is_empty());
        assert_eq!((key, value), ("foo", "bar"));

        let (i, (key, value)) = option(r#"foo:"A longer value?""#).unwrap();
        assert!(i.is_empty());
        assert_eq!((key, value), ("foo", "A longer value?"));
    }

    #[test]
    fn test_options() {
        let (i, opts) = options(r#"{k:"v"}"#).unwrap();
        assert!(i.is_empty());
        assert_eq!(opts, vec![("k", "v")]);

        let (i, opts) = options(r#"{ k:"v" }"#).unwrap();
        assert!(i.is_empty());
        assert_eq!(opts, vec![("k", "v")]);

        let (i, opts) = options(r#"{k1:"v1",k2:"v2"}"#).unwrap();
        assert!(i.is_empty());
        assert_eq!(opts, vec![("k1", "v1"), ("k2", "v2")]);

        let (i, opts) = options(r#"{k:"v1",k:"v2",k:"v3"}"#).unwrap();
        assert!(i.is_empty());
        assert_eq!(opts, vec![("k", "v1"), ("k", "v2"), ("k", "v3")]);

        let (i, opts) = options(r#"{  k1:"v1", k2:"v2" ,  k3:"v3"}"#).unwrap();
        assert!(i.is_empty());
        assert_eq!(opts, vec![("k1", "v1"), ("k2", "v2"), ("k3", "v3")]);
    }

    #[test]
    fn test_options_trailing_comma() {
        let (i, opts) = options(r#"{k1:"v1",}"#).unwrap();
        assert!(i.is_empty());
        assert_eq!(opts, vec![("k1", "v1")]);

        let (i, opts) = options(r#"{k1:"v1", }"#).unwrap();
        assert!(i.is_empty());
        assert_eq!(opts, vec![("k1", "v1")]);

        let (i, opts) = options(r#"{k1:"v1" , }"#).unwrap();
        assert!(i.is_empty());
        assert_eq!(opts, vec![("k1", "v1")]);
    }

    #[test]
    fn test_options_multiline() {
        let i = r##"{
          label: "string",
          color: "#3366ff", # i like bright blue
        }"##;

        let (i, opts) = options(i).unwrap();
        assert!(i.is_empty());
        assert_eq!(opts, vec![("label", "string"), ("color", "#3366ff")]);
    }

    #[test]
    fn test_global_options() {
        let (i, go) = global_option("title {}").unwrap();
        assert!(i.is_empty());
        assert_eq!(go.option_type, GlobalOptionType::Title);
        assert!(go.options.is_empty());

        let (i, go) = global_option(r#"header {k: "v"}"#).unwrap();
        assert!(i.is_empty());
        assert_eq!(go.option_type, GlobalOptionType::Header);
        assert_eq!(go.options.len(), 1);
        assert_eq!(go.options["k"], "v");

        let (i, go) = global_option(r#"entity {k1: "v1", k2: "v2"}"#).unwrap();
        assert!(i.is_empty());
        assert_eq!(go.option_type, GlobalOptionType::Entity);
        assert_eq!(go.options.len(), 2);
        assert_eq!(go.options["k1"], "v1");
        assert_eq!(go.options["k2"], "v2");

        let (i, go) = global_option(r#"relationship{ k1:"X" , k2 :   "v2", k1:"v1" }"#).unwrap();
        println!("{}", i);
        assert!(i.is_empty());
        assert_eq!(go.option_type, GlobalOptionType::Relationship);
        assert_eq!(go.options.len(), 2);
        assert_eq!(go.options["k1"], "v1");
        assert_eq!(go.options["k2"], "v2");

        assert!(global_option(r#"something {}"#).is_err());
    }
}
