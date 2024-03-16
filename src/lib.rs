//! An AI-proof* history-of-ownership protocol (*Given some reasonable assumptions)
//!
//! Provenance is a simple way for users, companies, or applications to say "yes, I
//! made/edited/created this file". This rust crate provides the reference implementation for the
//! provenance protocol.
//!
//! # Example
//!
//! ```
//! // TODO
//! ```
//!
//! # Why is this useful?
//!
//! In this age of AI-generated content, provenance provides a solution to the question "Is this
//! photo real, or AI?" by giving the creators of unbelievable images/videos the option to prove
//! that they created the video. If an image or video does not have provenance information, then
//! you should be suspicious.
//!
//! # How does this work?
//!
//! Provenance is accomplished by cryptographically signing the file so that you can have proof
//! that the signatory did indeed sign the file, and that the file wasn't modified after being
//! signed. This process is recursive: you can give provenance to a file that already has
//! provenance, allowing for a _history_ of provenance to be attached to a single file. For
//! example: Joe Blogs took the photo, then PhotoShack.app edited the photo, then user joeblogs1999
//! uploaded the photo to instagran.com.
//!
//! Attaching provenance is done via the [`Signatory`] trait. This allows different signing methods
//! to be implemented, for example for different filetypes (signing a PNG is different to signing
//! raw text). It requires that you implement a method of adding a signature to some given bytes
//! ([`Signatory::sign`]), as well as a method of verifying a signature on a given doc
//! ([`Signatory::verify`]).
//!
//!
//! Needed:
//!
//! - A way of listing the signatures on a doc
//! - A way of verifying a signature
//! - A way of signing a doc
//! - A way of getting signatory information from a doc
//!
//!

use anyhow::anyhow;
use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier};

pub enum SigningMethod {
    Text,
}

const PROVENANCE_PREAMBLE: &str = "~~🔏";
const PROVENANCE_POSTAMBLE: &str = "🔏~~";
const PROVENANCE_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default, Debug)]
pub struct SignerDetails {
    pub url: String,
    pub details: String,
}

fn get_signing_key_from_url(url: &str) -> SigningKey {
    // Just use some example bytes for the signing key
    let key_bytes = &[0; 32];
    // FIXME for now, don't actually fetch provenance data from a server and just hardcode the
    // signing key
    match url {
        "https://provenance.twitter.com/beyarkay" => SigningKey::from_bytes(key_bytes),
        &_ => todo!("Other URLs aren't supported yet"),
    }
}

/// Verify that a given document has been signed, and return the signatories details.
///
/// The process for verifying a document has been properly signed is:
///
/// - Extract the provenance version, url, base64-encoded signature, and underlying document from
/// the signed document
/// - decode the signature from base64 into a sequence of bytes
/// - query the URL to get the information about the signer,
///
///
///
pub fn verify(signed_doc: &str) -> anyhow::Result<SignerDetails> {
    let split = signed_doc.split_once('\n');
    let Some((first, doc)) = split else {
        return Err(anyhow!(
            "Document has only one line, therefore cannot be signed"
        ));
    };
    let words = first.split(' ').collect::<Vec<_>>();
    let [preamble, version, url, signature_b64, postamble] = words[..] else {
        return Err(anyhow!(
            "Document doesn't have five space-separated words in first line"
        ));
    };
    if url.is_empty() {
        return Err(anyhow!("URL cannot be empty"));
    }
    if signature_b64.is_empty() {
        return Err(anyhow!("Signature cannot be empty"));
    }
    if preamble != PROVENANCE_PREAMBLE {
        return Err(anyhow!(
            "Document preamble is '{preamble}', not '{PROVENANCE_PREAMBLE}'"
        ));
    }
    if version != PROVENANCE_VERSION {
        return Err(anyhow!(
            "Document version is '{version}', not '{PROVENANCE_VERSION}'"
        ));
    }
    if postamble != PROVENANCE_POSTAMBLE {
        return Err(anyhow!(
            "Document postamble is '{postamble}', not '{PROVENANCE_POSTAMBLE}'"
        ));
    }

    let Ok(signature_vec) = URL_SAFE.decode(signature_b64.as_bytes()) else {
        return Err(anyhow!(
            "Base64 signature '{signature_b64}' couldn't be decoded from Base64 using '=' for padding and '-_' as the extra characters"
        ));
    };

    let Ok(signature) = Signature::from_slice(signature_vec.as_slice()) else {
        return Err(anyhow!(
            "Couldn't convert slice '{:?}' into a Signature",
            signature_vec.as_slice()
        ));
    };

    let verifying_key = get_signing_key_from_url(url).verifying_key();
    if verifying_key.verify(doc.as_bytes(), &signature).is_err() {
        return Err(anyhow!(
            "Document signature '{signature}' could not be verified"
        ));
    }

    Ok(SignerDetails {
        url: url.to_string(),
        details: "".to_string(),
    })
}

