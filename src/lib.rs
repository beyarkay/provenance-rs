//! A history-of-ownership protocol for securely proving where a document came from.
//!
//! Provenance is a simple way for users, companies, or applications to say "yes, I
//! made/edited/created this file". This makes it easy to know if you can trust an image or
//! document you see online: If it has provenance, then you know who had a hand in creating the
//! document, but if it doesn't have provenance, you should be suspicious and ask what the creator
//! has to hide.
//!
//! This rust crate provides the reference implementation for the provenance protocol.
//!
//! # Example
//!
//! ```
//! use provenance_rs::{sign, verify, Base64SigningKey};
//! use ed25519_dalek::SigningKey;
//!
//! // In reality this would be the server of whomever you're delegating trust. An example
//! // server implementation (which is used for these tests) is available at
//! // https://github.com/beyarkay/provenance-server
//! let username = "beyarkay";
//! let url = format!("http://localhost:8000/provenance/{username}");
//! let doc = "Some document that I definitely wrote";
//! // In reality you'd get the server to generate a keypair for you and give you the (secret)
//! // signing key.
//! let base64_signing_key =
//!     Base64SigningKey("-5TaFC0xFOj_hf7mlvVaLKKpVFTaXUrLDzRqaaf7gFw=".to_string());
//! let signing_key: SigningKey = base64_signing_key.try_into().unwrap();
//!
//! let signed_doc = sign(doc, signing_key, &url);
//!
//! assert!(verify(&signed_doc).0.is_ok());
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
//! Needed:
//!
//! - A way of listing the signatures on a doc
//! - A way of verifying a signature
//! - A way of signing a doc
//! - A way of getting signatory information from a doc

extern crate reqwest;
extern crate serde;
use anyhow::anyhow;
use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub enum SigningMethod {
    Text,
}

