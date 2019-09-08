# md2tex
[![crates badge][crates-badge]][crates.io]
[![docs badge][docs-badge]][docs]
[![ci badge][ci-badge]][ci]

[crates.io]: https://crates.io/crates/md2tex
[crates-badge]: https://img.shields.io/badge/crates.io-v0.1.3-orange.svg?longCache=true

[docs]: https://docs.rs/crate/md2tex/0.1.3
[docs-badge]: https://docs.rs/md2tex/badge.svg

[ci]: https://travis-ci.org/lbeckman314/md2tex
[ci-badge]: https://api.travis-ci.org/lbeckman314/md2tex.svg?branch=master

A small utility to convert markdown files to tex. Forked from [md2pdf](https://gitea.tforgione.fr/tforgione/md2pdf/), with an added focus on mdbook conversions. Also with the goal of eventually contributing back upstream.

Used by [mdbook-latex](https://github.com/lbeckman314/mdbook-latex) to generate PDF's.

## Usage

```sh
md2tex -i input.md -o output.tex
```
