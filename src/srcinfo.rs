use itertools::Itertools;
use std::{collections::HashMap, path::Path};

pub fn parse(source: &str) -> Result<Package<'_>, String> {
    let attrs = source
        .lines()
        .filter_map(|line| match line.chars().next() {
            Some('\t') => Some(&line[1..]),
            Some(_) => Some(line),
            None => None,
        })
        .flat_map(Attr::try_from);

    let mut base = "";
    let mut version = "";
    let mut maintainers = vec![];
    let mut repology = HashMap::new();
    let mut sources = vec![];

    let mut srcs = vec![];
    let mut sums = vec![];
    for attr in attrs {
        match attr.typ {
            AttrType::Pkgbase => base = attr.value,
            AttrType::Pkgver => version = attr.value,
            AttrType::Maintainer => maintainers.push(attr.value),
            AttrType::Repology => {
                let (key, value) = attr.value.split_once(": ").unwrap();
                repology.insert(key, value);
            }
            AttrType::Source => srcs.push(attr),
            AttrType::HashSum(_) => sums.push(attr),
        }
    }

    if base.is_empty() {
        Err("Required attribute `pkgbase` not found".to_owned())?
    }

    if version.is_empty() {
        Err("Required attribute `pkgver` not found".to_owned())?
    }


    Ok(Package {
        base,
        version,
        maintainers,
        repology,
        sources,
    })
}

#[derive(Clone, Debug, PartialEq)]
pub struct Package<'a> {
    pub base: &'a str,
    pub version: &'a str,
    pub maintainers: Vec<&'a str>,
    pub repology: HashMap<&'a str, &'a str>,
    pub sources: Vec<SourceEntry<'a>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SourceEntry<'a> {
    dest: &'a Path,
    url: &'a str,
    hashes: Vec<HashSum<'a>>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HashSum<'a> {
    typ: HashType,
    value: &'a str,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Attr<'a> {
    pub typ: AttrType,
    pub value: &'a str,
    pub arch: Arch,
    pub distro: Distro,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AttrType {
    Pkgbase,
    Pkgver,
    Maintainer,
    Repology,
    Source,
    HashSum(HashType),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HashType {
    B2,
    MD5,
    SHA1,
    SHA224,
    SHA256,
    SHA384,
    SHA512,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Arch {
    All,
    Amd64,
    Arm64,
    Armel,
    Armhf,
    I386,
    Mips64el,
    Ppc64el,
    Riscv64,
    S390x,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Distro {
    All,
    Debian(Debian),
    Ubuntu(Ubuntu),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Debian {
    All,
    Bullseye,
    Bookworm,
    Trixie,
    Sid,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Ubuntu {
    All,
    Focal,
    Jammy,
    Noble,
    Oracular,
    Devel,
}

impl<'a> Attr<'a> {
    pub fn new(typ: AttrType, value: &'a str, arch: Arch, distro: Distro) -> Self {
        Self {
            typ,
            value,
            arch,
            distro,
        }
    }
}

impl<'a> TryFrom<&'a str> for Attr<'a> {
    type Error = String;
    fn try_from(line: &'a str) -> Result<Self, Self::Error> {
        let (key, value) = line.split_once(" = ").unwrap();
        let mut parts = key.split('_');
        let attr = parts.next().unwrap();
        let typ = attr.try_into()?;
        let next_part = parts.next().map(|p| match p {
            "x86" => {
                assert_eq!(parts.next(), Some("64"));
                "x86_64"
            }
            _ => p,
        });
        let (distro, arch) = match next_part {
            None => (Distro::All, Arch::All),
            Some(distro_or_arch) => match distro_or_arch.try_into() {
                Ok(distro) => {
                    let arch = parts
                        .next()
                        .map(|p| p.try_into().unwrap_or(Arch::All))
                        .unwrap_or(Arch::All);
                    (distro, arch)
                }
                Err(_) => {
                    let arch = distro_or_arch.try_into().map_err(|_| {
                        format!("Unknown distro or architecture: `{distro_or_arch}`")
                    })?;
                    (Distro::All, arch)
                }
            },
        };

        Ok(Attr::new(typ, value, arch, distro))
    }
}

impl TryFrom<&str> for AttrType {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        use AttrType::*;
        match value {
            "pkgbase" => Ok(Pkgbase),
            "pkgver" => Ok(Pkgver),
            "maintainer" => Ok(Maintainer),
            "repology" => Ok(Repology),
            "source" => Ok(Source),
            _ => {
                if let Ok(hash_type) = value.try_into() {
                    Ok(HashSum(hash_type))
                } else {
                    Err(format!("Unknown attribute: `{value}`"))
                }
            }
        }
    }
}

impl TryFrom<&str> for HashType {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        use HashType::*;
        match value {
            "b2sums" => Ok(B2),
            "md5sums" => Ok(MD5),
            "sha1sums" => Ok(SHA1),
            "sha224sums" => Ok(SHA224),
            "sha256sums" => Ok(SHA256),
            "sha384sums" => Ok(SHA384),
            "sha512sums" => Ok(SHA512),
            _ => Err(format!("Unknown hash type: `{value}`")),
        }
    }
}

impl TryFrom<&str> for Arch {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        use Arch::*;
        match value {
            "amd64" | "x86_64" => Ok(Amd64),
            "arm64" | "aarch64" => Ok(Arm64),
            "armel" | "arm" => Ok(Armel),
            "armhf" | "armv7h" => Ok(Armhf),
            "i386" | "i686" => Ok(I386),
            "mips64el" => Ok(Mips64el),
            "ppc64el" => Ok(Ppc64el),
            "riscv64" => Ok(Riscv64),
            "s390x" => Ok(S390x),
            _ => Err(format!("Unknown architecture: `{value}`")),
        }
    }
}

impl TryFrom<&str> for Distro {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value
            .try_into()
            .map(Self::Debian)
            .or(value.try_into().map(Self::Ubuntu))
    }
}

impl TryFrom<&str> for Debian {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        use Debian::*;
        match value {
            "debian" => Ok(All),
            "bullseye" => Ok(Bullseye),
            "bookworm" => Ok(Bookworm),
            "trixie" => Ok(Trixie),
            "sid" => Ok(Sid),
            _ => Err(format!("Unknown Debian release: `{value}`")),
        }
    }
}

impl TryFrom<&str> for Ubuntu {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        use Ubuntu::*;
        match value {
            "ubuntu" => Ok(All),
            "focal" => Ok(Focal),
            "jammy" => Ok(Jammy),
            "noble" => Ok(Noble),
            "oracular" => Ok(Oracular),
            "devel" => Ok(Devel),
            _ => Err(format!("Unknown Ubuntu release: `{value}`")),
        }
    }
}
