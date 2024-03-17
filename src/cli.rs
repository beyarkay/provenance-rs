use colored::Colorize;
use std::path::PathBuf;

use anyhow::anyhow;
use clap::{Parser, Subcommand};
use ed25519_dalek::SigningKey;
use provenance_rs::{sign, verify, Base64SigningKey};

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
            let doc = std::fs::read_to_string(document.clone())?;
            let signing_key: SigningKey = Base64SigningKey(signing_key).try_into()?;
            let output = sign(&doc, signing_key, &url);
            std::fs::write(out.clone(), output)?;
            eprintln!(
                "[{}] added provenance to {document:?} {}",
                "Success".green().bold(),
                format!("(output written to {out:?})").dimmed()
            );
        }
        Commands::Verify { path } => {
            let signed_doc = std::fs::read_to_string(path.clone())?;
            match verify(&signed_doc) {
                (Ok(signer_details), _remainder) => {
                    eprintln!(
                        "[{}] '{}' has confirmed authorship of {path:?}",
                        "Success".green().bold(),
                        signer_details.verification_url,
                    );
                }

                (Err(_), _remainder) => {
                    return Err(anyhow!(
                        "[{}] couldn't verify {path:?}",
                        "Failure".red().bold()
                    ))
                }
            }
        }
    };

    Ok(())
}
