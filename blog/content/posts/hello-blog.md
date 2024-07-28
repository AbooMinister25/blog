---
title = "Writing a Static Site Generator"
tags = ["introduction"]
date = "2024-07-17T6:00:00"
---

It took two years and several rewrites, but since you're seeing this, it means I've (finally) gotten my static site generator to a stage in which it's feasibly working, and have deployed my website to the web.

I've decided to start blogging, mostly content related to tech/programming, and occasionally content which differs from that. Although really the whole writing blog posts thing was an excuse to try my hand at writing a static site generator, which ended up taking longer than I had intended, mostly due to the large amount of rewrites and my wavering attention and motivation to continue hacking on it the past couple of years, as well as conflicting priorities.

It does work fairly well, though, and it's somewhat fast, so I'll give myself that much credit. I figured it'd be more fun to try my hand at rolling my own rather than using something like [Hugo](https://gohugo.io/) or [Zola](https://getzola.org). I decided to use this first post to provide some overview into what went into this thing.

## Building a static site generator

My requirements for a static site generator weren't particularly extravagant, they really only consisted of the following:

-   Taxonomies.
-   Full text search.
-   The ability to write ongoing "series" of posts.
-   Being fast enough that writing posts and having the site build on changes would be seemingly instant.
-   Support for [excalidraw](https://excalidraw.com/) diagrams (more on this later).
-   SCSS/SASS stylesheets.

Something that would fit these requirements seemed simple enough to implement, so I decided to go for it. I reached for [Rust](https://www.rust-lang.org/) as my language of choice to write this in.

So at it's core, what a static site generator does is it takes a set of input files (markdown, some form of stylesheet, etc), processes them (parsing the markdown and emitting HTML, compiling the stylesheets, formatting everything with templates, and so on), and writes all this to disk. However, most mainstream static site generators tend to regenerate the entire site every time you build, but with a blog, most of your changes are really only going to be centered around whatever post you're writing/editing at the time. As a result, it'd be convenient if _only_ the content that has changed since the last build of a site were rebuilt.

{{! note !}}
This said, plenty of static site generators are fast enough that, especially with smaller sites, build times aren't really going to be a problem when iterating on some content.
But that's not much fun, is it.
{{! end !}}

I decided to try my hand at implementing incremental builds. Taking this alongside my list of requirements, I got a basic workflow for what I wanted this to look like:

-   Read in all the input files and hash them.
-   Compare these hashes to any existing hashes that were stored from previous runs of the static site generator, and decide what needs to be built.
-   Process all of the entries that need to be built.
-   Format the posts with templates and write those to disk.
-   Write the remaining entries to disk.
-   Do any post-processing/generate things like an atom feed.

I'll start at the top.

## Discovering Entries

The starting point for the static site generator is to discover all the input files that need to be built, presumably from a root directory that will be crawled, and stick them all into an array. For incremental builds, however, there's an extra step involved: it'll need to determine whether or not a file has changed since the last run of the static site generator. I decided to do this with hashes, hash all the files that were read in, and compare them to hashes that were persisted for the same files from a previous run of the static site generator. If the hash had changed, the entry was updated. If there isn't an existing hash for the file, it means that it's a new entry, and also needs to be built.

{{! note !}}
I added some exceptions for this, however; there's a set of files, mostly "index" files, which will always be built, regardless of whether or not they've changed since the last run. This is because the home page, for example, won't be re-built to include any new or updated posts since a previous run, since technically the hash for its file hadn't changed.

I considered an alternative to this, in which I'd represent all the entries of the static site generator in a tree-like structure, and then use that to determine which entries were linked to another, and decide what to rebuild based on that, but that's more complexity than I'd like for now. It stands as something I might play with implementing later.
{{! end !}}

For persisting this hash data in between runs, I opted to use an [SQLite](https://www.sqlite.org/) database. I don't need some full fledged database server, and SQLite only needing a single file made things simpler. I also persist other things in between runs, but I'll get into that later.

What I got, then, was having the static site generator recursively traverse the files in the provided root of the site using the [ignore](https://crates.io/crates/ignore) crate, compare the hashes of all the files, and put everything that needed to be built into the following `Entry` struct:

```rust
#[derive(Debug)]
pub struct Entry {
    pub path: PathBuf,
    pub raw_content: Vec<u8>,
    pub hash: String,
    pub new: bool,
}
```

Now, `ignore` is nice because it respects ignore globs found in `.gitignore` and `.ignore` files. The directory structure for my blog looks like

```
assets/
    images/
    js/
    styles/
content/
    posts/
    series/
    index.md
    ...
static/
    fonts/
    icons/
templates/
themes/
.ignore
```

Now, the contents of `templates/` and `themes/` do not need to be processed by the static site generator, so my `.ignore` contains the following:

```
themes/
templates/
```

## Markdown

The next step is to take the discovered `Vec` of `Entry`s and process them. How a given entry is processed is determined by the directory in which it is placed; if an entry is in the `content/` directory, it'll be processed as a markdown post, if it's in the `assets/` directory, it'll be processed as an asset, and so on. I'll start by discussing markdown.

I use Rust's [comrak](https://lib.rs/crates/comrak) crate, "A 100% CommonMark-compatible GitHub Flavored Markdown parser and formatter", to handle parsing all my markdown. Now, a markdown file for a post in my blog consists of the frontmatter, metadata declared at the beginning of the document, and the rest of the content. I opted to use TOML as the format of choice for my frontmatter, and as such a markdown file for a post in my blog looks something like the following:

```markdown
---
title = "Hello, Blog"
tags = ["introduction"]
date = "2024-07-15T3:39:00"
draft=true
---

# Blah

Blah blah blah
```

`comrak` lets me define what delimiter I use for my frontmatter (in this case, `---`), so that it knows where to start parsing the markdown from. Aside from this, it's pretty trivial to just read in all the frontmatter into a string, use Rust's [toml](https://lib.rs/crates/toml) crate to parse that, and deserialize it into a struct.

```rust
fn parse_frontmatter(&self, content: &str) -> Result<Frontmatter> {
    let mut opening_delim = false;
    let mut frontmatter_content = String::new();

    for line in content.lines() {
        if line.trim() == "---" {
            if opening_delim {
                break;
            }

            opening_delim = true;
            continue;
        }

        frontmatter_content.push_str(line);
        frontmatter_content.push('\n');
    }

    let frontmatter = toml::from_str(&frontmatter_content)?;
    Ok(frontmatter)
}
```

I also want to automatically generate a table of contents for each post, compiled of all the (second) headings in the document. `comrak` makes this easy, since it generates an AST I can work with.

```rust
// Adapted from https://github.com/kivikakk/comrak/blob/main/examples/headers.rs
fn parse_toc<'a>(&self, root: &'a AstNode<'a>) -> Vec<String> {
    let mut toc_headers = Vec::new();

    for node in root.children() {
        let NodeValue::Heading(header) = node.data.clone().into_inner().value else {
            continue;
        };

        if header.level != 2 {
            continue;
        }

        let mut text = Vec::new();
        collect_text(node, &mut text);

        // Safe to unwrap, input good UTF-8, comrak guarantees output is good UTF-8.
        toc_headers.push(String::from_utf8(text).unwrap());
    }

    toc_headers
}
```

A markdown document is parsed and formatted, and then turned into the following struct:

```rust
pub struct Document {
    pub date: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub content: String,
    pub frontmatter: Frontmatter,
    pub toc: Vec<String>,
    pub summary: String,
}
```

I also wanted syntax highlighting for code blocks, which `comrak` has built in support for, leveraging the [syntect](https://lib.rs/crates/syntect) crate.

What happens then, is that an `Entry` will be converted into a `Document`, which is then in turn converted into the following `Page` struct.

```rust
#[derive(Debug, Serialize, Deserialize, Eq, Clone)]
pub struct Page {
    pub path: PathBuf,
    pub title: String,
    pub tags: Vec<String>,
    pub permalink: String,
    #[serde(rename = "body")]
    pub raw_content: Option<String>,
    pub content: Option<String>,
    pub frontmatter: Option<Frontmatter>,
    pub toc: Option<Vec<String>>,
    pub summary: String,
    pub hash: String,
    pub date: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    #[serde(skip)]
    pub new: bool,
    pub index: bool,
}
```

The final step in this process is to format the page with a template and write everything to disk. I opted to use the [tera](lib.rs/tera) crate for this, mostly because tera's template language is reminiscent of Python's Jinja2 which I was already familiar with.

## Static Files and Assets

Aside from markdown, content passed to my static site generator can take the form of either a static file, or an asset. In the context of my static site generator, a static file is something that is copied over to the output directly pretty much as-is, whereas an asset refers to a resource that is typically passed through an asset pipeline - this includes images, stylesheets, or JavaScript.

Handling static files was trivial, it was just an `std::fs::copy` from the source to the output location. Assets were a bit more involved, though. Depending on what the asset was, I wanted to perform some sort of processing on it. This was either preprocessing (SCSS stylesheets, for example) or postprocessing (embedding the fonts in an SVG). For handling my stylesheets, I had a few options.

-   [sass-rs](https://lib.rs/crates/sass-rs) is a rust wrapper around libsass, but it's currently unmaintained.
-   [sass-alt](https://lib.rs/crates/sass-alt) is another set of alternative bindings to libsass, which markets itself as a more powerful alternative to sass-rs. However, it was last updated 7 years ago.
-   [grass](https://lib.rs/crates/grass) is a pure rust implementation of SASS.
-   [rsass](https://lib.rs/crates/rsass) is another pure rust implementation of SASS, but it doesn't yet support the indentation syntax.

I opted to use the [rsass](https://lib.rs/crates/rsass) crate; although incomplete, It's good enough for my use case here. For JavaScript scripts, the preprocessing consisted of invoking a bundler that would bundle the script and its dependencies.

So now I needed to choose a bundler. I didn't have many requirements, but I did want something fast, and it turns out [esbuild](https://esbuild.github.io/) is about as fast as it gets. It's written in Go, but I found [esbuild-rs](https://lib.rs/crates/esbuild-rs) which wraps Go API that esbuild exposes. The downside of this is that my Rust static site generator now requires Go to build, which is somewhat cursed.

{{! note !}}
A Rust alternative to esbuild _does_ exist: [swc](https://swc.rs) is pretty fast, and it ships with a bundler, all of which I could invoke via the Rust API that it exposes. However the bundler for swc (swcpack) isn't something that will be actively developed as per the docs, and I didn't discover the Rust API until after I'd already gotten everything working with esbuild.
{{! end !}}

This is all the preprocessing I do for now, but I do intend on expanding this at some point. The next step would be to have a better preprocessing pipeline for images, converting them to preferred formats and such. But everything else as of now is just copied over to the output location as-is.

```rust
fn preprocess_and_write<T: AsRef<Path> + Debug>(
    &self,
    out_dir: T,
    filename: &str,
) -> Result<PathBuf> {
    Ok(match self.path.extension().and_then(OsStr::to_str) {
        Some("scss") => {
            let out_path = out_dir.as_ref().join(format!("{filename}.css"));

            let format = output::Format {
                style: output::Style::Compressed,
                ..Default::default()
            };

            let css = compile_scss_path(&self.path, format)?;
            fs::write(&out_path, css)?;

            out_path
        }
        Some(ext @ "js") => {
            let out_path = out_dir.as_ref().join(format!("{filename}.{ext}"));
            bundle_js(&self.path, &out_path)?;

            out_path
        }
        Some(ext) => {
            let out_path = out_dir.as_ref().join(format!("{filename}.{ext}"));
            fs::copy(&self.path, &out_path)?;

            out_path
        }
        None => {
            let out_path = out_dir.as_ref().join(filename);
            fs::copy(&self.path, &out_path)?;

            out_path
        }
    })
}
```

I mentioned support for [excalidraw](https://excalidraw.com/) diagrams as something that I wanted in a static site generator, and I'll elaborate on that here. Excalidraw is a tool that you can use to create diagrams and flowcharts which have a handwritten feel to them, and you can export them from the web application as SVGs. The issues begin, however, when you begin to use these SVGs in your websites via an `<img>` tag: I found that my SVGs were defaulting to the system font, and weren't using the handwritten font that I was expecting. After a [bit of digging](https://stackoverflow.com/questions/12583879/linking-to-css-in-an-svg-embedded-by-an-img-tag/12585887#12585887) into the issue, it turns out that images need to be standalone files, and whatever SVGs you use as images can't use files that are being requested from an external resource. I narrowed down two possible solutions.

-   Turn all the text into paths using something like `inkscape`.
-   Embed the fonts as base64 using a data uri.

The first solution didn't seem very wieldy, I didn't want to depend on `inkscape` as part of my workflow, so I decided to opt for embedding the fonts. To embed the fonts, I needed to parse the SVG file, find the URLs in the file, fetch their contents, and encode them with base64 before substituting the URL with a data uri. I opted to use the [xmltree](https://lib.rs/crates/xmltree) crate to parse the SVG files, and [base64](https://lib.rs/crates/base64) to encode the fonts.

```rust
pub fn embed_font<P: AsRef<Path> + Debug>(path: P) -> Result<()> {
    let mut file = File::open(&path)?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let mut root = Element::parse(contents.as_bytes())?;

    let style = root
        .get_mut_child("defs")
        .and_then(|d| d.get_mut_child("style"));

    if let Some(s) = style {
        let text = s.get_text();

        if let Some(t) = text {
            if let Some(mat) = URL_RE.find(&t) {
                let url = mat.as_str();
                let font = CACHE.lock().unwrap().fetch_font(url)?;
                let replacement = format!(
                    "
                @font-face {{
                    font-family: \"Virgil\";
                    src: url(data:font/font;base64,{font});
                }}
                "
                );

                s.children.push(XMLNode::Text(replacement));
            }
        }
    }

    root.write(File::create(path)?)?;

    Ok(())
}
```

and `Cacher`

```rust
#[derive(Debug)]
struct Cacher {
    cache: HashMap<String, String>,
}

impl Cacher {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub fn fetch_font(&mut self, url: &str) -> Result<String> {
        if let Some(content) = self.cache.get(url) {
            Ok(content.to_string())
        } else {
            let resp = reqwest::blocking::get(url)?.bytes()?;
            let encoded = BASE64_STANDARD.encode(resp);

            self.cache.insert(url.to_string(), encoded);
            Ok(self.cache.get(url).unwrap().to_string())
        }
    }
}
```

I do all the font embedding _after_ the asset has been copied over to the output directory, so it acts as a form of postprocessing.

## Handling Indexes

So at this point I've got a workings static site generator with incremental builds. There were a few problems with my approach, though.

To start with, aside from the initial clean build, my static site generator never has the full picture of what the site looks like, since I only need to process whatever entries have been modified/newly created since the last run. This is somewhat problematic, since it prevents me from generating things which require an index of the contents of the site (including, but not limited to the index page, which has a listing of all the posts in the site, the atom feed, and the search index).

Now, initially my main concern was with the search index, since I hadn't gotten to implementing the generation of the index pages yet. I generate a search engine at an `index.json` file at the site's output directory, which is then passed to the full text search implementation I'm using. In the end, I decided to, just like the rest of my site, update the index incrementally, rather than regenerating it every time the site is built. And it seemed like a good use-case for sets.

What the static site generator does, then, is it checks whether or not there's an existing `index.json` file in the output directory, and if not, creates one and dumps all of the posts it's processing. If the file does exist, however, then it means that the current build isn't a clean build, and the existing index will need to be updated. I read in the JSON from the index, compare a `HashSet` of posts that the static site generator is currently working with to a `HashSet` of the posts in the generated index, and update/insert everything that has either changed or is new.

```rust
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Index {
    pub working_pages: HashSet<Page>,
}

impl Index {
    pub fn build_index(&self, root: &Path) -> Result<()> {
        let path = root.join("index.json");
        let serialized = if path.exists() {
            let content = fs::read_to_string(&path)?;
            let mut old_index = Self {
                working_pages: serde_json::from_str(&content)?,
            };

            for page in &self.working_pages {
                old_index.working_pages.replace(page.clone());
            }

            serde_json::to_string(&old_index.working_pages)?
        } else {
            serde_json::to_string(&self.working_pages)?
        };

        fs::write(&path, serialized)?;

        Ok(())
    }

```

{{! note !}}
Rather than using `HashSet::replace`, I could have also used a `HashMap` with `None` values and the `Entry` API, which may have been more ergonomic.

Of course, it would have been even better if `HashSet`s in Rust exposed something reminiscent of the `Entry` API themselves, but it's whatever.
{{! end !}}

Aside from search indexes, though, I also need to worry about persisting data when generating the Atom feed and the index pages. At this point, I figured that since I was already persisting the hashes and locations of input files in an SQLite database, I might as well use the database to persist the necessary information to generate these aforementioned indexes. For generating the search index I needed to also have access to the raw content of the file for the search engine to index, but I didn't want to store the contents of every file in the database, which is why I didn't use SQLite then. But for the Atom feed and the index pages, I don't need to store the content, which made storing it in the database more bearable.

What I ended up with is storing all the relevant metadata in a table in the database, loading it on every program run, and using it to generate whatever required it.

I'm not particularly happy with this current solution, though. It doesn't seem ideal to have what is essentially two indexes lying around, each of which has access to different amounts of the original information, and I would like to have a more streamlined way to persist the index in such a way that _everything_ which requires having access to it can do so from a single source. I'll leave it for now, but I do intend on revisiting it in the future.

## Development

With a (mostly) working static site generator in hand, there were a few more things which I wanted to add for the sake of a more streamlined development and writing process. Namely, this included having some sort of development server that would serve the generated site and hot reload on filesystem changes.

When running in development mode, my static site generator runs a small web server using [hyper](https://lib.rs/crates/hyper), and watches for filesystem changes using [notify](https://lib.rs/crates/notify). I also wanted live reload functionality, which essentially injects some code into your HTML pages to automatically reload a page as a result of something (like a filesystem change) in development. After some digging, I found [tower-livereload](https://lib.rs/crates/tower-livereload) which worked great.

## Final Thoughts

It took a handful more rewrites than I initally expected to get this thing up and running, and was a larger investment of time than I expected. That said, I am satisfied with the result, and while it's definitely still a work in progress, I am glad it works.

My next steps in regards to this will probably be (another) rewrite/substantial refactor - after a certain point, I became somewhat desperate to produce a working piece of software, and as a result the code is very messy, and a lot of it could use a sanity check. There's a lot more I want to do, some problems I want to iron out, and also dedicate some effort into making this static site generator more performant.

What slowed me down was the lack of a concrete idea of what exactly I wanted this to be - it wasn't originally a static site generator, and I went into programming it without much thought or planning, which delayed the process fairly significantly. But it _does_
work, and I did enjoy developing this.

That's all for now.

The entire codebase is on [GitHub](https://github.com/AbooMinister25/blog/).
