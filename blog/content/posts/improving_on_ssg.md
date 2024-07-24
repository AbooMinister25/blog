---
title = "Improving on my Static Site Generator"
tags = ["programming", "static site generator"]
date = "2024-07-21T9:00:00"
draft = true
---

The [first post](/posts/Writing-a-Static-Site-Generator) on this blog discussed how I wrote the static site generator which powers this blog. I'll use this post to discuss a set of ideas and improvements that I intend to implement in my static site generator, and how I might address some of the existing shortcomings.

As it is right now, my static site generator _works_, but it's woefully incomplete. It took me several rewrites and endless amounts of bikeshedding to get to the current product, and while I am rather proud of it, there's definitely more to be done. Enough that I've already started a [pseudo-rewrite/major refactor](https://github.com/AbooMinister25/blog/tree/refactor) of the codebase. A lot of the codebase remains unchanged, but everything has been restructured and given a sanity check, and I also fully intend to write actual tests this time. I'll list a few of my primary concerns:

-   I got _way_ too excited about splitting everything up into [cargo workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html). Splitting everything up in the fashion that I've done it doesn't make sense, and it only serves to make the codebase harder to work with. I suppose that speaks to the qualms of attempting to prematurely split up concerns in a codebase.

-   A lack of any testing makes it harder to verify behavior in the codebase. I don't want to rush through tests this time around, and writing bad tests isn't going to get me very far regardless.

blah
