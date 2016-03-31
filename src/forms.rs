use std::collections::HashMap;
use std::str;
use std::u8;

//use nom;

/// This function recevies our form data byte array as input
/// and returns a map of the form
/// ```text
/// { name: value }
/// ```
///
/// `Err(String)` is returned if there is an error with parsing,
/// or if `data` is invalid.
//
// TODO(nokaa): This function is currently O(n^2).
// This is unacceptable, we must refactor it to be O(n).
// The issue is that we parse the form into names and values,
// and then we parse through each name and value to replace
// escaped characters. Escaped characters ought be replaced
// during the inital parse.
pub fn parse_form(data: &[u8]) -> Result<HashMap<String, Vec<u8>>, String> {
    use nom::IResult::*;
    let mut form_map: HashMap<String, Vec<u8>> = HashMap::new();

    let forms = match form_data_parser(data) {
        Done(_, f) => f,
        _ => return Err(String::from("Error parsing data")),
    };

    for form in &forms {
        let name = try!(replace_special_characters(form.name));
        let name = match str::from_utf8(&name[..]) {
            Ok(n) => n.to_string(),
            Err(e) => return Err(format!("{}", e)),
        };

        let value = try!(replace_special_characters(form.value));
        form_map.insert(name, value);
    }

    Ok(form_map)
}

/// The type used for parsing form data
#[derive(Debug)]
struct Form<'a> {
    /// The `name` field in an html input
    pub name: &'a [u8],
    /// The value associated with the input field
    pub value: &'a [u8],
}

named!(form_data_parser<&[u8], Vec<Form> >, many0!(form_parser));
named!(form_parser<&[u8], Form>,
    chain!(
        name: name ~
        val: value ,

        ||{Form{name: name, value: val}}
    )
);
named!(name, take_until_and_consume!([b'=']));
named!(value, alt!(take_until_and_consume!([b'&']) | take_while!(t)));

fn t<T> (_: T) -> bool {
    true
}

/// When we receive form data with enctype
/// `application/x-www-form-urlencoded` any characters 
/// that are not `[0-9A-Za-z]`, `*`, `-`, or `_` are
/// replaced by `%XX` where `XX` is the 2 digit hex value
/// of the character. Spaces (' ') are replaced by `+`.
/// 
/// `replace_special_characters` goes through `data` and
/// replaces all escaped characters with the appropriate
/// character.
///
/// For now we ignore carriage returns, as *nix dislikes them.
///
/// Returns `Err(String)` if we receive invalid input, i.e. if
/// `data` ends with `%` or `%X`.
///
pub fn replace_special_characters(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut buf: Vec<u8> = vec![];

    let mut data = data.into_iter();

    while let Some(&c) = data.next() {
        if b'%' == c {
            let mut d: Vec<u8> = vec![];

            if let Some(&c) = data.next() {
                d.push(c);
            } else {
                return Err("Unexpected end of input!".to_string());
            }
            if let Some(&c) = data.next() {
                d.push(c);
            } else {
                return Err("Unexpected end of input!".to_string());
            }

            let val = match str::from_utf8(&d[..]) {
                Err(e) => return Err(format!("{}", e)),
                Ok(v) => {
                    match u8::from_str_radix(v, 16) {
                        Ok(v) => v,
                        Err(_) => return Err(format!("Error parsing hex value {}", v)),
                    }
                }
            };

            // For now we are not pushing carriage returns, eventually we
            // should maybe check if we are on Windows or *nix?
            // TODO(nokaa): Handle this properly
            if b'\r' != val {
                buf.push(val);
            }
        } else if b'+' == c {
            buf.push(b' ');
        } else {
            buf.push(c);
        }
    }

    Ok(buf)
}
