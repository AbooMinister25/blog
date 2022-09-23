# Blog

This repository hosts the source code for my personal blog, where I occasionally write about tech, and even less occasionally about non-tech.

## Details

This repository consists of two components, a static site generator which takes my blog posts (in the `contents/` folder) and emits static HTML to a `public/` folder. The other component is a web server, powered by Rust's `axum` web framework. This serves all my content.

## Building

TODO

Keep in mind, if you want a static site generator, this probably isn't for you. It is first and foremost a personal tool, with no guarantees of being regularly maintained (or having the best code and architecture, for that matter).
