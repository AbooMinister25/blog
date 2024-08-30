---
title = "FastAPI Logging with Structlog"
tags = ["programming", "python", "logging"]
date = "2024-07-25T6:10:00"
draft = true
---

I'm a developer over at [Vipyr Security](https://vipyrsec.com/), where we run a malware scanning service for analyzing and reporting malicious packages on the [Python Package Index](https://pypi.org/). There's a handful of services that are used to facilitate this, all of which are tied together by ["mainframe"](https://github.com/vipyrsec/dragonfly-mainframe), a web server written with Python and [FastAPI](https://fastapi.tiangolo.com/).

I'll use this post to briefly discuss how we've set up logging in our codebases, most significantly in mainframe.

## Structured Logging

We decided to opt for a logging solution that took advantage of _structured logging_ - instead of log output being difficult to parse, structured logging ensures that logs are formatted as events in key-value pairs. The go-to solution in Python for this is provided via [structlog](https://www.structlog.org/en/stable/index.html), which purports itself as a fast, simple, and powerful production ready logging solution.

With `structlog`, all log entries are essentially dictionaries, and can be manipulated as such.

```py
# From the structlog documentation
>>> from structlog import get_logger
>>> log = get_logger()
>>> log.info("key_value_logging", out_of_the_box=True, effort=0)
2020-11-18 09:17:09 [info     ] key_value_logging    effort=0 out_of_the_box=True

>>> log = log.bind(user="anonymous", some_key=23)
>>> log = log.bind(user="hynek", another_key=42)
>>> log.info("user.logged_in", happy=True)
2020-11-18 09:18:28 [info     ] user.logged_in    another_key=42 happy=True some_key=23 user=hynek
```

`structlog` also enables a [processor pipeline](https://www.structlog.org/en/stable/processors.html) which operate on the event dictionary for each log entry. The format in which log entries are emitted is based on the formatters that are used, which include a console renderer for development, and a JSON formatter for production.

## Our Setup

`structlog` is useable as a drop-in replacement for the standard library's `logging` library if you use it to wrap the stdlib logger.

In our setup, we have log entries from both the standard library `logging` and `structlog` be passed to the standard library to be formatted, but use `structlog`'s `structlog.stdlib.ProcessorFormatter` as the `logging.Formatter` which is used. This ![diagram](https://www.structlog.org/en/stable/standard-library.html#rendering-using-structlog-based-formatters-within-logging) in the structlog documentation illustrates this setup.

What this allows us to do is use `logging`s `dictConfig` for configuration alongside structlog, enabling more sophisticated logging configurations.