pub fn sign(doc: &str, signing_key: SigningKey) -> String {
    let url = "https://provenance.twitter.com/beyarkay";
    let signature = signing_key.sign(doc.as_bytes());
    let encoded_signature = URL_SAFE.encode(signature.to_bytes());
    let doc_with_provenance = format!(
        "{PROVENANCE_PREAMBLE} {PROVENANCE_VERSION} {url} {encoded_signature} {PROVENANCE_POSTAMBLE}\n{doc}",
    );
    doc_with_provenance
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::OsRng;

    #[test]
    fn crypto_sign() {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let _signed = sign("Tsunami warning", signing_key);
    }

    #[test]
    fn verification_fails_if_no_newline() {
        assert!(verify("document text here").is_err());
    }

    #[test]
    fn verification_fails_if_bad_start() {
        assert!(
            verify(format!("<!PROVENANCE_PREAMBLE!> {PROVENANCE_VERSION} url signature {PROVENANCE_POSTAMBLE}\ndocument text here").as_str())
                .is_err()
        );
    }

    #[test]
    fn verification_fails_if_bad_ending() {
        assert!(
            verify(format!("{PROVENANCE_PREAMBLE} {PROVENANCE_VERSION} url signature <!PROVENANCE_POSTAMBLE!>\ndocument text here").as_str())
                .is_err()
        );
    }

    #[test]
    fn verification_fails_if_bad_version() {
        assert!(verify(
            format!("{PROVENANCE_PREAMBLE} <!PROVENANCE_VERSION!> url signature {PROVENANCE_POSTAMBLE}\ndocument text here").as_str(),
        ).is_err());
    }

    #[test]
    fn verification_fails_if_signature_is_empty() {
        assert!(verify(
            format!("{PROVENANCE_PREAMBLE} {PROVENANCE_VERSION} url  {PROVENANCE_POSTAMBLE}\ndocument text here").as_str(),
        ).is_err());
    }

    #[test]
    fn verification_fails_if_url_is_empty() {
        assert!(verify(
            format!("{PROVENANCE_PREAMBLE} {PROVENANCE_VERSION}  signature {PROVENANCE_POSTAMBLE}\ndocument text here").as_str(),
        ).is_err());
    }
    #[test]
    fn verification_fails_if_wrong_number_of_args() {
        assert!(verify("one two three four\ndocument text here").is_err());
    }

    #[test]
    fn verification_fails_during_signature_from_slice() {
        let url = "https://provenance.twitter.com/beyarkay";
        let signature = URL_SAFE.encode("not a valid signature".as_bytes());
        assert!(verify(
            format!("{PROVENANCE_PREAMBLE} {PROVENANCE_VERSION} {url} {signature} {PROVENANCE_POSTAMBLE}\ndocument text here").as_str()
        ).is_err());
    }

    #[test]
    fn verification_fails_during_base64_decoding() {
        let url = "https://provenance.twitter.com/beyarkay";
        let signature = "!exclamations!arent!base64!";
        assert!(verify(
            format!("{PROVENANCE_PREAMBLE} {PROVENANCE_VERSION} {url} {signature} {PROVENANCE_POSTAMBLE}\ndocument text here").as_str()
        ).is_err());
    }

    #[test]
    fn verification_fails_if_bad_key() {
        let url = "https://provenance.twitter.com/beyarkay";
        let doc = "document text here";
        // This randomly generated key won't be the same as the correct key
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let signature = URL_SAFE.encode(signing_key.sign(doc.as_bytes()).to_bytes());
        assert!(verify(
            format!("{PROVENANCE_PREAMBLE} {PROVENANCE_VERSION} {url} {signature} {PROVENANCE_POSTAMBLE}\n{doc}").as_str()
        ).is_err());
    }

    #[test]
    fn verification_fails_if_bad_doc() {
        let url = "https://provenance.twitter.com/beyarkay";
        let doc = "document text here";
        let signing_key = get_signing_key_from_url(url);

        let signature = URL_SAFE.encode(signing_key.sign(doc.as_bytes()).to_bytes());
        let mutated_doc = format!("{doc}and then some extra data");

        assert!(verify(
            format!("{PROVENANCE_PREAMBLE} {PROVENANCE_VERSION} {url} {signature} {PROVENANCE_POSTAMBLE}\n{mutated_doc}").as_str()
        ).is_err());
    }

    #[test]
    fn verification_succeeds() {
        let url = "https://provenance.twitter.com/beyarkay";
        let doc = "document text here";
        let signing_key = get_signing_key_from_url(url);

        let signature = URL_SAFE.encode(signing_key.sign(doc.as_bytes()).to_bytes());

        assert!(verify(
            format!("{PROVENANCE_PREAMBLE} {PROVENANCE_VERSION} {url} {signature} {PROVENANCE_POSTAMBLE}\n{doc}").as_str()
        ).is_ok());
    }
}
