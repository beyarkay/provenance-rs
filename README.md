# Provenance Protocol

[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-provenance-rs?logo=docs.rs" height="20">](https://docs.rs/provenance-rs)
[<img alt="crates.io" src="https://img.shields.io/badge/crates.io-provenance-rs?logo=crates.io" height="20">](https://crates.io/crates/provenance-rs)
[<img alt="lib.rs" src="https://img.shields.io/badge/lib.rs-provenance-rs?logo=lib.rs" height="20">](https://lib.rs/crates/provenance-rs)

---

> NOTE: Provenance is a work in progress, please see the [whitepaper](https://github.com/beyarkay/provenance-rs/blob/main/Provenance%20Protocol%20Whitepaper.pdf) for
> details about what it will become.

The provenance protocol allows anyone to verify the source of an image, PDF, or
piece of text, thereby making AI-generated images nearly impossible to pass off
as real.

This repository contains

1. The reference implementation for the provenance protocol
2. The reference web server for verification of provenance signatures
3. A GUI application for easily signing and verifying provenance of images or
   text.

As an example, this is how the provenance protocol would be used:

1. A journalist (working for the fictional Foo-Bar Times) takes a photo which
   people might be suspicious of. Maybe it's of a political figure, maybe it's
   a breaking news story, maybe it's of a celebrity.
2. The journalist uses the provenance protocol to cryptographically sign the
   image, embedding this signature into the image so that wherever the image
   goes, the signature follows.
3. You, a sceptical citizen, see the image from the journalist and at suspect
   that the image is AI generated.
4. Since the image used the provenance protocol to embed a signature into the
   image, you can verify two important things:
   1. The image has not been edited in any way since it was signed.
   2. The person who signed the image does indeed work for the Foo-Bar Times

With this information, you can more fully trust the image, because you know for
sure that the Foo-Bar Times has put their reputation behind the image being
true.

It is important to note that the protocol cannot _prove_ an image isn't AI
generated, it can only prove that the given image came from a certain source.
The user still has to trust the source at the end of the day. However, trusting
a handful of outlets is a lot easier than trusting every username on the
internet.

## Inspiration

The provenance protocol is inspired by [antigen
presentation](https://en.wikipedia.org/wiki/Antigen_presentation), a process
where healthy cells showcase fragments of their internals to passing immune
cells as proof that the healthy cells haven't been taken over by a virus.
Passing immune cells kill any cells that either 1) aren't presenting any
internal fragments or 2) are presenting the wrong sort of internal fragments.
