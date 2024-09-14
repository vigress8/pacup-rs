#[derive(Clone, Debug, PartialEq)]
pub struct Package<'a> {
    pub base: &'a str,
    pub version: &'a str,
    pub maintainers: Vec<&'a str>,
    pub repology: RepologyInfo,
    pub sources: Vec<SourceEntry>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RepologyInfo {}

#[derive(Clone, Debug, PartialEq)]
pub struct SourceEntry {}

#[derive(Clone, Debug, PartialEq)]
pub struct Attr<'a> {
    pub typ: AttrType,
    pub value: &'a str,
    pub arch: Arch,
    pub distro: Distro,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AttrType {
    Pkgbase,
    Pkgver,
    Maintainer,
    Repology,
    Source,
    HashSum(HashType),
}

#[derive(Clone, Debug, PartialEq)]
pub enum HashType {
    B2,
    MD5,
    SHA1,
    SHA224,
    SHA256,
    SHA384,
    SHA512,
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

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
pub enum Distro {
    All,
    Debian(Debian),
    Ubuntu(Ubuntu),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Debian {
    All,
    Bullseye,
    Bookworm,
    Trixie,
    Sid,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Ubuntu {
    All,
    Focal,
    Jammy,
    Noble,
    Oracular,
    Devel,
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
            _ => Err(format!("Invalid architecture: {value}")),
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
            _ => Err(format!("Invalid Debian release: {value}")),
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
            _ => Err(format!("Invalid Ubuntu release: {value}")),
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
                        format!("Invalid distro or architecture: `{distro_or_arch}`")
                    })?;
                    (Distro::All, arch)
                }
            },
        };

        Ok(Attr::new(typ, value, arch, distro))
    }
}

pub fn parse(source: &str) -> Result<Vec<Attr<'_>>, String> {
    let it = source
        .lines()
        .filter_map(|mut line| {
            if let Some('\t') = line.chars().next() {
                line = &line[1..];
            }
            match line.split(&['_', ' ']).next().unwrap() {
                "pkgbase" | "pkgver" | "maintainer" | "repology" | "source" => Some(line),
                attr if attr.ends_with("sums") => Some(line),
                _ => None,
            }
        })
        .map(Attr::try_from);
    let mut vec = vec![];
    for e in it {
        vec.push(e?);
    }
    Ok(vec)
}
