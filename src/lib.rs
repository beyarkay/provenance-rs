//! An AI-proof* history-of-ownership protocol (*Given some reasonable assumptions)
//!
//! Provenance is a simple way for users, companies, or applications to say "yes, I
//! made/edited/created this file". This rust crate provides the reference implementation for the
//! provenance protocol.
//!
//! # Example
//!
//! ```
//! use provenance_rs::{Signatory, TextSignatory};
//! // Some text to sign
//! let text = "Example text";
//!
//! // Signing the text
//!
//! let mut text_signer = TextSignatory::new();
//! let signed_text = text_signer.sign(text.to_string());
//! assert_eq!(
//!     signed_text,
//!     "=🔏0.1.0=\nExample text",
//! );
//!
//! // Verifying that the signed text came from the right place
//! assert!(text_signer.verify(&signed_text));
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

use anyhow::anyhow;
use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};

pub enum SigningMethod {
    Text,
}

const PROVENANCE_PREAMBLE: &str = "~~🔏";
const PROVENANCE_POSTAMBLE: &str = "🔏~~";
const PROVENANCE_VERSION: &str = env!("CARGO_PKG_VERSION");

fn get_verifying_key_from_url(url: &str) -> VerifyingKey {
    let mut csprng = rand::rngs::OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    signing_key.verifying_key()
}

#[derive(Default)]
pub struct SignatoryDetails {
    pub url: String,
    pub details: String,
}

pub fn verify(signed_doc: &str) -> anyhow::Result<SignatoryDetails> {
    let split = signed_doc.split_once('\n');
    let Some((first, doc)) = split else {
        return Err(anyhow!(
            "Document has only one line, therefore cannot be signed"
        ));
    };
    let words = first.split(' ').collect::<Vec<_>>();
    let [preamble, version, url, signature_b64, postamble] = words[..] else {
        return Err(anyhow!(
            "Document doens't have five space-separated words in first line"
        ));
    };
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

    let binding = URL_SAFE.decode(signature_b64.as_bytes()).unwrap();
    let signature = Signature::from_slice(binding.as_slice()).unwrap();

    let verifying_key = get_verifying_key_from_url(url);
    if verifying_key.verify(doc.as_bytes(), &signature).is_err() {
        return Err(anyhow!(
            "Document signature '{signature}' could not be verified"
        ));
    }

    Ok(SignatoryDetails {
        url: url.to_string(),
        details: "".to_string(),
    })

    // assert!(signing_key.verify(doc.as_bytes(), &signature).is_ok());

    // println!("verifyingKey:\n{:?}", verifying_key.as_bytes());
    // assert!(verifying_key.verify(doc.as_bytes(), &signature).is_ok());

    // let encoded_signature = URL_SAFE.encode(signature.to_bytes());
    // println!("Signature base64 {}", encoded_signature);

    // let doc_with_provenance = format!(
    //     "{PROVENANCE_PREAMBLE}{PROVENANCE_VERSION} {url} {encoded_signature} {PROVENANCE_POSTAMBLE}\n{doc}",
    // );
    // true
}
pub fn sign(doc: &str, signing_key: SigningKey) -> String {
    let url = "https://provenance.twitter.com/beyarkay";
    // println!("SigningKey:\n{:?}", signing_key.as_bytes());

    let signature = signing_key.sign(doc.as_bytes());
    // println!("Signature:\n{:?}", signature.to_bytes());

    // assert!(signing_key.verify(doc.as_bytes(), &signature).is_ok());

    // let verifying_key = signing_key.verifying_key();

    // println!("verifyingKey:\n{:?}", verifying_key.as_bytes());
    // assert!(verifying_key.verify(doc.as_bytes(), &signature).is_ok());

    let encoded_signature = URL_SAFE.encode(signature.to_bytes());
    // println!("Signature base64 {}", encoded_signature);

    let doc_with_provenance = format!(
        "{PROVENANCE_PREAMBLE} {PROVENANCE_VERSION} {url} {encoded_signature} {PROVENANCE_POSTAMBLE}\n{doc}",
    );
    println!("{}", doc_with_provenance);
    doc_with_provenance
}

