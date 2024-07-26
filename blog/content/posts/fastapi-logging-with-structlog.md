---
title = "FastAPI Logging with Structlog"
tags = ["programming", "python", "logging"]
date = "2024-07-25T6:10:00"
---

I'm a developer over at [Vipyr Security](https://vipyrsec.com/), where we run a malware scanning service for analyzing and reporting malicious packages on the [Python Package Index](https://pypi.org/). There's a handful of services that are used to facilitate this, all of which are tied together by ["mainframe"](https://github.com/vipyrsec/dragonfly-mainframe), a web server written with Python and [FastAPI](https://fastapi.tiangolo.com/).

Logging is important, it facilitates observability and aids debugging, and so the need for a sufficient solution for logging made itself clear. From the beginning, there were a few things that I wanted.

- Structured logging