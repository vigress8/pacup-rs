use std::collections::HashMap;

use pacup::srcinfo::*;

#[test]
fn test_parse() {
    let source = include_str!("data/1password-cli-bin.SRCINFO");
    assert_eq!(
        parse(source),
        Ok(Package {
            base: "1password-cli-bin",
            version: "2.28.0",
            maintainers: vec!["Oren Klopfer <oren@taumoda.com>"],
            repology: HashMap::from([("project", "1password-cli")]),
            sources: vec![]
        })
    );
}
