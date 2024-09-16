use itertools::Itertools;
use std::{collections::HashMap, path::PathBuf};

pub fn parse_package(source: &str) -> Result<Package<'_>, String> {
    let attrs = source
        .lines()
        .filter_map(|line| match line.chars().next() {
            Some('\t') => Some(&line[1..]),
            Some(_) => Some(line),
            None => None,
        })
        .flat_map(Attr::try_from)
        .collect_vec();

    let (mut base, mut version) = ("", "");
    let (mut maintainers, mut sources) = (vec![], vec![]);
    let mut repology = HashMap::new();

    let (mut srcs, mut sums) = (vec![], vec![]);
    for (typ, mut chunk) in &attrs.iter().chunk_by(|attr| attr.typ) {
        match typ {
            AttrType::Pkgbase => {
                base = chunk.next().unwrap().value;
                if let Some(base2) = chunk.next() {
                    Err(format!(
                        "Duplicate `pkgbase` attribute: `{base}`, `{}`",
                        base2.value
                    ))?
                }
            }
            AttrType::Pkgver => {
                version = chunk.next().unwrap().value;
                if let Some(version2) = chunk.next() {
                    Err(format!(
                        "Duplicate `pkgver` attribute: `{version}`, `{}`",
                        version2.value
                    ))?
                }
            }
            AttrType::Maintainer => maintainers = chunk.map(|attr| attr.value).collect(),
            AttrType::Repology => {
                for attr in chunk {
                    let (key, value) = attr
                        .value
                        .split_once(": ")
                        .ok_or(format!("Invalid Repology entry: `{}`", attr.value))?;
                    repology.insert(key, value);
                }
            }
            AttrType::Source => srcs = chunk.collect(),
            AttrType::HashSum(_) => sums = chunk.filter(|sum| sum.value != "SKIP").collect(),
        }
    }

    for src in srcs {
        let mut hashes = vec![];

        let (dest, url) = if let Some((dest, url)) = src.value.split_once("::") {
            (dest.into(), url)
        } else {
            (src.value.rsplit('/').next().unwrap().into(), src.value)
        };

        use HashType::*;
        for typ in [B2, MD5, SHA1, SHA224, SHA256, SHA384, SHA512] {
            for (i, sum) in sums.iter().enumerate() {
                match sum {
                    Attr {
                        arch,
                        distro,
                        typ: AttrType::HashSum(t),
                        value,
                    } if *arch == src.arch && *distro == src.distro && *t == typ => {
                        hashes.push(HashSum { typ, value });
                        sums.remove(i);
                        break;
                    }
                    _ => {}
                }
            }
        }

        sources.push(SourceEntry { dest, url, hashes });
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
    pub dest: PathBuf,
    pub url: &'a str,
    pub hashes: Vec<HashSum<'a>>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HashSum<'a> {
    pub typ: HashType,
    pub value: &'a str,
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
