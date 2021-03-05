use crate::ast;
use std::collections::HashMap;
use nom::{IResult, branch::alt, bytes::{
        complete::{is_a, is_not, tag, take_while, take_while1},
    }, character::{
        complete::{
            alphanumeric1,
            char,
            line_ending,
            one_of,
            satisfy,
            space0,
            not_line_ending,
            multispace1,
            multispace0,
        },
        is_alphabetic,
        is_alphanumeric
    }, combinator::{
        value,
        map,
        opt,
        all_consuming,
    }, multi::{
        many0,
        many1,
        separated_list1,
    },
    sequence::{
        delimited,
        separated_pair,
        pair,
        terminated,
        preceded,
    }};

pub fn parse(i: &str) -> IResult<&str, Vec<ast::Ast>> {
    all_consuming(
        many0(
            delimited(
                blank_or_comment,
                alt((
                    map(entity, |e| ast::Ast::Entity(e)),
                    map(relation, |r| ast::Ast::Relation(r)),
                    map(attribute, |a| ast::Ast::Attribute(a)),
                )),
                eol_comment,
            )
        )
    )(i)
}

fn comment(i: &str) -> IResult<&str, &str> {
    delimited(char('#'), not_line_ending, line_ending)(i)
}

fn blank_or_comment(i: &str) -> IResult<&str, ()> {
    value(
        (),
        many0(
            alt((
                multispace1,
                comment,
            ))
        )
    )(i)
}

