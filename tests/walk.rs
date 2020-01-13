use std::fs;
use std::path::Path;

use pmv::walk;

fn setup(id: &str) {
    let _ = fs::create_dir(Path::new("temp"));
    let _ = fs::remove_dir_all(Path::new(&format!("temp/{}", id)));
    for dir1 in ["foo", "bar", "baz"].iter() {
        for dir2 in ["foo", "bar", "baz"].iter() {
            let _ = fs::create_dir_all(Path::new(&format!("temp/{}/{}/{}", id, dir1, dir2)));
            for fname in ["foo", "bar", "baz"].iter() {
                let path: String = format!("temp/{}/{}/{}/{}", id, dir1, dir2, fname);
                fs::write(Path::new(&path), path.as_bytes()).unwrap();
            }
        }
    }
}

#[test]
fn no_specials() {
    setup("no_specials");
    let matches = walk(Path::new("temp/no_specials"), "foo/bar/baz").unwrap();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].0.path(), Path::new("temp/no_specials/foo/bar/baz"));
    assert_eq!(matches[0].1, Vec::<String>::new());
}

#[test]
fn question() {
    setup("question");
    let mut matches = walk(Path::new("temp/question"), "ba?/ba?/ba?").unwrap();
    assert_eq!(matches.len(), 8);
    matches.sort_by(|a, b| a.0.path().cmp(&b.0.path()));

    let paths: Vec<_> = matches.iter().map(|m| m.0.path()).collect();
    assert_eq!(
        paths,
        vec![
            Path::new("temp/question/bar/bar/bar"),
            Path::new("temp/question/bar/bar/baz"),
            Path::new("temp/question/bar/baz/bar"),
            Path::new("temp/question/bar/baz/baz"),
            Path::new("temp/question/baz/bar/bar"),
            Path::new("temp/question/baz/bar/baz"),
            Path::new("temp/question/baz/baz/bar"),
            Path::new("temp/question/baz/baz/baz"),
        ]
    );

    let patterns: Vec<_> = matches
        .iter()
        .map(|x| {
            let s = &x.1;
            s.iter().fold("".to_string(), |acc, x| acc + "." + x)
        })
        .collect();
    assert_eq!(
        patterns,
        vec![
            String::from(".r.r.r"),
            String::from(".r.r.z"),
            String::from(".r.z.r"),
            String::from(".r.z.z"),
            String::from(".z.r.r"),
            String::from(".z.r.z"),
            String::from(".z.z.r"),
            String::from(".z.z.z"),
        ]
    );
}

#[test]
fn star() {
    setup("star");
    let mut matches = walk(Path::new("temp/star"), "b*/b*/b*").unwrap();
    assert_eq!(matches.len(), 8);
    matches.sort_by(|a, b| a.0.path().cmp(&b.0.path()));

    let paths: Vec<_> = matches.iter().map(|x| x.0.path()).collect();
    assert_eq!(
        paths,
        vec![
            Path::new("temp/star/bar/bar/bar"),
            Path::new("temp/star/bar/bar/baz"),
            Path::new("temp/star/bar/baz/bar"),
            Path::new("temp/star/bar/baz/baz"),
            Path::new("temp/star/baz/bar/bar"),
            Path::new("temp/star/baz/bar/baz"),
            Path::new("temp/star/baz/baz/bar"),
            Path::new("temp/star/baz/baz/baz"),
        ]
    );

    let patterns: Vec<_> = matches
        .iter()
        .map(|x| {
            let s = &x.1;
            s.iter().fold("".to_string(), |acc, x| acc + "." + x)
        })
        .collect();
    assert_eq!(
        patterns,
        vec![
            String::from(".ar.ar.ar"),
            String::from(".ar.ar.az"),
            String::from(".ar.az.ar"),
            String::from(".ar.az.az"),
            String::from(".az.ar.ar"),
            String::from(".az.ar.az"),
            String::from(".az.az.ar"),
            String::from(".az.az.az"),
        ]
    );
}
