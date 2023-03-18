use std::{fmt::Debug, vec};

use nom::bytes::streaming::tag;
use nom::IResult;

fn main() {
    println!("Hello, world!");

    #[derive(Debug)]
    struct I;
    impl SingleCharShape for I {
        const NAME: char = 'I';
    }
    println!("{:?}", parse_one("I", &I));

    #[derive(Debug)]
    struct M;
    impl SingleCharShape for M {
        const NAME: char = 'M';
    }
    #[derive(Debug)]
    struct A;
    impl SingleCharShape for A {
        const NAME: char = 'A';
    }
    #[derive(Debug)]
    struct C;
    impl SingleCharShape for C {
        const NAME: char = 'C';
    }

    #[derive(Debug, Default)]
    struct X(String);
    impl IntoExtensionShape for X {
        fn as_shape(&self) -> ExtensionShape {
            ExtensionShape::Prefix(String::from("X"))
        }
    }

    println!("{:?}", parse_one("Xabcd", &X::default()));

    #[derive(Debug)]
    struct G;
    impl IntoExtensionShape for G {
        fn as_shape(&self) -> ExtensionShape {
            ExtensionShape::Multi(String::from("G"))
        }
        fn generate(&self) -> Vec<Extension> {
            ["i", "m", "a", "c"]
                .into_iter()
                .map(|x| Extension(x.to_string()))
                .collect()
        }
    }

    println!("{:?}", parse_one("G", &G));
    println!("{:?}", parse_one("q", &G));

    println!(
        "{:?}",
        parse("IMACXmyext", vec![&I, &M, &A, &C, &X::default()])
    );

    println!(
        "{:?}",
        parse("Xmyext_CMAI", vec![&I, &M, &A, &C, &X::default()])
    );
}

#[derive(Debug)]
pub enum ExtensionShape {
    Tag(String),
    Prefix(String),
    Multi(String),
}

impl ExtensionShape {
    pub fn identifier(&self) -> &str {
        match self {
            ExtensionShape::Tag(i) => i.as_str(),
            ExtensionShape::Prefix(i) => i.as_str(),
            ExtensionShape::Multi(i) => i.as_str(),
        }
    }
}

pub trait SingleCharShape {
    const NAME: char;
}
impl<T: SingleCharShape> IntoExtensionShape for T {
    fn as_shape(&self) -> ExtensionShape {
        ExtensionShape::Tag(String::from(Self::NAME))
    }
}

pub trait IntoExtensionShape {
    fn as_shape(&self) -> ExtensionShape;

    fn generate(&self) -> Vec<Extension> {
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct Extension(String);

pub fn parse_one<'str>(
    input: &'str str,
    ext: &dyn IntoExtensionShape,
) -> IResult<&'str str, Vec<Extension>> {
    let shape = ext.as_shape();
    match shape {
        ExtensionShape::Tag(_) => {
            let id = shape.identifier();
            nom::bytes::complete::tag(id)(input)
                .map(|(rest, x)| (rest, vec![Extension(x.to_owned())]))
        }
        ExtensionShape::Prefix(_) => {
            let id = shape.identifier();
            nom::sequence::pair(
                tag(id),
                nom::sequence::terminated(
                    nom::bytes::complete::take_till1(|c| c == '_'),
                    nom::multi::many_m_n(0, 1, nom::bytes::complete::tag("_")),
                ),
            )(input)
            .map(|(rest, (id, tail))| (rest, vec![Extension(format!("{}{}", id, tail))]))
        }
        ExtensionShape::Multi(_) => {
            let id = shape.identifier();
            nom::bytes::complete::tag(id)(input).map(|(rest, _)| (rest, ext.generate()))
        }
    }
}

pub fn parse<'str>(
    input: &'str str,
    extensions: Vec<&dyn IntoExtensionShape>,
) -> IResult<&'str str, Vec<Extension>> {
    let mut res = vec![];
    let mut remaining = input;
    while !remaining.is_empty() {
        let (idx, parsed);
        (idx, (remaining, parsed)) = extensions
            .as_slice()
            .iter()
            .enumerate()
            .find_map(|(idx, &ext)| parse_one(remaining, ext).ok().map(|r| (idx, r)))
            .ok_or(nom::Err::Failure(nom::error::Error {
                input: remaining,
                code: nom::error::ErrorKind::OneOf, // or something?
            }))?;

        res.push((idx, parsed));
    }

    res.sort_by_key(|&(idx, _)| idx);

    // TODO: flat_map seems wrongish; do all multi extensions generate "in order" (i.e. do they expand, at their position, to exactly the order that should be in the final string)? do we care about duplicates?

    Ok(("", res.into_iter().flat_map(|(_, e)| e).collect()))
}
