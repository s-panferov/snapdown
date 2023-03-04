use std::cell::Cell;
use std::fmt::{Debug, Display};
use std::path::Path;

use structopt::{StructOpt, StructOptInternal};

pub struct Block<'a, E: StructOpt> {
    pub comments: &'a str,
    pub lang: &'a str,
    pub directives: &'a str,
    pub arguments: Arguments<E>,
    pub text: &'a str,
    pub result: Cell<Option<String>>,
}

impl<'a, E: StructOpt> Display for Block<'a, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.comments)?;
        write!(f, "```{}", self.lang)?;
        if !self.directives.is_empty() {
            write!(f, " {}", self.directives)?;
        }
        write!(f, "\n")?;
        let result = self.result.take();
        if (self.arguments.draft || self.text.is_empty()) && result.is_some() {
            write!(f, "{}", result.unwrap())?
        } else {
            write!(f, "{}", self.text)?
        }
        write!(f, "\n```")
    }
}

#[derive(StructOpt, Debug)]
pub struct Arguments<E: StructOpt> {
    #[structopt(long)]
    pub draft: bool,

    #[structopt(flatten)]
    pub rest: E,
}

#[derive(StructOpt, Debug)]
pub struct Syntax {
    #[structopt(long)]
    pub trivia: bool,
}

pub fn parse_block<'a, E: StructOptInternal + Debug>(
    content: &'a str,
) -> Option<(&'a str, Block<'a, E>)> {
    let (comments, content) = content.split_once("```")?;
    let (lang, content) = content.split_once("\n")?;
    let (lang, directives) = lang.split_once(" ").unwrap_or((lang, ""));
    let directives = directives.trim();
    let (mut text, content) = content.split_once("```")?;
    if text.ends_with('\n') {
        // Remove the last \n before the  if it's present
        text = &text[0..text.len() - 1];
    }

    let arguments =
        std::iter::once("binary").chain(directives.trim().split(" ").filter(|arg| !arg.is_empty()));

    let arguments = Arguments::from_iter(arguments);

    Some((
        content,
        Block {
            comments,
            lang,
            directives,
            arguments,
            text,
            result: Cell::new(None),
        },
    ))
}

pub fn run_test<E, F>(file: &Path, func: F) -> Result<(), Box<dyn std::error::Error>>
where
    E: StructOptInternal + Debug,
    F: Fn(&mut Vec<Block<'_, E>>),
{
    let content = std::fs::read_to_string(&file).unwrap();

    let mut blocks = vec![];
    let mut input = content.as_ref();

    let global_refresh = std::env::var("SNAPDOWN_REFRESH").is_ok();

    while let Some((rest, mut block)) = parse_block::<E>(&input) {
        input = rest;
        if global_refresh {
            block.arguments.draft = true;
        }
        blocks.push(block)
    }

    func(&mut blocks);

    if blocks
        .iter()
        .any(|block| block.arguments.draft || block.text.is_empty())
    {
        let output = blocks
            .into_iter()
            .map(|block| block.to_string())
            .chain(std::iter::once(String::from("\n")))
            .collect::<Vec<_>>()
            .join("");

        std::fs::write(file, output).unwrap();
        return Err(String::from("Test updated. Retry once again").into());
    } else {
        for block in blocks.iter_mut() {
            if let Some(result) = block.result.take() {
                similar_asserts::assert_eq!(block.text, result)
            }
        }
    }

    Ok(())
}