const PROVENANCE_PREAMBLE: &str = "~~🔏";
const PROVENANCE_POSTAMBLE: &str = "🔏~~";
const PROVENANCE_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default, Debug)]
pub struct SignerDetails {
    pub verification_url: String,
    pub verification_key: VerifyingKey,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct SignerDetailsFromServer {
    pub verification_url: String,
    pub verification_key_b64: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct KeyDetails {
    pub verification: String,
    pub signing: String,
}

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub struct Username(String);

pub struct Base64Signature(pub String);

impl TryFrom<Base64Signature> for Signature {
    type Error = anyhow::Error;

    fn try_from(base64_signature: Base64Signature) -> Result<Self, Self::Error> {
        // Check that the string inside Base64Signature can be decoded into bytes
        let Ok(bytes_of_base64) = URL_SAFE.decode(base64_signature.0.as_bytes()) else {
            return Err(anyhow!(
                "Couldn't convert {} into bytes",
                base64_signature.0
            ));
        };

        // Check that the decoded bytes are the correct length
        if bytes_of_base64.len() != ed25519_dalek::SIGNATURE_LENGTH {
            return Err(anyhow!(
                "Base64Signature needs to be {} bytes long, but is {} bytes long",
                ed25519_dalek::SIGNATURE_LENGTH,
                bytes_of_base64.len(),
            ));
        }

        // Convert the unknown-length-slice into a known-length-slice
        // This will always succeed because of the length-check above
        let known_length_slice: &[u8; ed25519_dalek::SIGNATURE_LENGTH] =
            bytes_of_base64.as_slice().try_into()?;

        // Convert the slice of bytes into a Signature
        Ok(Signature::from_bytes(known_length_slice))
    }
}

pub struct Base64VerifyingKey(pub String);

impl TryFrom<Base64VerifyingKey> for VerifyingKey {
    type Error = anyhow::Error;

    fn try_from(base64_verifying_key: Base64VerifyingKey) -> Result<Self, Self::Error> {
        // Check that the string inside Base64VerifyingKey can be decoded into bytes
        let Ok(bytes_of_base64) = URL_SAFE.decode(base64_verifying_key.0.as_bytes()) else {
            return Err(anyhow!(
                "Couldn't convert {} into bytes",
                base64_verifying_key.0
            ));
        };

        // Check that the decoded bytes are the correct length
        if bytes_of_base64.len() != ed25519_dalek::SECRET_KEY_LENGTH {
            return Err(anyhow!(
                "Base64VerifyingKey needs to be {} bytes long, but is {} bytes long",
                ed25519_dalek::SECRET_KEY_LENGTH,
                bytes_of_base64.len(),
            ));
        }

        // Convert the unknown-length-slice into a known-length-slice
        // This will always succeed because of the length-check above
        let known_length_slice: &[u8; ed25519_dalek::SECRET_KEY_LENGTH] =
            bytes_of_base64.as_slice().try_into()?;

        // Convert the slice of bytes into a VerifyingKey
        Ok(VerifyingKey::from_bytes(known_length_slice)?)
    }
}

pub struct Base64SigningKey(pub String);

impl TryFrom<Base64SigningKey> for SigningKey {
    type Error = anyhow::Error;

    fn try_from(base64_signing_key: Base64SigningKey) -> Result<Self, Self::Error> {
        // Check that the string inside Base64SigningKey can be decoded into bytes
        let Ok(bytes_of_base64) = URL_SAFE.decode(base64_signing_key.0.as_bytes()) else {
            return Err(anyhow!(
                "Couldn't convert {} into bytes",
                base64_signing_key.0
            ));
        };

        // Check that the decoded bytes are the correct length
        if bytes_of_base64.len() != ed25519_dalek::PUBLIC_KEY_LENGTH {
            return Err(anyhow!(
                "Base64SigningKey needs to be {} bytes long, but is {} bytes long",
                ed25519_dalek::PUBLIC_KEY_LENGTH,
                bytes_of_base64.len(),
            ));
        }

        // Convert the unknown-length-slice into a known-length-slice
        // This will always succeed because of the length-check above
        let known_length_slice: &[u8; ed25519_dalek::PUBLIC_KEY_LENGTH] =
            bytes_of_base64.as_slice().try_into()?;

        // Convert the slice of bytes into a SigningKey
        Ok(SigningKey::from_bytes(known_length_slice))
    }
}

/// Given a provenance endpoint, retrieve the signing key
fn get_verifying_key_from_url(url: &str, client: &Client) -> anyhow::Result<VerifyingKey> {
    // Get the server response
    let response = client.get(url).send()?;
    // Check if it was successful
    if !response.status().is_success() {
        return Err(anyhow!(
            "GET request to {url} failed: {}",
            response.status()
        ));
    }

    // If it was successful, convert the JSON blob into an object
    let signer_details: SignerDetailsFromServer = response.json()?;

    // Convert the object (with a base64-encoded key) into a VerifyingKey object
    Base64VerifyingKey(signer_details.verification_key_b64).try_into()
}

/// Verify that a given document has been signed, and return the signatory's details.
///
/// The process for verifying a document has been properly signed is:
///
/// - Extract the provenance version, url, base64-encoded signature, and underlying document from
///   the signed document
/// - decode the signature from base64 into a sequence of bytes
/// - query the URL to get the information about the signer such as the verification key, username,
///   display name, and details about how the image came to be ("captured", "edited", etc)
/// - use the verification key to verify that the signer did indeed sign the unmodified document
/// - Return the details of the signing and signer.
pub fn verify(signed_doc: &str) -> (anyhow::Result<SignerDetails>, String) {
    let split = signed_doc.split_once('\n');
    let Some((first, doc)) = split else {
        return (
            Err(anyhow!(
                "Document has only one line, therefore cannot be signed"
            )),
            signed_doc.to_string(),
        );
    };
    let words = first.split(' ').collect::<Vec<_>>();
    let [preamble, version, url, signature_b64, postamble] = words[..] else {
        return (
            Err(anyhow!(
                "Document doesn't have five space-separated words in first line"
            )),
            doc.to_string(),
        );
    };
    if url.is_empty() {
        return (Err(anyhow!("URL cannot be empty")), doc.to_string());
    }
    if signature_b64.is_empty() {
        return (Err(anyhow!("Signature cannot be empty")), doc.to_string());
    }
    if preamble != PROVENANCE_PREAMBLE {
        return (
            Err(anyhow!(
                "Document preamble is '{preamble}', not '{PROVENANCE_PREAMBLE}'"
            )),
            doc.to_string(),
        );
    }
    if version != PROVENANCE_VERSION {
        return (
            Err(anyhow!(
                "Document version is '{version}', not '{PROVENANCE_VERSION}'"
            )),
            doc.to_string(),
        );
    }
    if postamble != PROVENANCE_POSTAMBLE {
        return (
            Err(anyhow!(
                "Document postamble is '{postamble}', not '{PROVENANCE_POSTAMBLE}'"
            )),
            doc.to_string(),
        );
    }

    let Ok(signature) = Base64Signature(signature_b64.to_string()).try_into() else {
        return (
            Err(anyhow!(
                "Couldn't convert base64 signature '{signature_b64}' into a signature"
            )),
            doc.to_string(),
        );
    };

    let client = reqwest::blocking::Client::new();

    let Ok(verification_key) = get_verifying_key_from_url(url, &client) else {
        return (
            Err(anyhow!("Couldn't fetch verification key from url '{url}'")),
            doc.to_string(),
        );
    };

    if verification_key.verify(doc.as_bytes(), &signature).is_err() {
        return (
            Err(anyhow!(
                "Document signature '{signature}' could not be verified"
            )),
            doc.to_string(),
        );
    }

    (
        Ok(SignerDetails {
            verification_url: url.to_string(),
            verification_key,
        }),
        doc.to_string(),
    )
}

/// Given a (possibly signed) document, verify all signers of that document.
///
/// This is similar to [`verify`], except it will return *all* signers
pub fn verify_all(signed_doc: &str) -> (Vec<anyhow::Result<SignerDetails>>, String) {
    let mut verifications = vec![];

    let mut doc = signed_doc.to_string();

    loop {
        // Try to verify the provenance of the document
        let verified: (anyhow::Result<SignerDetails>, String) = verify(&doc);
        // println!("Doc is ok? {}: \n```{doc}\n```\n", verified.0.is_ok());

        // If the given document and the returned document have the same number of lines, then
        // there is no signature on the document and we have exhausted all the provenance checking
        // we can do.
        if doc.lines().count() == verified.1.lines().count() {
            break;
        }

        // If this is not the final signer, push the verification and move onto the next one
        verifications.push(verified.0);

        // Now reassign `doc` to whatever the remainder was after verifying the document. This
        // allows one document to be signed multiple times by (potentially different) signers.
        doc = verified.1;
    }

    // Return a vector of all the verifications and the document as was left at the end of it all.
    (verifications, doc.to_string())
}

pub fn sign(doc: &str, signing_key: SigningKey, url: &str) -> String {
    let signature = signing_key.sign(doc.as_bytes());
    let encoded_signature = Base64Signature(URL_SAFE.encode(signature.to_bytes()));

    format_doc(url, encoded_signature, doc)
}

pub fn format_doc(url: &str, encoded_signature: Base64Signature, doc: &str) -> String {
    format!(
        "{PROVENANCE_PREAMBLE} {PROVENANCE_VERSION} {url} {} {PROVENANCE_POSTAMBLE}\n{doc}",
        encoded_signature.0
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::OsRng;
    use rand::Rng;

    fn generate_keys_for_user(
        url: &str,
        username: &Username,
        client: &Client,
    ) -> anyhow::Result<KeyDetails> {
        // Try to generate keys for the given user
        let response = client
            .get(format!("{url}/generate_key/{}", username.0))
            .send()?;

        // Key generation can fail (ie if the user already has keys)
        let Ok(key_details) = response.json() else {
            return Err(anyhow!("Failed to get keys for {}", username.0));
        };

        Ok(key_details)
    }

    #[test]
    fn verification_fails_if_no_newline() {
        assert!(verify("document text here").0.is_err());
    }

    #[test]
    fn verification_fails_if_bad_start() {
        assert!(
            verify(format!("<!PROVENANCE_PREAMBLE!> {PROVENANCE_VERSION} url signature {PROVENANCE_POSTAMBLE}\ndocument text here").as_str())
                .0.is_err()
        );
    }

    #[test]
    fn verification_fails_if_bad_ending() {
        assert!(
            verify(format!("{PROVENANCE_PREAMBLE} {PROVENANCE_VERSION} url signature <!PROVENANCE_POSTAMBLE!>\ndocument text here").as_str())
                .0.is_err()
        );
    }

    #[test]
    fn verification_fails_if_bad_version() {
        assert!(verify(
            format!("{PROVENANCE_PREAMBLE} <!PROVENANCE_VERSION!> url signature {PROVENANCE_POSTAMBLE}\ndocument text here").as_str(),
        ).0.is_err());
    }

    #[test]
    fn verification_fails_if_signature_is_empty() {
        assert!(verify(
            format!("{PROVENANCE_PREAMBLE} {PROVENANCE_VERSION} url  {PROVENANCE_POSTAMBLE}\ndocument text here").as_str(),
        ).0.is_err());
    }

    #[test]
    fn verification_fails_if_url_is_empty() {
        assert!(verify(
            format!("{PROVENANCE_PREAMBLE} {PROVENANCE_VERSION}  signature {PROVENANCE_POSTAMBLE}\ndocument text here").as_str(),
        ).0.is_err());
    }
    #[test]
    fn verification_fails_if_wrong_number_of_args() {
        assert!(verify("one two three four\ndocument text here").0.is_err());
    }

    #[test]
    fn verification_fails_during_signature_from_slice() {
        let url = "http://localhost:8000/provenance/beyarkay";
        let encoded_signature =
            Base64Signature(URL_SAFE.encode("not a valid signature".as_bytes()));
        let doc = "Document text here";

        assert!(verify(format_doc(url, encoded_signature, doc).as_str())
            .0
            .is_err());
    }

    #[test]
    fn verification_fails_during_base64_decoding() {
        let url = "http://localhost:8000/provenance/beyarkay";
        let badly_encoded_signature = Base64Signature("!exclamations!arent!base64!".to_string());
        let doc = "Document text here";

        assert!(
            verify(format_doc(url, badly_encoded_signature, doc).as_str())
                .0
                .is_err()
        );
    }

    #[test]
    fn verification_fails_if_bad_key() {
        let url = "http://localhost:8000/provenance/beyarkay";
        let doc = "document text here";
        // This randomly generated key won't be the same as the correct key for the user beyarkay
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let encoded_signature =
            Base64Signature(URL_SAFE.encode(signing_key.sign(doc.as_bytes()).to_bytes()));

        assert!(verify(format_doc(url, encoded_signature, doc).as_str())
            .0
            .is_err());
    }

    #[test]
    fn verification_fails_if_bad_doc() {
        // Verification should fail if the document was modified after being signed
        let url = "http://localhost:8000/provenance/beyarkay";
        let doc = "document text here";
        let client = reqwest::blocking::Client::new();
        // Generate a new key and retrieve the (signing, verifying) keypair
        let mut random_numbers = OsRng;
        let key_details = generate_keys_for_user(
            "http://localhost:8000",
            &Username(format!("user_{}", random_numbers.gen_range(0..1_000_000))),
            &client,
        )
        .unwrap();

        // Convert the base64 string into a SigningKey object
        let signing_key: SigningKey = Base64SigningKey(key_details.signing).try_into().unwrap();

        // Sign the document
        let signature = signing_key.sign(doc.as_bytes());
        // base64-encode the signature
        let encoded_signature = Base64Signature(URL_SAFE.encode(signature.to_bytes()));

        let mutated_doc = format!("{doc}and then some extra data");

        assert!(
            verify(format_doc(url, encoded_signature, &mutated_doc).as_str())
                .0
                .is_err()
        );
    }

    #[test]
    fn verification_succeeds() {
        let mut random_numbers = OsRng;
        let username = Username(format!("user_{}", random_numbers.gen_range(0..1_000_000)));
        let provenance_url = format!("http://localhost:8000/provenance/{}", username.0);
        let doc = "document text here";
        let client = reqwest::blocking::Client::new();

        // Generate a new keypair
        let key_details =
            generate_keys_for_user("http://localhost:8000", &username, &client).unwrap();
        // convert the base64 signing key to a SigningKey
        let signing_key: SigningKey = Base64SigningKey(key_details.signing).try_into().unwrap();
        // Sign the document
        let signature = signing_key.sign(doc.as_bytes());
        // Base64 encode the signature
        let encoded_signature = Base64Signature(URL_SAFE.encode(signature.to_bytes()));

        assert!(
            verify(format_doc(&provenance_url, encoded_signature, doc).as_str())
                .0
                .is_ok()
        );
    }
    #[test]
    fn multiple_signers() {
        let client = reqwest::blocking::Client::new();

        let mut random_numbers = OsRng;

        let mut usernames: Vec<Username> = (0..10)
            .map(|_| Username(format!("user_{}", random_numbers.gen_range(0..1_000_000))))
            .collect();

        let mut signing_keys: Vec<SigningKey> = usernames
            .iter()
            .map(|username| {
                let key_details =
                    generate_keys_for_user("http://localhost:8000", username, &client).unwrap();
                // convert the base64 signing key to a SigningKey
                Base64SigningKey(key_details.signing).try_into().unwrap()
            })
            .collect();

        let mut doc = "This is the document that's passing through lots of hands".to_string();

        for (signing_key, username) in signing_keys.iter().zip(usernames.iter()) {
            let provenance_url = format!("http://localhost:8000/provenance/{}", username.0);
            // Sign the document
            let signature = signing_key.sign(doc.as_bytes());
            // Base64 encode the signature
            let encoded_signature = Base64Signature(URL_SAFE.encode(signature.to_bytes()));

            doc = format_doc(&provenance_url, encoded_signature, &doc);
            assert!(verify(&doc).0.is_ok());
        }

        usernames.reverse();
        signing_keys.reverse();

        for (signing_key, username) in signing_keys.iter().zip(usernames.iter()) {
            let verified = verify(&doc);

            let details = verified.0.unwrap();
            // We don't use a `let` here so that rustc doesn't think it's a new variable and drop
            // it at the end of the loop iteration
            doc = verified.1;

            assert_eq!(
                details.verification_url,
                format!("http://localhost:8000/provenance/{}", username.0)
            );
            assert_eq!(details.verification_key, signing_key.verifying_key());
        }
    }

    fn generate_users_and_signing_keys(number: u8) -> Vec<(Username, SigningKey)> {
        let client = reqwest::blocking::Client::new();

        let mut random_numbers = OsRng;

        let usernames: Vec<Username> = (0..number)
            .map(|_| Username(format!("user_{}", random_numbers.gen_range(0..1_000_000))))
            .collect();

        let signing_keys: Vec<SigningKey> = usernames
            .iter()
            .map(|username| {
                let key_details =
                    generate_keys_for_user("http://localhost:8000", username, &client).unwrap();
                // convert the base64 signing key to a SigningKey
                Base64SigningKey(key_details.signing).try_into().unwrap()
            })
            .collect();

        usernames.into_iter().zip(signing_keys).collect()
    }

    #[test]
    fn verify_all_works() {
        let (mut usernames, mut signing_keys): (Vec<Username>, Vec<SigningKey>) =
            generate_users_and_signing_keys(10).into_iter().unzip();

        let original_doc = "This is the document that's passing through lots of hands".to_string();
        let mut doc = original_doc.clone();

        for (signing_key, username) in signing_keys.iter().zip(usernames.iter()) {
            let provenance_url = format!("http://localhost:8000/provenance/{}", username.0);
            // Sign the document
            let signature = signing_key.sign(doc.as_bytes());
            // Base64 encode the signature
            let encoded_signature = Base64Signature(URL_SAFE.encode(signature.to_bytes()));

            doc = format_doc(&provenance_url, encoded_signature, &doc);
            assert!(verify(&doc).0.is_ok());
        }

        usernames.reverse();
        signing_keys.reverse();
        let (results, remainder) = verify_all(&doc);

        assert_eq!(remainder, original_doc);

        for ((result, username), key) in results.iter().zip(usernames).zip(signing_keys) {
            let Ok(signer_details) = result else {
                panic!("Result is {result:?} (not OK)")
            };

            assert_eq!(
                signer_details.verification_url,
                format!("http://localhost:8000/provenance/{}", username.0)
            );

            assert_eq!(signer_details.verification_key, key.verifying_key());
        }
    }

    #[test]
    fn verify_all_but_some_are_bad() {
        // FIXME so if signer N in a chain of signers K..N..1 edited the underlying data, we cannot
        // confirm all signers n-1..1 because we can't know for sure what the edit was. Unless we
        // encode the original *and* the edited file, or the diff between them, but then things get
        // really tricky.
        //
        // And if the signer N pretends to not have edited the file, then we'll see all signers
        // N..1  (so including N) fail verification because we will be checking signatures N..1
        // against an edited document.
        let (mut usernames, mut signing_keys): (Vec<Username>, Vec<SigningKey>) =
            generate_users_and_signing_keys(4).into_iter().unzip();

        let original_doc = "This is the document that's passing through lots of hands".to_string();
        let mut doc = original_doc.clone();

        // Hard coded 10 "random" booleans because we don't like flakey tests
        let mut is_mutated = vec![
            false, true, false,
            true, // false, true,
                 // true, //  keep the non-compliant formatting so
                 // false, false, false, true, false, // that it's easy to see there's 10 elements
        ];
        assert_eq!(
            usernames.len(),
            is_mutated.len(),
            "Usernames and is_mutated should be the same length"
        );
        let mutation_string = " got mutated!".to_string();

        // println!("-- START SIGNING --");
        let iterator = signing_keys
            .iter()
            .zip(usernames.iter())
            .zip(is_mutated.iter());
        for ((signing_key, username), mutate) in iterator {
            // println!("{username:?} mutates?: {mutate}");
            let provenance_url = format!("http://localhost:8000/provenance/{}", username.0);
            // Sign the document
            let signature = signing_key.sign(doc.as_bytes());
            // Base64 encode the signature
            let encoded_signature = Base64Signature(URL_SAFE.encode(signature.to_bytes()));

            if *mutate {
                doc = format_doc(
                    &provenance_url,
                    encoded_signature,
                    &format!("{doc}{mutation_string}"),
                );
            } else {
                doc = format_doc(&provenance_url, encoded_signature, &doc);
            }
            // println!("mutated?: {mutate} Doc is:\n```\n{doc}\n```");

            if *mutate {
                assert!(verify(&doc).0.is_err());
            } else {
                assert!(verify(&doc).0.is_ok());
            }
            // println!("verification: {:?}\n", verify(&doc).0);
        }

        // Reverse the vectors since we verify in the opposite order to which we sign
        usernames.reverse();
        is_mutated.reverse();
        signing_keys.reverse();
        // println!("-- START VERIFICATION --");

        // Actually do the verification
        let (results, _remainder) = verify_all(&doc);

        let iterator = results
            .iter()
            .zip(usernames)
            .zip(signing_keys)
            .zip(is_mutated);

        // println!("-- START CHECKING THE VERIFICATION --");
        let mut doc_has_been_mutated = false;
        for (((result, username), key), mutated) in iterator {
            // println!(
            //     "{} mutated?: {mutated} {username:?}, {result:?}",
            //     if mutated == result.is_err() {
            //         "good"
            //     } else {
            //         "bad"
            //     }
            // );
            // continue;
            doc_has_been_mutated = doc_has_been_mutated || mutated;

            if doc_has_been_mutated {
                // println!("{username:?} is err");
                assert!(result.is_err())
            } else {
                // println!("{username:?} is ok");
                let Ok(signer_details) = result else {
                    panic!("Result is {result:?} (not Ok)")
                };

                assert_eq!(
                    signer_details.verification_url,
                    format!("http://localhost:8000/provenance/{}", username.0)
                );

                assert_eq!(signer_details.verification_key, key.verifying_key());
            }
        }
    }

    #[test]
    fn docstring_test() {
        use crate::sign;
        use ed25519_dalek::SigningKey;

        // In reality this would be the server of whomever you're delegating trust. An example
        // server implementation (which is used for these tests) is available at
        // https://github.com/beyarkay/provenance-server
        let url = "http://localhost:8000/provenance/beyarkay";
        let doc = "Some document that I definitely wrote";
        let base64_signing_key =
            Base64SigningKey("-5TaFC0xFOj_hf7mlvVaLKKpVFTaXUrLDzRqaaf7gFw=".to_string());
        let signing_key: SigningKey = base64_signing_key.try_into().unwrap();

        let _signed_doc = sign(doc, signing_key, url);
    }
}
