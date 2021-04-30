use std::io::{Write, Result};
use crate::ast;

pub fn render<W: Write>(w: &mut W, erd: &ast::Erd) -> Result<()> {
    graph_header(w)?;

    let mut graph_attrs = Vec::new();
    dbg!(&erd.title_options);

    if let Some(label) = &erd.title_options.label {
        graph_attrs.push((
            "label",
            format!("<<FONT POINT-SIZE=\"{}\">{}</FONT>>", erd.title_options.size, label),
        ));
        graph_attrs.push(("labeljust", "l".to_owned()));
        graph_attrs.push(("labelloc", "t".to_owned()));
    }

    graph_attrs.push(("rankdir", "LR".to_owned()));
    graph_attrs.push(("splines", "spline".to_owned()));

    graph_attributes(w, &graph_attrs)?;

    node_attributes(w, &vec![
        ("label", r#""\N""#.to_owned()),
        ("shape", "plaintext".to_owned()),
    ])?;

    edge_attributes(w, &vec![
        ("color", "gray50".to_owned()),
        ("minlen", "2".to_owned()),
        ("style", "dashed".to_owned()),
    ])?;

    for e in &erd.entities {
        render_entity(w, e)?;
    }

    for r in &erd.relationships {
        render_relationship(w, r)?;
    }

    graph_footer(w)?;
 
    Ok(())
}

fn graph_header<W: Write>(w: &mut W) -> Result<()> {
    write!(w, "graph {{\n")
}

fn render_attribute<W: Write>(w: &mut W, a: &ast::Attribute) -> Result<()> {
    let field = match (a.pk, a.fk) {
        (true, true)    => format!("<I><U>{}</U></I>", a.field),
        (true, false)   => format!("<U>{}</U>", a.field),
        (false, true)   => format!("<I>{}</I>", a.field),
        (false, false)  => a.field.clone(),
    };
    match &a.options.label {
        Some(l) => write!(w, "    <TR><TD ALIGN=\"LEFT\">{} [{}]</TD></TR>\n", field, l),
        None => write!(w, "    <TR><TD ALIGN=\"LEFT\">{}</TD></TR>\n", a.field),
    }
}

fn render_relationship<W: Write>(w: &mut W, r: &ast::Relation) -> Result<()> {
    let head_card = match r.card2 {
        ast::Cardinality::ZeroOne => "{0,1}",
        ast::Cardinality::One => "1",
        ast::Cardinality::ZeroPlus => "0..N",
        ast::Cardinality::OnePlus => "1..N",
    };
    let tail_card = match r.card1 {
        ast::Cardinality::ZeroOne => "{0,1}",
        ast::Cardinality::One => "1",
        ast::Cardinality::ZeroPlus => "0..N",
        ast::Cardinality::OnePlus => "1..N",
    };
    write!(w, r#"
    "{}" -- "{}" [ headlabel="{}", taillabel="{}" ];
"#, r.entity1, r.entity2, head_card, tail_card)?;

    Ok(())
}

fn render_entity<W: Write>(w: &mut W, e: &ast::Entity) -> Result<()> {
    write!(w, "    \"{}\" [\n", e.name)?;
    write!(w, r##"        label=<
<FONT FACE="Helvetica">
  <TABLE BORDER="0" CELLBORDER="1" CELLPADDING="4" CELLSPACING="0">
    <TR><TD><B><FONT POINT-SIZE="16">{}</FONT></B></TD></TR>
"##, e.name)?;

    for a in &e.attribs {
        render_attribute(w, a)?;
    }

    write!(w, r#"  </TABLE>
</FONT>
>];
"#)?;

    Ok(())
}

fn graph_attributes<W: Write>(w: &mut W, opts: &Vec<(&str, String)>) -> Result<()> {
    attributes(w, "graph", opts)
}

fn node_attributes<W: Write>(w: &mut W, opts: &Vec<(&str, String)>) -> Result<()> {
    attributes(w, "node", opts)
}

fn edge_attributes<W: Write>(w: &mut W, opts: &Vec<(&str, String)>) -> Result<()> {
    attributes(w, "edge", opts)
}

fn attributes<W: Write>(w: &mut W, name: &str, opts: &Vec<(&str, String)>) -> Result<()> {
    write!(w, "    {} [\n", name)?;
    for (key, value) in opts {
        write!(w, "        {}={},\n", key, value)?;
    }
    write!(w, "    ];\n")?;

    Ok(())
}

fn graph_footer<W: Write>(w: &mut W) -> Result<()> {
    write!(w, "}}\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_erd;
    use std::str::from_utf8;
    use pretty_assertions::{assert_eq, assert_ne};

    #[test]
    fn empty_graph() {
        let erd = ast::Erd::default();
        let mut buf = Vec::new();
        render(&mut buf, &erd).unwrap();
        assert_eq!(from_utf8(&buf).unwrap(), r#"graph {
    graph [
        rankdir=LR,
        splines=spline,
    ];
    node [
        label="\N",
        shape=plaintext,
    ];
    edge [
        color=gray50,
        minlen=2,
        style=dashed,
    ];
}
"#);
    }

    #[test]
    fn title_and_entity() {
        let s = r#"
title {label: "Foo"}

[thing]
"#;
        let erd = parse_erd(s).unwrap();
        let mut buf = Vec::new();
        render(&mut buf, &erd).unwrap();
        assert_eq!(from_utf8(&buf).unwrap(), r##"graph {
    graph [
        label=<<FONT POINT-SIZE="30">Foo</FONT>>,
        labeljust=l,
        labelloc=t,
        rankdir=LR,
        splines=spline,
    ];
    node [
        label="\N",
        shape=plaintext,
    ];
    edge [
        color=gray50,
        minlen=2,
        style=dashed,
    ];
    "thing" [
        label=<
<FONT FACE="Helvetica">
  <TABLE BORDER="0" CELLBORDER="1" CELLPADDING="4" CELLSPACING="0">
    <TR><TD><B><FONT POINT-SIZE="16">thing</FONT></B></TD></TR>
  </TABLE>
</FONT>
>];
}
"##);
    }

    #[test]
    fn simple() {
        let s = include_str!("../examples/simple.er");
        let erd = parse_erd(s).unwrap();
        let mut buf = Vec::new();
        render(&mut buf, &erd).unwrap();
        assert_eq!(from_utf8(&buf).unwrap(), r##"graph {
    graph [
        rankdir=LR,
        splines=spline,
    ];
    node [
        label="\N",
        shape=plaintext,
    ];
    edge [
        color=gray50,
        minlen=2,
        style=dashed,
    ];
    "Person" [
        label=<
<FONT FACE="Helvetica">
  <TABLE BORDER="0" CELLBORDER="1" CELLPADDING="4" CELLSPACING="0">
    <TR><TD><B><FONT POINT-SIZE="16">Person</FONT></B></TD></TR>
    <TR><TD ALIGN="LEFT">name</TD></TR>
    <TR><TD ALIGN="LEFT">height</TD></TR>
    <TR><TD ALIGN="LEFT">weight</TD></TR>
    <TR><TD ALIGN="LEFT">birth date</TD></TR>
    <TR><TD ALIGN="LEFT">birth_place_id</TD></TR>
  </TABLE>
</FONT>
>];
    "Birth Place" [
        label=<
<FONT FACE="Helvetica">
  <TABLE BORDER="0" CELLBORDER="1" CELLPADDING="4" CELLSPACING="0">
    <TR><TD><B><FONT POINT-SIZE="16">Birth Place</FONT></B></TD></TR>
    <TR><TD ALIGN="LEFT">id</TD></TR>
    <TR><TD ALIGN="LEFT">birth city</TD></TR>
    <TR><TD ALIGN="LEFT">birth state</TD></TR>
    <TR><TD ALIGN="LEFT">birth country</TD></TR>
  </TABLE>
</FONT>
>];

    "Person" -- "Birth Place" [ headlabel="1", taillabel="0..N" ];
}
"##);
 
    }

    #[test]
    fn test_empty_graph_with_opts() {
        let mut buf = Vec::new();
        graph_header(&mut buf).unwrap();
        graph_attributes(
            &mut buf, 
            &vec![
                ("a", "b".to_owned()),
                ("c", "\"d\"".to_owned())
            ],
        ).unwrap();
        graph_footer(&mut buf).unwrap();
        assert_eq!(from_utf8(&buf).unwrap(),
r#"graph {
    graph [
        a=b,
        c="d",
    ];
}
"#);
    }

    #[test]
    fn render_file() {
        // let mut f = std::fs::File::create("/tmp/out.dot").unwrap();
        // graph_header(&mut f).unwrap();

        // graph_attributes(&mut f, &[
        //     ("label", "<<FONT POINT-SIZE=\"20\">T</FONT>>"),
        //     ("labeljust", "l"),
        //     ("labelloc", "t"),
        //     ("rankdir", "LR"),
        //     ("splines", "spline"),
        // ]).unwrap();

        // node_attributes(&mut f, &[
        //     ("label", r#""\N""#),
        //     ("shape", "plaintext"),
        // ]).unwrap();

        // edge_attributes(&mut f, &[
        //     ("color", "gray50"),
        //     ("minlen", "2"),
        //     ("style", "dashed"),
        // ]).unwrap();

        // let s = std::include_str!("../examples/simple.er");
        // let erd = crate::parser::parse_erd(s).unwrap();
        // render(&mut f, &erd).unwrap();
    }
}