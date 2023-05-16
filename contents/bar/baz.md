---
title = "baz"
tags = ["programming", "test"]
series = "Hello World Series"
---

test post
hi this is a test post. It consists of _both_ tests and `tests`. Can't believe it, right?
I didn't believe it at first myself. Also - don't mention it to anyone, but theres also a

`println!("Hello World")`

_italic_   
  
**bold**

**_bolt_italic_** 

-   yeah this is a thing
-   b 
-   c 

## Header 1

Yeah, this has some code. Deal with it.
test-lang.org">rust's website</a>
</div>

```rust
fn main() {
    // Generate lorem ipsum text with Title Case.
    let title = lipsum::lipsum_title();
    // Print underlined title and lorem ipsum text.
    println!("{}\n{}\n", title, str::repeat("=", title.len()));

    // First command line argument or "" if not supplied.
    let arg = std::env::args().nth(1).unwrap_or_default();
    // Number of words to generate.
    let n = arg.parse().unwrap_or(25);
    // Print n words of lorem ipsum text.
    println!("{}", lipsum::lipsum(n));  
} 
```

## Header 2

a

## Header 3

b

## Header 4

c

[this is a link](https://pydis.org)  fd