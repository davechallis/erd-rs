# erd-rs

Rust CLI tool for creating entity-relationship diagrams from plain text markup.
Based on [erd](https://github.com/BurntSushi/erd) (uses the same input format
and output rendering).

Entities, relationships and attributes are defined in a simple plain text
markup format, which is used to generate an entity-relationship diagram in
[DOT](https://en.wikipedia.org/wiki/DOT_(graph_description_language)) format.

This can then be rendered into e.g. pdf, png, svg, etc. using
[Graphviz](https://graphviz.org/) or another similar tool.
