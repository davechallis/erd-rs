use crate::ast::{self, GlobalOption, GlobalOptionType};
use std::collections::HashMap;
use std::hash::Hash;
use nom::{IResult, branch::alt, bytes::{
        complete::{is_not, tag, take_while, take_while1},
    }, character::{
        complete::{
            alphanumeric1,
            char,
            line_ending,
            one_of,
            space0,
            not_line_ending,
            multispace1,
            multispace0,
        },
        is_alphanumeric
    }, combinator::{
        value,
        map,
        opt,
        all_consuming,
    }, multi::{
        many0,
        separated_list1,
    },
    sequence::{
        delimited,
        separated_pair,
        pair,
        terminated,
        preceded,
    }};

pub fn parse_erd(i: &str) -> Result<ast::Erd, String> {
    let a = match parse(i) {
        Ok((_m, a)) => a,
        Err(err) => return Err(err.to_string()),
    };

    let mut entities = Vec::new();
    let mut relationships = Vec::new();
    let mut title_options = HashMap::new();
    let mut header_options = HashMap::new();
    let mut entity_options = HashMap::new();
    let mut relationship_options = HashMap::new();

    for o in a.into_iter() {
        match o {
            ast::Ast::Entity(mut e) => {
                merge_hashmap(&mut e.header_options, &header_options);
                merge_hashmap(&mut e.entity_options, &entity_options);
                entities.push(e);
            },
            ast::Ast::Relation(mut r) => {
                merge_hashmap(&mut r.options, &relationship_options);
                relationships.push(r);
            },
            ast::Ast::Attribute(a) => {
                match entities.last_mut() {
                    Some(e) => e.add_attribute(a),
                    None => return Err(String::from("found attribute without a preceding entity to attach it to")),
                }
            },
            ast::Ast::GlobalOption(GlobalOption { option_type, options }) => {
                match option_type {
                    GlobalOptionType::Title => title_options.extend(options),
                    GlobalOptionType::Header => header_options.extend(options),
                    GlobalOptionType::Entity => entity_options.extend(options),
                    GlobalOptionType::Relationship => relationship_options.extend(options),
                }
            }
        }
    }

    Ok(ast::Erd { entities, relationships, title_options })
}

fn merge_hashmap<K: Eq + Hash + Clone, V: Clone>(dst: &mut HashMap<K, V>, src: &HashMap<K, V>) {
    dst.extend(src.iter().map(|(k, v)| (k.clone(), v.clone())))
}

fn parse(i: &str) -> IResult<&str, Vec<ast::Ast>> {
    let (_, (mut global_opts, mut era)) = all_consuming(
        pair(
            many0(
                delimited(
                    blank_or_comment, 
                    map(global_option, |g| ast::Ast::GlobalOption(g)),
                    eol_comment
                )
            ),
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
            )
        )
    )(i)?;

    global_opts.append(&mut era);
    Ok((i, global_opts))
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
    Ok((i, ast::Entity { name: name.to_owned(), attribs: Vec::new(), header_options: opts.clone(), entity_options: opts }))
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

fn global_option(i: &str) -> IResult<&str, GlobalOption> {
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
    let (i, opts) = delimited(
        terminated(char('{'), space0),
         opt(separated_list1(delimited(space0, char(','), space0), option)),
         preceded(space0, char('}')),
    )(i)?;

    Ok((i, opts.unwrap_or_else(|| Vec::new())))
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

// #[derive(Debug, PartialEq)]
// pub enum ErdParseError<I> {
//     UnrecognisedGlobalOption,
//     Nom(I, nom::error::ErrorKind),
// }

// impl<I> nom::error::ParseError<I> for ErdParseError<I> {
//     fn from_error_kind(input: I, kind: ErrorKind) -> Self {
//         ErdParseError::Nom(input, kind)
//     }

//     fn append(_: I, _: ErrorKind, other: Self) -> Self {
//         other
//     }
//}

#[cfg(test)]
mod tests {
    use std::include_str;

    use maplit::hashmap;

    use super::*;

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