---
title = "Advent of Code Day 1"
tags = ["programming", "adventofcode"]
date = "2024-12-01T4:00:00"
slug = "day1"

[series]
part=1
---

Work over the past week kept me busy, so I wasn't able to tackle any of the puzzles until this weekend. School and college applications have kept me busy, so the time I've spent programming has gradually dwindled. I had some time free up this winter, though, so AoC will serve as a nice way to get back into things.

I'll do most of my solutions in Python, but if I have the time I might attempt the languages from the Python Discord's _Language Roulette_ challenge, where a random language is selected from a predetermined pool every day to do the solutions in.

Many people use helper libraries for things like fetching inputs and submitting solutions, as well as implementing some common operations, which streamline the process of solving puzzles. I like salt-die's [aoc-lube](https://github.com/salt-die/aoc_lube), but I'll likely end up implementing a script of my own at some point.

## Part 1

Today's puzzle gave us an input that looks like the following

```
3   4
4   3
2   5
1   3
3   9
3   3
```

For the first part, we needed to pair up the smallest numbers of both lists together, find the distances between them, then pair up the second smallest numbers and find the distances between them, and so on, summing all of those distances for the final answer.

For the example input, this would be pairing `(1, 3)` with a distance of 2, then `(2, 3)` with a distance of 1, etc.

This seems easy enough, I just need to extract the `int`s from the input, split them into two columns, sort both to be in ascending order, and then use `zip` to pair them up.

```py
def part_1(inp: str) -> int:
    numbers = [int(i) for i in inp.split()]
    return sum([abs(a - b) for a, b in zip(sorted(numbers[::2]), sorted(numbers[1::2]))])
```

After the `int`s are extracted, `numbers` will look like `[3, 4, 4, 3, 2, 5, ...]`, with every element from the first column being _every other_ element of the list.

This lets me use slicing with a step of `2` to get every other element. `numbers[::2]` will get me the first column, and `numbers[1::2]` (every other element, but starting from the second item of the list) will get me the second column.

I can then pair them up with `zip` and sum the absolute value of differences between all the pairs.

## Part 2

In part 2, we needed to calculate what AoC calls a "similarity score" for the list, which we do by adding all the numbers in the first column after multiplying that number by the number of times it appears in the second column.

So, given

```
3   4
4   3
2   5
1   3
3   9
3   3
```

the first number `3` appears three times in the second column, so we multiply it by `3` to get `9`, and we do that for every element of the first column.

Setting up is identical to the last part. After splitting the input up into two columns, I can use `list.count` to count tne number of times an element from the first column appears in the second column, and use that as the factor I multiply by.

```py
def part_2(inp: str) -> int:
    numbers = [int(i) for i in inp.split()]
    second_col = numbers[1::2]

    return sum(i * second_col.count(i) for i in numbers[::2])
```

## Conclusion

That's a wrap, here's my entire solution.

```py
import functools
from aoc_lube import fetch, submit  # type: ignore


def part_1(inp: str) -> int:
    numbers = [int(i) for i in inp.split()]
    return sum([abs(a - b) for a, b in zip(sorted(numbers[::2]), sorted(numbers[1::2]))])


def part_2(inp: str) -> int:
    numbers = [int(i) for i in inp.split()]
    second_col = numbers[1::2]

    return sum(i * second_col.count(i) for i in numbers[::2])


if __name__ == "__main__":
    raw_input = fetch(2024, 1)
    submit(2024, 1, 1, functools.partial(part_1, raw_input))
    submit(2024, 1, 2, functools.partial(part_2, raw_input))
```
