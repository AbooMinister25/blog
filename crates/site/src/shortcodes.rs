use std::collections::HashMap;

use color_eyre::Result;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{alpha1, alphanumeric1, digit1, multispace0},
    combinator::{map, map_res, opt, recognize},
    error::ParseError,
    multi::{many0, many0_count, separated_list0},
    sequence::{delimited, pair, preceded},
    IResult, Parser,
};
use serde::Serialize;
use tera::Context as TeraContext;

use crate::context::Context;

#[derive(Debug, PartialEq)]
pub enum Item {
    Text(String),
    Shortcode(Shortcode),
}

#[derive(Debug, PartialEq)]
pub struct Shortcode {
    pub name: String,
    pub arguments: HashMap<String, Value>,
    pub body: String,
}

#[derive(Debug, PartialEq, Serialize)]
pub enum Value {
    Bool(bool),
    Number(i32),
    String(String),
    List(Vec<Value>),
}

#[tracing::instrument(level = tracing::Level::DEBUG)]
pub fn evaluate_shortcode(ctx: &Context, shortcode: Shortcode) -> Result<String> {
    let mut context = TeraContext::from_serialize(shortcode.arguments)?;
    context.insert("body", &shortcode.body);

    let rendered = ctx.tera.render(&shortcode.name, &context)?;
    Ok(rendered)
}

#[tracing::instrument(level = tracing::Level::DEBUG)]
pub fn parse(input: &str) -> IResult<&str, Vec<Item>> {
    let (input, mut items) = many0(alt((
        map(shortcode, Item::Shortcode),
        map(text, Item::Text),
    )))(input)?;

    items.push(Item::Text(input.to_string()));

    Ok((input, items))
}

fn text(input: &str) -> IResult<&str, String> {
    let (input, text) = take_until("{{!")(input)?;
    Ok((input, text.to_string()))
}

fn shortcode(input: &str) -> IResult<&str, Shortcode> {
    let (input, (name, arguments)) =
        ws(delimited(tag("{{!"), ws(shortcode_start), tag("!}}")))(input)?;
    let (input, body) = take_until("{{!")(input)?;
    let (input, _) = delimited(tag("{{!"), ws(tag("end")), tag("!}}"))(input)?;

    Ok((
        input,
        Shortcode {
            name,
            arguments,
            body: body.to_string(),
        },
    ))
}

fn shortcode_start(input: &str) -> IResult<&str, (String, HashMap<String, Value>)> {
    let (input, function_name) = ws(recognize(pair(
        alt((alpha1, tag("_"))),
        many0_count(alt((alphanumeric1, tag("_")))),
    )))(input)?;
    let (input, arguments) = opt(ws(delimited(
        tag("("),
        separated_list0(tag(","), ws(argument)),
        tag(")"),
    )))(input)?;

    Ok((
        input,
        (
            function_name.to_string(),
            arguments.unwrap_or(Vec::new()).into_iter().collect(),
        ),
    ))
}

fn argument(input: &str) -> IResult<&str, (String, Value)> {
    let (input, name) = recognize(pair(
        alt((alpha1, tag("_"))),
        many0_count(alt((alphanumeric1, tag("_")))),
    ))
    .parse(input)?;
    let (input, _) = ws(tag("="))(input)?;
    let (input, value) = ws(value)(input)?;

    Ok((input, (name.to_string(), value)))
}

fn value(input: &str) -> IResult<&str, Value> {
    let boolean = alt((
        map(tag("true"), |_| Value::Bool(true)),
        map(tag("false"), |_| Value::Bool(false)),
    ));
    let number = alt((
        map_res(digit1, |digit_str: &str| {
            digit_str.parse::<i32>().map(Value::Number)
        }),
        map(preceded(tag("-"), digit1), |digit_str: &str| {
            Value::Number(-digit_str.parse::<i32>().unwrap())
        }),
    ));
    let string = map(
        delimited(
            tag::<&str, &str, nom::error::Error<_>>("\""),
            take_until("\""),
            tag("\""),
        ),
        |s: &str| Value::String(s.to_string()),
    );
    let list = map(
        delimited(tag("["), separated_list0(tag(","), ws(value)), tag("]")),
        Value::List,
    );

    alt((boolean, number, string, list))(input)
}

fn ws<'a, O, E: ParseError<&'a str>, F>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: Parser<&'a str, O, E>,
{
    delimited(multispace0, inner, multispace0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_content() {
        let test_input = "# Hello World

this is a thing

## this is another thing

**hi**

{{! test(a=1, b=2) !}}
hello world
{{! end !}}

**more**";

        let items = parse(test_input).unwrap().1;
        assert_eq!(
            items,
            vec![
                Item::Text(
                    "# Hello World\n\nthis is a thing\n\n## this is another thing\n\n**hi**\n\n"
                        .to_string()
                ),
                Item::Shortcode(Shortcode {
                    name: "test".to_string(),
                    arguments: vec![
                        ("a".to_string(), Value::Number(1)),
                        ("b".to_string(), Value::Number(2))
                    ]
                    .into_iter()
                    .collect(),
                    body: "hello world\n".to_string()
                }),
                Item::Text("\n\n**more**".to_string())
            ]
        );
    }

    #[test]
    fn test_parse_no_shortcodes() {
        let test_input = "# Hello World

this is a thing

## this is another thing

**hi**

**more**";

        let items = parse(test_input).unwrap().1;
        assert_eq!(
            items,
            vec![
                Item::Text(
                    "# Hello World\n\nthis is a thing\n\n## this is another thing\n\n**hi**\n\n**more**"
                        .to_string()
                ),
            ]
        );
    }

    #[test]
    fn test_parse_only_shortcode() {
        let test_input = "{{! test(a=1, b=2) !}}
hello world
{{! end !}}";

        let items = parse(test_input).unwrap().1;
        assert_eq!(
            items,
            vec![
                Item::Shortcode(Shortcode {
                    name: "test".to_string(),
                    arguments: vec![
                        ("a".to_string(), Value::Number(1)),
                        ("b".to_string(), Value::Number(2))
                    ]
                    .into_iter()
                    .collect(),
                    body: "hello world\n".to_string()
                }),
                Item::Text(String::new())
            ]
        );
    }

    #[test]
    fn parse_empty() {
        let test_input = "";

        let items = parse(test_input).unwrap().1;
        assert_eq!(items, vec![Item::Text(String::new())]);
    }

    #[test]
    fn test_parse_shortcode_arguments() {
        let test_input = "
{{! test(a=1, b=2) !}}
hello world
{{! end !}}
        ";

        let shortcode = shortcode(test_input).unwrap().1;

        assert_eq!(
            shortcode,
            Shortcode {
                name: "test".to_string(),
                arguments: vec![
                    ("a".to_string(), Value::Number(1)),
                    ("b".to_string(), Value::Number(2))
                ]
                .into_iter()
                .collect(),
                body: "hello world\n".to_string()
            }
        );
    }

    #[test]
    fn test_parse_shortcode_no_arguments() {
        let test_input = r"
{{! test !}}
hello world
{{! end !}}
        ";

        let shortcode = shortcode(test_input).unwrap().1;

        assert_eq!(
            shortcode,
            Shortcode {
                name: "test".to_string(),
                arguments: HashMap::new(),
                body: "hello world\n".to_string()
            }
        );
    }
}
