# erd-rs

Rust CLI tool for creating entity-relationship diagrams from plain text markup.
Based on [erd](https://github.com/BurntSushi/erd) (uses the same input format
and output rendering).

Entities, relationships and attributes are defined in a simple plain text
markup format, which is used to generate an entity-relationship diagram in
[DOT](https://en.wikipedia.org/wiki/DOT_(graph_description_language)) format.

This can then be rendered into e.g. pdf, png, svg, etc. using
[Graphviz](https://graphviz.org/) or another similar tool.

## Status

Currently under development, general parsing and mostly default output is
completed, but not yet feature complete.

Approximate TODO:

* default options
* global options (parsing is implemented, but rendering is not)
* checks that entities exist when parsing relationships
* code cleanup/refactoring
* internal function/parser documentation
* user guide/overview
* additional error handling
* additional unit tests
* add github actions to run tests
* add action to build/push docker image to docker hub
