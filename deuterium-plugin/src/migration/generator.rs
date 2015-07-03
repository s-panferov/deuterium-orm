use syntax::{codemap};
use syntax::ext::base;
use syntax::ext::build::AstBuilder;
use syntax::util::small_vector;
use std::ascii::AsciiExt;

use super::super::helpers;

impl super::super::Generator<()> for super::MigrationState {
    fn generate<'a>(self, sp: codemap::Span, cx: &mut base::ExtCtxt, _: ()) -> Box<base::MacResult + 'a> {

        let pathes = ::std::fs::read_dir(&self.path).unwrap();
        let mut migrations = vec![];

        let path_checker = regex!(r"^_(\d{12})");
        let upcaser = regex!(r"_([a-z])");

        for path in pathes {
            let path = path.unwrap().path();

            let filestem = match path.file_stem() {
                Some(f) => f.to_str().unwrap(),
                None => { continue }
            };
            let captures = path_checker.captures(&filestem[..]);

            if captures.is_none() { continue };

            let captures = captures.expect("Mailformed migration name");
            let tm = captures.at(1).expect("Timestamp must exists");
            let version: u64 = tm.parse().ok().expect("Timestamp must be valid u64");
            let name = filestem.replace(captures.at(0).unwrap(), "");

            let name = upcaser.replace_all(&name, |caps: &::regex::Captures| {
                caps.at(1).unwrap().to_ascii_uppercase()
            });

            migrations.push(format!("({}, {}, {})", filestem.to_string(), version, name.to_string()));
        }

        let macro_body = migrations.connect(", ");

        let mut impls = vec![];
        impls.push(helpers::generate_macro_invocation(cx, "migrations", macro_body, sp));
        base::MacEager::items(small_vector::SmallVector::many(impls))

    }
}
