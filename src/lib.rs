//! Nigeria's NIPOST digital postcode (NDAPS) for DuckDB, on the `ng-postcode` Rust core.
//!
//! The functions have the names and answers of the macros in ng-postcode's `sql/duckdb.sql`.
//! A value that is not well formed gives NULL, and so does a NULL argument.

use std::error::Error;

use duckdb::core::{DataChunkHandle, Inserter, LogicalTypeHandle, LogicalTypeId};
use duckdb::ffi::duckdb_string_t;
use duckdb::types::DuckString;
use duckdb::vscalar::{ScalarFunctionSignature, VScalar};
use duckdb::vtab::arrow::WritableVector;
use duckdb::{Connection, Result, duckdb_entrypoint_c_api};
use ng_postcode::{Postcode, Prefix, Segment};

/// The text arguments of one row, or `None` if any of them is NULL.
fn arguments<const N: usize>(input: &DataChunkHandle, row: usize) -> Option<[String; N]> {
    let mut texts = [const { String::new() }; N];
    for (column, text) in texts.iter_mut().enumerate() {
        let vector = input.flat_vector(column);
        if vector.row_is_null(row as u64) {
            return None;
        }
        let mut value = unsafe { vector.as_slice_with_len::<duckdb_string_t>(input.len()) }[row];
        *text = DuckString::new(&mut value).as_str().into_owned();
    }
    Some(texts)
}

fn varchars(count: usize) -> Vec<LogicalTypeHandle> {
    (0..count).map(|_| LogicalTypeId::Varchar.into()).collect()
}

/// A scalar function from `$arity` text arguments to text, or NULL.
macro_rules! text_function {
    ($name:ident, $arity:literal, |$arguments:pat_param| $answer:expr) => {
        struct $name;

        impl VScalar for $name {
            type State = ();

            fn invoke(
                _: &(),
                input: &mut DataChunkHandle,
                output: &mut dyn WritableVector,
            ) -> Result<(), Box<dyn Error>> {
                let mut out = output.flat_vector();
                for row in 0..input.len() {
                    let answer: Option<String> =
                        arguments::<$arity>(input, row).and_then(|$arguments| $answer);
                    match answer {
                        Some(text) => out.insert(row, text.as_str()),
                        None => out.set_null(row),
                    }
                }
                Ok(())
            }

            fn signatures() -> Vec<ScalarFunctionSignature> {
                vec![ScalarFunctionSignature::exact(
                    varchars($arity),
                    LogicalTypeId::Varchar.into(),
                )]
            }
        }
    };
}

fn segment(name: &str) -> Option<Segment> {
    Some(match name {
        "state" => Segment::State,
        "lga" => Segment::Lga,
        "district" => Segment::District,
        "area" => Segment::Area,
        "unit" => Segment::Unit,
        _ => return None,
    })
}

text_function!(Version, 0, |[]| Some(env!("CARGO_PKG_VERSION").to_owned()));

// The letters and digits of a code in upper case, or NULL if it holds anything else.
text_function!(Compact, 1, |[text]| {
    let allowed = |c: char| c.is_ascii_alphanumeric() || c == ' ' || c == '-';
    text.chars().all(allowed).then(|| {
        let kept = text.chars().filter(char::is_ascii_alphanumeric);
        kept.map(|c| c.to_ascii_uppercase()).collect()
    })
});

// The compact code, EK01A03FK01.
text_function!(Parse, 1, |[text]| Some(
    Postcode::parse(&text).ok()?.as_str().to_owned()
));

// The compact prefix, such as EK01A03.
text_function!(PrefixParse, 1, |[text]| Some(
    Prefix::parse(&text).ok()?.as_str().to_owned()
));

// The hyphenated form of a code or a prefix.
text_function!(Format, 1, |[text]| Some(
    Prefix::parse(&text).ok()?.to_string()
));

// The compact prefix of a code, cut off after a segment. NULL for an unknown segment.
text_function!(Truncate, 2, |[code, through]| {
    let prefix = Postcode::parse(&code).ok()?.truncate(segment(&through)?);
    Some(prefix.as_str().to_owned())
});

// The prefix one segment shorter, or NULL for a state.
text_function!(Parent, 1, |[text]| Some(
    Prefix::parse(&text).ok()?.parent()?.as_str().to_owned()
));

/// Whether a code starts with a prefix.
struct Within;

impl VScalar for Within {
    type State = ();

    fn invoke(
        _: &(),
        input: &mut DataChunkHandle,
        output: &mut dyn WritableVector,
    ) -> Result<(), Box<dyn Error>> {
        let mut out = output.flat_vector();
        for row in 0..input.len() {
            let answer = arguments::<2>(input, row).and_then(|[code, prefix]| {
                Some(
                    Prefix::parse(&prefix)
                        .ok()?
                        .contains(&Postcode::parse(&code).ok()?),
                )
            });
            match answer {
                Some(inside) => unsafe {
                    out.as_mut_slice_with_len::<bool>(input.len())[row] = inside
                },
                None => out.set_null(row),
            }
        }
        Ok(())
    }

    fn signatures() -> Vec<ScalarFunctionSignature> {
        vec![ScalarFunctionSignature::exact(
            varchars(2),
            LogicalTypeId::Boolean.into(),
        )]
    }
}

#[duckdb_entrypoint_c_api]
pub unsafe fn extension_entrypoint(con: Connection) -> Result<(), Box<dyn Error>> {
    con.register_scalar_function::<Version>("ng_postcode_version")?;
    con.register_scalar_function::<Compact>("ng_postcode_compact")?;
    con.register_scalar_function::<Parse>("ng_postcode_parse")?;
    con.register_scalar_function::<PrefixParse>("ng_postcode_prefix_parse")?;
    con.register_scalar_function::<Format>("ng_postcode_format")?;
    con.register_scalar_function::<Truncate>("ng_postcode_truncate")?;
    con.register_scalar_function::<Parent>("ng_postcode_parent")?;
    con.register_scalar_function::<Within>("ng_postcode_within")?;
    Ok(())
}
