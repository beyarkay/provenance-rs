use colored::Colorize;
use std::path::PathBuf;

use anyhow::anyhow;
use clap::{Parser, Subcommand};
use ed25519_dalek::SigningKey;
use provenance_rs::{sign, verify_all, Base64SigningKey};

/// Usage:
///
/// $ pvnc sign \
///     --doc <DOCUMENT_IN> \
///     --b64_signature <BASE64_SIGNATURE> \
///     --url <PROVENANCE_URL> \
///     --out <DOCUMENT_OUT>
/// $ pvnc verify <SIGNED_DOCUMENT>
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Add provenance to a document. Requires a (secret) signing key and a provenance url.
    #[clap(alias = "c")]
    Sign {
        /// Document to sign
        #[arg(short = 'd', long)]
        document: PathBuf,
        /// Signing key (base64 encoded)
        // TODO: optionally point to a file with the key in it.
        #[arg(short = 'k', long)]
        signing_key: String,
        /// Provenance URL from which checkers can verify that you signed this document
        #[arg(short = 'u', long)]
        url: String,
        /// Path which the signed document will be written to
        #[arg(short = 'o', long)]
        out: PathBuf,
    },
    /// Verify that a given document has provenance
    #[clap(alias = "v")]
    Verify {
        /// Path of the document to check
        path: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Sign {
            document,
            signing_key,
            url,
            out,
        } => {
            let doc_string = std::fs::read_to_string(document.clone())?;
            let output = sign_string(doc_string, Base64SigningKey(signing_key), &url)?;
            std::fs::write(out.clone(), output)?;
            eprintln!(
                "[{}] added provenance to {document:?} {}",
                "Success".green().bold(),
                format!("(output written to {out:?})").dimmed()
            );
        }
        Commands::Verify { path } => {
            let signed_doc = std::fs::read_to_string(path.clone())?;
            let verifications = verify_all(&signed_doc);
            let num_verified = verifications.0.iter().filter(|v| v.is_ok()).count();
            let total = verifications.0.len();

            if total == 1 {
                if let Ok(signer_details) = &verifications.0[0] {
                    eprintln!(
                        "[{}] '{}' has confirmed authorship of {path:?}",
                        "Success".green().bold(),
                        signer_details.verification_url,
                    );
                } else {
                    eprintln!(
                        "[{}] couldn't verify {path:?} with provenance server",
                        "Failure".red().bold(),
                    )
                }
            } else {
                eprintln!(
                    "[{}] {}/{} ({:.2}%) provenance servers have confirmed authorship of '{}'",
                    "Information".blue().bold(),
                    num_verified,
                    total,
                    (num_verified as f64 / total as f64) * 100.0,
                    path.to_string_lossy(),
                );
                for verification in verifications.0 {
                    if let Ok(signer_details) = verification {
                        eprintln!(
                            "[{}] '{}' has confirmed authorship of {path:?}",
                            "Success".green().bold(),
                            signer_details.verification_url,
                        );
                    } else {
                        eprintln!(
                            "[{}] couldn't verify {path:?} with provenance server",
                            "Failure".red().bold(),
                        )
                    }
                }
            }
            if total == num_verified {
                return Ok(());
            } else {
                return Err(anyhow!(
                    "[{}] Not all provenance was successful",
                    "Failure".red().bold()
                ));
            }
        }
    };

    Ok(())
}

fn sign_string(
    document: String,
    base64_signing_key: Base64SigningKey,
    url: &str,
) -> anyhow::Result<String> {
    let signing_key: SigningKey = base64_signing_key.try_into()?;
    Ok(sign(&document, signing_key, url))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_string_basic() {
        let url = "http://localhost:8000/provenance/beyarkay";
        let document = "Some document that I definitely wrote".to_string();
        let base64_signing_key =
            Base64SigningKey("-5TaFC0xFOj_hf7mlvVaLKKpVFTaXUrLDzRqaaf7gFw=".to_string());

        let signed_string = sign_string(document, base64_signing_key, url).unwrap();
        let provenance_version: &str = env!("CARGO_PKG_VERSION");
        assert_eq!(
            signed_string,
            format!("~~🔏 {provenance_version} http://localhost:8000/provenance/beyarkay 01_e1TwyaDlWnvv7DO9KewhqsfFHP-mAMy74oUwjqB9Vpxa8kHNDg1SRFotz14bIwwws997HICGf2A5Ab98MBg== 🔏~~\nSome document that I definitely wrote")
            );
    }
}