pub trait Signatory {
    type Document;
    type Signature;

    fn sign(&mut self, document: Self::Document) -> Self::Document;
    fn verify(&mut self, document: &Self::Document) -> bool;
    fn list_signatures(
        &mut self,
        document: Self::Document,
    ) -> (Vec<Self::Signature>, Self::Document);
}

#[derive(Default)]
pub struct TextSignatory;

impl TextSignatory {
    pub fn new() -> Self {
        TextSignatory {}
    }
}

impl Signatory for TextSignatory {
    type Document = String;
    type Signature = String;

    fn sign(&mut self, document: Self::Document) -> Self::Document {
        format!("=🔏0.1.0=\n{document}")
    }

    fn verify(&mut self, document: &Self::Document) -> bool {
        document.starts_with("=🔏0.1.0=\n")
    }

    fn list_signatures(
        &mut self,
        document: Self::Document,
    ) -> (Vec<Self::Signature>, Self::Document) {
        let mut signatures = document
            .split_inclusive("=🔏0.1.0=\n")
            .map(|s| s.to_owned())
            .collect::<Vec<String>>();

        let doc = signatures
            .split_off(signatures.len() - 1)
            .first()
            .unwrap()
            .to_owned();

        (signatures, doc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::OsRng;

    #[test]
    fn doctest() {
        // Some text to sign
        let text = "Example text";
        // Signing the text
        let mut text_signer = TextSignatory::new();
        let signed_text = text_signer.sign(text.to_string());
        assert_eq!(signed_text, "=🔏0.1.0=\nExample text");
        // Verifying that the signed text came from the right place
        assert!(text_signer.verify(&signed_text));
    }

    #[test]
    fn multi_sign() {
        // Some text to sign
        let text = "Example text";
        // Signing the text
        let mut text_signer = TextSignatory::new();
        let signed_text = text_signer.sign(text.to_string());
        let signed_text = text_signer.sign(signed_text.to_string());

        let signed_text = text_signer.sign(signed_text.to_string());

        assert_eq!(signed_text, "=🔏0.1.0=\n=🔏0.1.0=\n=🔏0.1.0=\nExample text");
        // Verifying that the signed text came from the right place
        assert!(text_signer.verify(&signed_text));

        // List all the signatures
        let (signatures, doc) = text_signer.list_signatures(signed_text);

        // Assert the signatures were collected
        assert_eq!(
            signatures,
            vec!["=🔏0.1.0=\n", "=🔏0.1.0=\n", "=🔏0.1.0=\n"],
        );
        // Assert the oroginal document was collected
        assert_eq!(doc, "Example text");
    }

    #[test]
    fn crypto_sign() {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let _signed = sign("Tsunami warning", signing_key);
    }

    #[test]
    fn verifification_fails_if_no_newline() {
        assert!(verify("document text here").is_err());
    }

    #[test]
    fn verifification_fails_if_not_starting_correctly() {
        assert!(
            verify(format!("<!PROVENANCE_PREAMBLE!> {PROVENANCE_VERSION} url signature {PROVENANCE_POSTAMBLE}\ndocument text here").as_str())
                .is_err()
        );
    }

    #[test]
    fn verifification_fails_if_not_ending_correctly() {
        assert!(
            verify(format!("{PROVENANCE_PREAMBLE} {PROVENANCE_VERSION} url signature <!PROVENANCE_POSTAMBLE!>\ndocument text here").as_str())
                .is_err()
        );
    }

    #[test]
    fn verifification_fails_if_bad_version() {
        assert!(verify(
            format!("{PROVENANCE_PREAMBLE} <!PROVENANCE_VERSION!> url signature {PROVENANCE_POSTAMBLE}\ndocument text here").as_str(),
        ).is_err());
    }

    #[test]
    fn verifification_fails_if_wrong_number_of_args() {
        assert!(verify("one two three four\ndocument text here").is_err());
    }
}
