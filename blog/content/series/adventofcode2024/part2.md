---
title = "Advent of Code Day 2"
tags = ["programming", "adventofcode"]
date = "2024-12-09T5:00:00"
slug = "day2"

[series]
part=2
---

Day 2 was pretty straightforward, and there isn't all that much I want to say about it, so I'll get straight to the problem.

## Part 1

The puzzle gives us an input that consists of rows of _reports_, each of which is made up of a list of _levels_, which are just numbers.

```
7 6 4 2 1
1 2 7 8 9
9 7 6 2 1
1 3 2 4 5
8 6 4 4 1
1 3 6 7 9
```

In the example input they give us, there are 6 reports with 5 levels each. The first report is `7 6 4 2 1`, and the rest follow.

We need to figure out how many reports out of those given to us are _safe_, based on two rules. A report is safe if all the levels are all increasing or all decreasing, or if two levels change by at least one but no more than three. So, we can have jumps from `4` to `2`, or `5` to `6`, but not `1` to `5` or no change at all.

After parsing the input, first thing I did was figure out how to calculate the differences given a report.

```py
differences = (abs(a-b) for a, b in itertools.pairwise(report))
```

`itertools.pairwise` pairs a given iterable as follows:

```py
>>> import itertools
>>> list(itertools.pairwise((1, 2, 3, 4)))
[(1, 2), (2, 3), (3, 4)]
```

I can then take the differences and check that `0 < difference < 4` for each difference.

For checking whether the reports are all increasing or all decreasing, I initially removed the `abs` from `differences` and checked whether all of the differences were of the same sign. If they're all positive, the reports are all increasing, and if they're all negative, the reports are all decreasing.

```py
all(i > 0 for i in differences) or all(i < 0 for i in differences)
```

I then realized that, hey, if everything in a report is increasing, then that means it's sorted, doesn't it? And if everything is decreasing, it's sorted in descending order. Instead of checking the signs of the differences, I could just do

```py
report == sorted(report) or report == sorted(report, reverse=True)
```

for each report. I opted for this in the end, but it wasn't _really_ like I felt either option was more or less ergonomic.

My final solution for part 1, then, was:

```py
import itertools

def is_safe(report: list[int]) -> bool:
    differences = (abs(a - b) for a, b in itertools.pairwise(report))
    return all(0 < i < 4 for i in differences) and (
        report == sorted(report) or report == sorted(report, reverse=True)
    )


def part_1(inp: str) -> int:
    reports = parse_input(inp)
    return sum(is_safe(report) for report in reports)
```

## Part 2

For part 2, it turns out that a report can still be considered safe if that same report with any one element removed would make it safe. For example, for the report `1 3 2 4 5`, it's unsafe since the change from `3` to `2` is a decrease, whereas the initial change from `1` to `3` is an increase. Under the new rules of Part 2, however, because we can remove a single level, this report is safe if we remove the `3`.

So, I need every combination of a given report with one element removed, which `itertools.combinations` solved for me.

```py
>>> import itertools
>>> report = (1, 2, 3, 4, 5)
>>> list(itertools.combinations(report, 4))
[(1, 2, 3, 4), (1, 2, 3, 5), (1, 2, 4, 5), (1, 3, 4, 5), (2, 3, 4, 5)]
```

and so my final solution for part 2 was

```py
def part_2(inp: str) -> int:
    reports = parse_input(inp)
    return sum(
        is_safe(report)
        or any(is_safe(list(c)) for c in itertools.combinations(report, len(report) - 1))
        for report in reports
    )
```

## Conclusion

Here's my entire solution for the puzzle:

```py
import functools
import itertools

from aoc_lube import fetch, submit  # type: ignore


def parse_input(inp: str) -> list[list[int]]:
    return [[int(n) for n in lst.split()] for lst in inp.splitlines()]


def is_safe(report: list[int]) -> bool:
    differences = (abs(a - b) for a, b in itertools.pairwise(report))
    return all(0 < i < 4 for i in differences) and (
        report == sorted(report) or report == sorted(report, reverse=True)
    )


def part_1(inp: str) -> int:
    reports = parse_input(inp)
    return sum(is_safe(report) for report in reports)


def part_2(inp: str) -> int:
    reports = parse_input(inp)
    return sum(
        is_safe(report)
        or any(is_safe(list(c)) for c in itertools.combinations(report, len(report) - 1))
        for report in reports
    )


if __name__ == "__main__":
    raw_input = fetch(2024, 2)
    submit(2024, 2, 1, functools.partial(part_1, raw_input))
    submit(2024, 2, 2, functools.partial(part_2, raw_input))
```
