use extract::{Context, Error, Extract, Immediate};
use percent_encoding;
use std::borrow::Cow;
use util::BufStream;

fn decode(s: &str) -> Result<String, Error> {
    percent_encoding::percent_decode(s.as_bytes())
        .decode_utf8()
        .map(Cow::into_owned)
        .map_err(|e| Error::invalid_argument(&e))
}

impl<B: BufStream> Extract<B> for String {
    type Future = Immediate<Self>;

    fn extract(ctx: &Context) -> Self::Future {
        use codegen::Source::*;

        match ctx.callsite().source() {
            Capture(idx) => {
                let path = ctx.request().uri().path();
                let value = ctx.captures().get(*idx, path);

                Immediate::result(decode(value))
            }
            Header(header_name) => {
                let value = match ctx.request().headers().get(header_name) {
                    Some(value) => value,
                    None => {
                        return Immediate::err(Error::missing_argument());
                    }
                };

                let r = value
                    .to_str()
                    .map(|s| s.to_string())
                    .map_err(|e| Error::invalid_argument(&e));
                Immediate::result(r)
            }
            QueryString => {
                let query = ctx.request().uri().query().unwrap_or("");

                Immediate::result(decode(query))
            }
            Body => {
                unimplemented!();
            }
            Unknown => {
                unimplemented!();
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn extract() {
        assert_eq!("hello, world", decode("hello,%20world").unwrap());
        assert!(decode("%ff").unwrap_err().is_invalid_argument());
    }
}
