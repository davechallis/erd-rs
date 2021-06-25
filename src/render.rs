use std::io::{Write, Result};
use crate::ast;

pub struct Renderer<W: Write> {
    w: W
}

impl<W: Write> Renderer<W> {
    pub fn new(w: W) -> Self {
        Self { w }
    }

    pub fn render_erd(&mut self, erd: &ast::Erd) -> Result<()> {
        self.graph_header()?;

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

        self.graph_attributes(&graph_attrs)?;

        self.node_attributes(&vec![
            ("label", r#""\N""#.to_owned()),
            ("shape", "plaintext".to_owned()),
        ])?;

        self.edge_attributes(&vec![
            ("color", "gray50".to_owned()),
            ("minlen", "2".to_owned()),
            ("style", "dashed".to_owned()),
        ])?;

        for e in &erd.entities {
            self.entity(e)?;
        }

        for r in &erd.relationships {
            self.relationship(r)?;
        }

        self.graph_footer()
    }

    fn graph_header(&mut self) -> Result<()> {
        write!(self.w, "graph {{\n")
    }

    fn render_attribute(&mut self, a: &ast::Attribute) -> Result<()> {
        let field = match (a.pk, a.fk) {
            (true, true)    => format!("<I><U>{}</U></I>", a.field),
            (true, false)   => format!("<U>{}</U>", a.field),
            (false, true)   => format!("<I>{}</I>", a.field),
            (false, false)  => a.field.clone(),
        };
        write!(self.w, "    ")?;
        self.open_tag("TR")?;
        self.open_tag_attrs("TD", &[("ALIGN", "LEFT".to_owned())])?;
        match &a.options.label {
            Some(l) => write!(self.w, "{} [{}]", field, l)?,
            None => write!(self.w, "{}", a.field)?,
        }
        self.close_tag("TD")?;
        self.close_tag("TR")?;
        write!(self.w, "\n")
    }

    fn open_tag(&mut self, tag: &str) -> Result<()> {
        write!(self.w, "<{}>", tag)
    }

    fn open_tag_attrs(&mut self, tag: &str, attrs: &[(&str, String)]) -> Result<()> {
        write!(self.w, "<{}", tag)?;
        for (k, v) in attrs {
            write!(self.w, " {}=\"{}\"", k, v)?;
        }
        write!(self.w, ">")
    }

    fn close_tag(&mut self, tag: &str) -> Result<()> {
        write!(self.w, "</{}>", tag)
    }

    fn relationship(&mut self, r: &ast::Relation) -> Result<()> {
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
        write!(self.w, r#""{}" -- "{}" [ headlabel="{}", taillabel="{}" ];
"#, r.entity1, r.entity2, head_card, tail_card)
    }

    fn entity(&mut self, e: &ast::Entity) -> Result<()> {
        write!(self.w, r#"    "{name}" [
        label=<
"#, name=e.name)?;

        self.open_tag_attrs("FONT", &[("FACE", e.header_options.font.clone())])?;
        write!(self.w, "\n  ")?;

        let mut attrs = Vec::new();
        attrs.push(("BORDER", e.header_options.border.to_string()));
        attrs.push(("CELLBORDER", e.header_options.cell_border.to_string()));
        attrs.push(("CELLPADDING", e.header_options.cell_padding.to_string()));
        attrs.push(("CELLSPACING", e.header_options.cell_spacing.to_string()));

        if let Some(c) = &e.options.background_color {
            attrs.push(("BGCOLOR", c.clone()))
        }
        self.open_tag_attrs("TABLE", &attrs)?;

        write!(
            self.w,
            "\n    <TR><TD><B><FONT POINT-SIZE=\"{size}\">{name}</FONT></B></TD></TR>\n",
            size=e.header_options.size,
            name=e.name,
        )?;

        for a in &e.attribs {
            self.render_attribute(a)?;
        }

        write!(self.w, r#"  </TABLE>
</FONT>
>];
"#)?;

        Ok(())
    }

    fn graph_attributes(&mut self, opts: &Vec<(&str, String)>) -> Result<()> {
        self.attributes("graph", opts)
    }

    fn node_attributes(&mut self, opts: &Vec<(&str, String)>) -> Result<()> {
        self.attributes("node", opts)
    }

    fn edge_attributes(&mut self, opts: &Vec<(&str, String)>) -> Result<()> {
        self.attributes("edge", opts)
    }

    fn attributes(&mut self, name: &str, opts: &Vec<(&str, String)>) -> Result<()> {
        write!(self.w, "    {} [\n", name)?;
        for (key, value) in opts {
            write!(self.w, "        {}={},\n", key, value)?;
        }
        write!(self.w, "    ];\n")
    }

    fn graph_footer(&mut self) -> Result<()> {
        write!(self.w, "}}\n")
    }


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
        let mut renderer = Renderer::new(&mut buf);
        renderer.render_erd(&erd).unwrap();
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
        let mut renderer = Renderer::new(&mut buf);
        renderer.render_erd(&erd).unwrap();
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
        let mut renderer = Renderer::new(&mut buf);
        renderer.render_erd(&erd).unwrap();
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
        let mut renderer = Renderer::new(&mut buf);
        renderer.graph_header().unwrap();
        renderer.graph_attributes(
            &vec![
                ("a", "b".to_owned()),
                ("c", "\"d\"".to_owned())
            ],
        ).unwrap();
        renderer.graph_footer().unwrap();
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