fn eol_comment(i: &str) -> IResult<&str, ()> {
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

fn entity(i: &str) -> IResult<&str, ast::Entity> {
    let (i, name) = delimited(char('['), ident, char(']'))(i)?;
    let (i, opts) = trailing_options(i)?;
    Ok((i, ast::Entity { name: name.to_owned(), attribs: Vec::new(), options: opts }))
}

fn attribute(i: &str) -> IResult<&str, ast::Attribute> {
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
    attr.options = opts;
    Ok((i, attr))
}

fn relation(i: &str) -> IResult<&str, ast::Relation> {
    let (i, entity1) = ident(i)?;
    let (i, (card1, card2)) = separated_pair(
        cardinality,
        tag("--"),
        cardinality,
    )(i)?;
    let (i, entity2) = ident(i)?;
    let (i, opts) = trailing_options(i)?;

    let rel = ast::Relation {
        entity1: entity1.to_owned(), 
        entity2: entity2.to_owned(), 
        card1: card1.to_owned(), 
        card2: card2.to_owned(), 
        options: opts,
    };
    Ok((i, rel))
}

fn cardinality(i: &str) -> IResult<&str, ast::Cardinality> {
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

fn option(i: &str) -> IResult<&str, (&str, &str)> {
    separated_pair(
        alphanumeric1, 
        delimited(space0, char(':'), space0),
        quoted
    )(i)
}

fn trailing_options(i: &str) ->IResult<&str, HashMap<String, String>> {
    let (i, opts) = delimited(space0, opt(options), space0)(i)?;
    let opts: HashMap<String, String> = if let Some(o) = opts {
        o.into_iter().map(|(k, v)| (k.to_owned(), v.to_owned())).collect()
    } else {
        HashMap::new()
    };
    Ok((i, opts))
}
fn options(i: &str) -> IResult<&str, Vec<(&str, &str)>> {
    delimited(
        terminated(char('{'), space0),
         separated_list1(delimited(space0, char(','), space0), option), 
         preceded(space0, char('}')),
    )(i)
}

fn quoted(i: &str) -> IResult<&str, &str> {
    delimited(char('"'), is_not("\""), char('"'))(i)
}

fn ident(i: &str) -> IResult<&str, &str> {
    let (i, id) = delimited(space0, alt((
        ident_quoted,
        ident_no_space,
    )), space0)(i)?;
    Ok((i, id))
}

fn ident_quoted(i: &str) -> IResult<&str, &str> {
    let (i, id) = alt((
        delimited(char('"'), take_while(|c: char| !c.is_control() && c != '"'), char('"')),
        delimited(char('\''), take_while(|c: char| !c.is_control() && c != '\''), char('\'')),
        delimited(char('`'), take_while(|c: char| !c.is_control() && c != '`'), char('`')),
    ))(i)?;
    Ok((i, id))
}

fn ident_no_space(i: &str) -> IResult<&str, &str> {
    take_while1(|c| is_alphanumeric(c as u8) || c == '_')(i)
}

#[cfg(test)]
mod tests {
    use std::include_str;

    use maplit::hashmap;

    use super::*;

    #[test]
    fn test_parse_simple() {
        let s = include_str!("../examples/simple.er");
        let (i, a) = parse(s).unwrap();
        for x in a {
            println!("{:?}", x);
        }
        assert!(i.is_empty());
    }

    #[test]
    fn test_parse_nfldb() {
        let s = include_str!("../examples/nfldb.er");
        let (i, a) = parse(s).unwrap();
        for x in a {
            println!("{:?}", x);
        }
        assert!(i.is_empty());
    }

    #[test]
    fn test_comments() {
        assert_eq!(comment("# foo\r\n"), Ok(("", " foo")));
    }

    #[test]
    fn test_entity() {
        let (i, e) = entity("[foo]").unwrap();
        assert!(i.is_empty());
        assert_eq!(e, ast::Entity::with_name("foo"));

        let (i, e) = entity("[\"foo bar\"]").unwrap();
        assert!(i.is_empty());
        assert_eq!(e, ast::Entity::with_name("foo bar"));

        let (i, e) = entity("[foo] {foo: \"bar\"}").unwrap();
        assert!(i.is_empty());
        assert_eq!(e, ast::Entity::new("foo", hashmap!{"foo".to_owned() => "bar".to_owned()}));

        let (i, e) = entity("[`foo - bar`] {a: \"a\", b: \"b\"}").unwrap();
        assert!(i.is_empty());
        assert_eq!(e, ast::Entity::new("foo - bar", hashmap!{
            "a".to_owned() => "a".to_owned(),
            "b".to_owned() => "b".to_owned(),
        }));
    }

    #[test]
    fn test_attribute() {
        let (i, attr) = attribute("foo").unwrap();
        assert_eq!(attr, ast::Attribute::with_field("foo"));
        assert!(i.is_empty());

        let (i, attr) = attribute("*foo").unwrap();
        assert_eq!(attr, ast::Attribute {
            field: "foo".to_owned(),
            pk: true,
            ..Default::default()
        });
        assert!(i.is_empty());

        let (i, attr) = attribute("***foo").unwrap();
        assert_eq!(attr, ast::Attribute {
            field: "foo".to_owned(),
            pk: true,
            ..Default::default()
        });
        assert!(i.is_empty());

        let (i, attr) = attribute("+foo").unwrap();
        assert_eq!(attr, ast::Attribute {
            field: "foo".to_owned(),
            fk: true,
            ..Default::default()
        });
        assert!(i.is_empty());

        let (i, attr) = attribute("+*foo").unwrap();
        assert_eq!(attr, ast::Attribute {
            field: "foo".to_owned(),
            pk: true,
            fk: true,
            ..Default::default()
         });
        assert!(i.is_empty());

        let (i, attr) = attribute("***++*foo").unwrap();
        assert_eq!(attr, ast::Attribute {
            field: "foo".to_owned(),
            pk: true,
            fk: true,
            ..Default::default()
        });
        assert!(i.is_empty());

        let (i, attr) = attribute("  \t foo").unwrap();
        assert_eq!(attr, ast::Attribute {
            field: "foo".to_owned(),
            ..Default::default()
        });
        assert!(i.is_empty());

        let (i, attr) = attribute("*foo {a:\"b\", c : \"d\"}").unwrap();
        assert_eq!(attr, ast::Attribute {
            field: "foo".to_owned(),
            pk: true,
            fk: false,
            options: hashmap!{
                "a".to_owned() => "b".to_owned(),
                "c".to_owned() => "d".to_owned(),
            }
        });
        assert!(i.is_empty());
    }

    #[test]
    fn test_relation() {
        let (i, rel) = relation("E1 1--+ E2").unwrap();
        assert!(i.is_empty());
        assert_eq!(rel, ast::Relation {
            entity1: "E1".to_owned(),
            entity2: "E2".to_owned(),
            card1: ast::Cardinality::One,
            card2: ast::Cardinality::OnePlus,
            options: HashMap::new(),
        });

        let (i, rel) = relation("`Entity 1` *--? 'Entity 2'").unwrap();
        assert!(i.is_empty());
        assert_eq!(rel, ast::Relation {
            entity1: "Entity 1".to_owned(),
            entity2: "Entity 2".to_owned(),
            card1: ast::Cardinality::ZeroPlus,
            card2: ast::Cardinality::ZeroOne,
            options: HashMap::new(),
        });

        let (i, rel) = relation("E1 1--1 E2 {a:\"b\", b: \"c\"}").unwrap();
        assert!(i.is_empty());
        assert_eq!(rel, ast::Relation {
            entity1: "E1".to_owned(),
            entity2: "E2".to_owned(),
            card1: ast::Cardinality::One,
            card2: ast::Cardinality::One,
            options: hashmap!{
                "a".to_owned() => "b".to_owned(),
                "b".to_owned() => "c".to_owned(),
            }
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
}
