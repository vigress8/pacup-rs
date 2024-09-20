use crate::srcinfo::{HashType, SourceEntry};
use anyhow::bail;
use blake2::digest::DynDigest;
use blake2::Digest;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use reqwest::Client;
use std::{fmt, fs::File, io::Write};

fn to_hasher(typ: HashType) -> Box<dyn DynDigest> {
    use HashType::*;
    match typ {
        B2 => Box::new(blake2::Blake2s256::new()),
        MD5 => Box::new(md5::Md5::new()),
        SHA1 => Box::new(sha1::Sha1::new()),
        SHA224 => Box::new(sha2::Sha224::new()),
        SHA256 => Box::new(sha2::Sha256::new()),
        SHA384 => Box::new(sha2::Sha384::new()),
        SHA512 => Box::new(sha2::Sha512::new()),
    }
}

impl SourceEntry<'_> {
    pub async fn download(&self, client: &Client) -> anyhow::Result<()> {
        let mut res = client.get(self.url).send().await?.error_for_status()?;
        let content_len = res.content_length().unwrap_or(0);

        let bar = ProgressBar::new(content_len).with_style(
            ProgressStyle::with_template(
                "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})",
            )?
            .with_key("eta", |state: &ProgressState, w: &mut dyn fmt::Write| {
                write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
            })
            .progress_chars("#>-"),
        );

        let expected_hash = self.hashes.first().map(|hash| hash.value);
        let mut maybe_hasher = self.hashes.first().map(|hash| to_hasher(hash.typ));
        let mut handle = File::options().create(true).write(true).open(&self.dest)?;

        while let Some(chunk) = res.chunk().await? {
            bar.inc(chunk.len() as u64);
            if let Some(ref mut hasher) = maybe_hasher.as_mut() {
                hasher.update(&chunk);
            }
            handle.write_all(&chunk)?;
        }

        bar.finish();
        let final_hash =
            maybe_hasher.map_or(String::new(), |hasher| hex::encode(hasher.finalize()));

        if expected_hash.map_or(true, |hash| hash == final_hash) {
            Ok(())
        } else {
            bail!(
                "Hash mismatch: expected `{}`, got `{}`",
                expected_hash.unwrap(),
                final_hash
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::srcinfo::*;
    use reqwest::Client;

    #[tokio::test]
    async fn test_download() {
        let client = Client::new();
        let entry = SourceEntry {
            url: "https://example.com",
            dest: "example.tmp".into(),
            hashes: vec![HashSum {
                typ: HashType::SHA256,
                value: "ea8fac7c65fb589b0d53560f5251f74f9e9b243478dcb6b3ea79b5e36449c8d9",
            }],
        };
        entry.download(&client).await.unwrap();
    }
}
