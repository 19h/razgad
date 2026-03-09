use crate::{Confidence, DetectedSymbol, Error, Scheme, decode, schemes};

pub fn heuristic_decode_symbol(input: &str) -> Result<DetectedSymbol, Error> {
    let (scheme, confidence) = detect_scheme(input)?;
    let symbol = decode(scheme, input)?;
    Ok(DetectedSymbol {
        scheme,
        confidence,
        symbol,
    })
}

fn detect_scheme(input: &str) -> Result<(Scheme, Confidence), Error> {
    schemes::detect(input)
        .ok_or_else(|| Error::new(format!("unable to determine mangling scheme: {input}")))
}
