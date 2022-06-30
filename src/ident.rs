use heck::ToUpperCamelCase;

/// Converts a `snake_case` identifier to an `UpperCamel` case Rust type identifier.
pub fn to_upper_camel(s: &str) -> String {
    let mut ident = s.to_upper_camel_case();

    // Suffix an underscore for the `Self` Rust keyword as it is not allowed as raw identifier.
    if ident == "Self" {
        ident += "_";
    }
    ident
}
