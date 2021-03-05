use std::io::{Write, Result};
use crate::ast;

fn render<W: Write>(w: &mut W, erd: &[ast::Ast]) -> Result<()> {
    graph_header(w)?;
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
    match a.options.get("label") {
        Some(l) => write!(w, r#"<TR><TD ALIGN="LEFT">{} [{}]</TD></TR> "#, field, l),
        None => write!(w, r#"<TR><TD ALIGN="LEFT">{}</TD></TR> "#, a.field),
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
    let bgcolor = match e.options.get("bgcolor") {
        Some(c) => c,
        None => "#d0e0d0",
    };
    write!(w, "    \"{}\" [\n", e.name)?;
    write!(w, r##"        label=<
<FONT FACE="Helvetica">
  <TABLE BGCOLOR="{}" BORDER="0" CELLBORDER="1" CELLPADDING="4" CELLSPACING="0">
    <TR><TD><B><FONT POINT-SIZE="16">{}</FONT></B></TD></TR>
"##, bgcolor, e.name)?;

    for a in &e.attribs {
        render_attribute(w, a)?;
    }

    write!(w, r#"
  </TABLE>
</FONT>
>];
"#)?;

    Ok(())
}

fn graph_attributes<W: Write>(w: &mut W, opts: &[(&str, &str)]) -> Result<()> {
    attributes(w, "graph", opts)
}

fn node_attributes<W: Write>(w: &mut W, opts: &[(&str, &str)]) -> Result<()> {
    attributes(w, "node", opts)
}

fn edge_attributes<W: Write>(w: &mut W, opts: &[(&str, &str)]) -> Result<()> {
    attributes(w, "edge", opts)
}

fn attributes<W: Write>(w: &mut W, name: &str, opts: &[(&str, &str)]) -> Result<()> {
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
    use std::str::from_utf8;

    #[test]
    fn test_empty_graph() {
        let mut buf = Vec::new();
        render(&mut buf, &[]).unwrap();
        assert_eq!(from_utf8(&buf).unwrap(), "graph {\n}\n");
    }

    #[test]
    fn test_empty_graph_with_opts() {
        let mut buf = Vec::new();
        graph_header(&mut buf).unwrap();
        graph_attributes(&mut buf, &[("a", "b"), ("c", "\"d\"")]).unwrap();
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
}
