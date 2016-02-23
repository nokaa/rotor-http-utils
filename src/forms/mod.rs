mod parser;

use chomp::parse_only;

use std::collections::HashMap;
use std::str;

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
    // The map that will store form `names` and values
    let mut form_map: HashMap<String, Vec<u8>> = HashMap::new();

    // Parse through form data and return `Vec<Form>`
    let val = match parse_only(parser::form, data) {
        Ok(v) => v,
        _ => return Err("Unable to parse form data!".to_string()),
    };

    // Insert each `Form` into the map
    for form in &val {
        // For both `name` and `value` we must run
        // `replace_special_characters`. Form data is
        // received with most non-alphanumeric characters
        // escaped. Form names typically do not contain
        // escaped characters, but they are allowed.
        // E.g. `file type` is a valid form name, but is
        // received as `file+type`.
        let name = try!(parser::replace_special_characters(form.name));
        let name = str::from_utf8(&name[..]).unwrap().to_string();
        let value = try!(parser::replace_special_characters(form.value));
        form_map.insert(name, value);
    }

    Ok(form_map)
}
