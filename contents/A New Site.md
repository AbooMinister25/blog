---
title = "A New Site"
tags = ["programming", "rust"]
summary = "I give an overview of my site and how I built it." 
---


I've gone and written a website for myself. In Rust! All with my very own static site generator with a few neat features. Before we get into that, however, I'd like to take this chance to introduce myself. I'm Rayyan Cyclegar, a high schooler and programming enthusiast. I even have a [GitHub](https://github.com/AbooMinister25) profile, complete with my projects and contributions.

Now, back to the blog. I've often found that writing is a nice way for me to get my thoughts out there. Whether it be to share knowledge, talk about something cool, or to simply rant into the void that is the internet. It took me a good few weeks to hack this thing together, so I'll take the pleasure of walking you all through it!

## The Gist of it
So there are tons of preexisting solutions I could've used. Static site generators such as Zola and Hugo are more than capable for my needs. But if I had gone with those, this process wouldn't have been nearly as fun. The gist of the process is that it takes all my markdown, stored in a `contents/` folder, all my stylesheets (SASS) in a `sass/` folder, and my assets inside an `assets/` folder. It then compiles what needs to be compiled, minifies assets, renders everything using templates, and chucks the generated HTML and stylesheets into their respective `public/` and `styles/`. All with support for incremental building!

![hi](/assets/markdown_flowchart.excalidraw.svg)

## The Markdown Side of Things
The core of a static site generator is *markdown*. A markup language used for writing formatted text. Now, I did mention I wrote this in Rust, which has two notable crates for parsing markdown:
- pulldown-cmark
- comrak

I decided to go with [comrak](https://github.com/kivikakk/comrak). From a glance, it seems easy enough to use, and has complete support for CommonMark and all five of GitHub Flavored Markdown (Shortened to GFM) specific extensions. Built in support for [syntect](https://github.com/trishume/syntect), the crate I used for syntax highlighting, also helped.

My entire process for handling markdown consists of walking the content directory, reading in all the files one by one, hashed the files, which I'll go more into later, and rendered everything using templates. I also wrote any data I wanted to persist into an SQLite database.

Why SQLite? Well, my database needs for this aren't particularly extravagant, and I didn't want to go through the trouble of setting up diesel and migrations. I'm not going to be writing to it all the time, either.

Ah right, templates. I had a few options for this:
- A rust port of handlebars
- Tera
- Liquid
- Maud
- Among others…

I eventually settled on [Tera](https://tera.netlify.app/), for no particular reason other than it seemed straightforward enough to set up, and had everything I really cared about.

A good static site generator will typically parse some form of *frontmatter* from your markdown files. Frontmatter is some additional information about each page of Markdown at the top of your file, it can look something like this.

```markdown
---
title = "A New Site"
tags = ["programming"]
summary = "I give an overview of my site and how I built it." 
---
```

I used rust's [toml](https://docs.rs/toml/latest/toml/) crate for parsing the frontmatter.

## Stylesheets
For styling everything, I went with [SASS](https://sass-lang.com/), since it's what all the cool kids are using.