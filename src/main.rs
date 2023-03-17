use std::fmt::Debug;
use std::ops::Deref;

use nom;
use nom::bytes::streaming::tag;
use nom::{character, IResult};

fn main() {
    println!("Hello, world!");

    #[derive(Debug)]
    struct I;
    impl IntoExtensionShape for I {
        fn into_shape(&self) -> ExtensionShape {
            ExtensionShape::Tag(String::from("I"))
        }

        fn generate(&self) -> Vec<Extension> {
            vec![Extension(String::from("I"))]
        }
    }
    println!("{:?}", parse_one("I", Box::new(I)));

    #[derive(Debug)]
    struct X(String);
    impl IntoExtensionShape for X {
        fn into_shape(&self) -> ExtensionShape {
            ExtensionShape::Prefix(String::from("X"))
        }

        fn generate(&self) -> Vec<Extension> {
            vec![Extension(format!("X{}", self.0))]
        }
    }

    println!("{:?}", parse_one("Xabcd", Box::new(X(Default::default()))));

    #[derive(Debug)]
    struct G;
    impl IntoExtensionShape for G {
        fn into_shape(&self) -> ExtensionShape {
            ExtensionShape::Multi(String::from("G"))
        }
        fn generate(&self) -> Vec<Extension> {
            ["i", "m", "a", "c"].into_iter().map(|x| Extension(x.to_string())).collect()
        }
    }
    
    println!("{:?}", parse_one("G", Box::new(G)));
}

#[derive(Debug)]
pub enum ExtensionShape {
    Tag(String),
    Prefix(String),
    Multi(String)
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

pub trait IntoExtensionShape {
    fn into_shape(&self) -> ExtensionShape;

    fn generate(&self) -> Vec<Extension>;
}

#[derive(Debug)]
pub struct Extension(String);

pub fn parse_one(input: &str, ext: Box<dyn IntoExtensionShape>) -> IResult<&str, Vec<Extension>> {
    let shape = ext.into_shape();
    match shape {
        ExtensionShape::Tag(_) => {
            let id = shape.identifier();
            nom::bytes::complete::tag(id)(input).map(|(rest, x)| (rest, vec![Extension(x.to_owned())]))
        }
        ExtensionShape::Prefix(_) => {
            let id = shape.identifier();
            nom::sequence::pair(tag(id), nom::bytes::complete::take_till1(|c| c == '_'))(input)
                .map(|(rest, (id, tail))| (rest, vec![Extension(format!("{}{}", id, tail))]))
        },
        ExtensionShape::Multi(_) => {
            let id = shape.identifier();
            nom::bytes::complete::tag(id)(input).map(|(rest, _)| (rest, ext.generate()))
        }
    }
}

pub fn parse(input: &str, extensions: Vec<Box<dyn IntoExtensionShape>>) -> IResult<&str, Vec<Extension>> {
    for ext in extensions {
        let parsed = parse_one(input, ext);
    }
    Ok(("", vec![]))
}
