use pacup::srcinfo::*;

#[test]
fn test_parse() {
    let source = include_str!("data/1password-cli-bin.SRCINFO");
    assert_eq!(
        parse(source),
        Ok(vec![
            Attr::new(
                AttrType::Pkgbase,
                "1password-cli-bin",
                Arch::All,
                Distro::All
            ),
            Attr::new(AttrType::Pkgver, "2.28.0", Arch::All, Distro::All),
            Attr::new(
                AttrType::Maintainer,
                "Oren Klopfer <oren@taumoda.com>",
                Arch::All,
                Distro::All
            ),
            Attr::new(
                AttrType::Repology,
                "project: 1password-cli",
                Arch::All,
                Distro::All
            ),
            Attr::new(
                AttrType::Source,
                "https://cache.agilebits.com/dist/1P/op2/pkg/v2.28.0/op_linux_amd64_v2.28.0.zip",
                Arch::All,
                Distro::All
            ),
            Attr::new(
                AttrType::HashSum(HashType::SHA256),
                "a0732965d86c4429b508d1f00531359936a50d62f78b01fc2df964d9f5f47982",
                Arch::All,
                Distro::All
            )
        ])
    );
}
