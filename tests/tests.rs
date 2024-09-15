use pacup::srcinfo::*;
use std::collections::HashMap;

#[test]
fn test_1password_cli_bin() {
    let source = include_str!("data/1password-cli-bin.SRCINFO");
    assert_eq!(
        parse_package(source),
        Ok(Package {
            base: "1password-cli-bin",
            version: "2.28.0",
            maintainers: vec!["Oren Klopfer <oren@taumoda.com>"],
            repology: HashMap::from([("project", "1password-cli")]),
            sources: vec![SourceEntry {
                dest: None,
                url:
                    "https://cache.agilebits.com/dist/1P/op2/pkg/v2.28.0/op_linux_amd64_v2.28.0.zip",
                hashes: vec![HashSum {
                    typ: HashType::SHA256,
                    value: "a0732965d86c4429b508d1f00531359936a50d62f78b01fc2df964d9f5f47982",
                }]
            }]
        })
    );
}

#[test]
fn test_megasync_deb() {
    let source = include_str!("data/megasync-deb.SRCINFO");
    assert_eq!(
        parse_package(source),
        Ok(Package {
            base: "megasync-deb",
            version: "5.2.1",
            maintainers: vec!["Arrowsome <ramtintoosi@gmail.com>"],
            repology: HashMap::from([("project", "megasync")]),
            sources: vec![
                SourceEntry {
                    dest: None,
                    url: "https://mega.nz/linux/repo/xUbuntu_22.04/amd64/megasync_5.2.1-4.1_amd64.deb",
                    hashes: vec![HashSum {
                        typ: HashType::SHA256,
                        value: "5b8ce6c03a0a933832451eab2425519e7fa8480f58b5a0c6d9457476ec0e9aa1"
                    }]
                },
                SourceEntry {
                    dest: None,
                    url: "https://mega.nz/linux/repo/xUbuntu_24.04/amd64/megasync_5.2.1-2.1_amd64.deb",
                    hashes: vec![HashSum {
                        typ: HashType::SHA256,
                        value: "3e1f9816c449dede48a095d7cdcba733be26739fb1e6cbc1a43a47a372654865"
                    }]
                },
                SourceEntry {
                    dest: None,
                    url: "https://mega.nz/linux/repo/Debian_11/amd64/megasync_5.2.1-4.1_amd64.deb",
                    hashes: vec![HashSum {
                        typ: HashType::SHA256,
                        value: "304ae53fbdbbfa5fd290c97129e93df9ef0ea5610b94a68e8a4e6d8cee8fd1b3"
                    }]
                },
                SourceEntry {
                    dest: None,
                    url: "https://mega.nz/linux/repo/Debian_12/amd64/megasync_5.2.1-4.1_amd64.deb",
                    hashes: vec![HashSum {
                        typ: HashType::SHA256,
                        value: "5a63deac549fd09daeca479fd0bbbf0cb329dc9bef1245a334a1a75535548635"
                    }]
                }
            ]
        })
    );
}
