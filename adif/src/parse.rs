// https://docs.rs/crate/adif/0.1.3/source/src/parser.rs

use regex::Regex;

use crate::data::{self, ADIFFile, ADIFRecord, ADIFType};

pub struct Token {
    pub key: String,
    pub len: usize,
    pub ty: Option<char>,
    pub val: String,
}

pub fn parse_tokens(data: &str) -> Vec<Token> {
    Regex::new(r"<([a-zA-Z|_]+):(\d+)(?::([a-z]))?>([^<\n]+)")
        .unwrap()
        .captures_iter(data)
        .map(|cap| Token {
            key: cap[1].to_string().to_uppercase(),
            len: cap[2].parse().expect("len in token not integer"),
            ty: cap
                .get(3)
                .map(|m| m.as_str().chars().next().unwrap().to_ascii_uppercase()),
            val: cap[4].trim_end().to_string(),
        })
        .collect()
}

pub fn build_token_list(tokens: Vec<Token>) -> Vec<(String, ADIFType)> {
    let mut tuples = Vec::new();
    for token in tokens {
        tuples.push((
            token.key.clone(),
            match token.ty {
                Some(ty) => match ty {
                    _ => ADIFType::Str(token.val),
                },
                None => ADIFType::Str(token.val),
            },
        ));
    }
    tuples
}

pub fn parse_adif(data: &str) -> ADIFFile {
    let data = data.replace("<eoh>", "<EOH>").replace("<eor>", "<EOR>");
    let data = data.split("<EOH>");
    let data = data.collect::<Vec<&str>>();

    let header = match data.len() {
        1 => {
            todo!()
        }
        2 => build_token_list(parse_tokens(data.first().unwrap_or(&""))),
        _ => {
            // bad file (multiple headers or blank)
            todo!()
        }
    };

    ADIFFile {
        header: data::ADIFHeader(header),
        body: data
            .last()
            .unwrap_or(&"")
            .split_terminator("<EOR>")
            .collect::<Vec<&str>>()
            .iter()
            .map(|l| data::ADIFRecord(build_token_list(parse_tokens(l))))
            .collect::<Vec<ADIFRecord>>(),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::{
        data::ADIFType,
        parse::{self, ADIFFile},
    };

    #[test]
    pub fn parse_adif() {
        let data = "ADIF Export\n
            <adif_ver:5>3.1.1\n
            <eoh>\n
            <call:6>N0CALL <gridsquare:4>AA00 <eor>";
        let file = parse::parse_adif(data);
        assert_eq!(
            file,
            ADIFFile {
                header: crate::data::ADIFHeader(vec![(
                    "ADIF_VER".to_string(),
                    ADIFType::Str("3.1.1".to_string(),),
                ),]),
                body: vec![crate::data::ADIFRecord(vec![
                    ("CALL".to_string(), ADIFType::Str("N0CALL".to_string(),),),
                    ("GRIDSQUARE".to_string(), ADIFType::Str("AA00".to_string(),),),
                ]),],
            }
        );
        println!("{}", file.serialize().unwrap());
    }
}
