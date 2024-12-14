# Provenance Protocol

[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-provenance-rs?logo=docs.rs" height="20">](https://docs.rs/provenance-rs)
[<img alt="crates.io" src="https://img.shields.io/badge/crates.io-provenance-rs?logo=crates.io" height="20">](https://crates.io/crates/provenance-rs)
[<img alt="lib.rs" src="https://img.shields.io/badge/lib.rs-provenance-rs?logo=lib.rs" height="20">](https://lib.rs/crates/provenance-rs)

---

This repository contains the reference implementation for the provenance
protocol, as well as a reference web server for verification of provenance
signatures, and GUI application for easily signing and verifying provenance
of images or text.

The provenance protocol allows anyone to verify the source of an image, PDF, or
piece of text. How this works in practice:

1. A journalist (working for the fictional Foo-Bar Times) takes a photo which
   people might be suspicious of. Maybe it's of a political figure, maybe it's
   a breaking news story, it doesn't matter.
2. The journalist uses the provenance protocol to cryptographically sign the
   image, embedding the signature into the image so that wherever the image
   goes, the signature follows.
3. You, a rationally sceptical citizen, see the image from the journalist and
   at first think that maybe the image is AI generated or otherwise edited.
4. Using the embedded signature, you can confirm two important things:
   1. The image has not been edited in any way since it was signed.
   2. The person who signed the image does indeed work for the Foo-Bar Times

With this information, you can more fully trust the image, because you know for
sure that the Foo-Bar Times has put their reputation behind the image being
true.

Instead of discussing how things happen when everyone's playing nicely, it
might be instructive to discuss what happens when one party tries to be
deceptive.

## What about when someone tries to say an AI image is real?

If you see a weird image online and it doesn't have provenance attached to it,
you should be suspicious. There is no way to know that the image hasn't been
doctored, and if you can't trust whoever is uploading the image, you have no
way of trusting the image.

If the image does have provenance attached, then you can verify the provenance
using the provenance website/desktop application. If the provenance can't be
verified, then you can't trust the image to be real.

If the provenance can be verified, then the website will tell you who signed
the image. It's up to you to decide if you trust the signer, but typically it's
a lot easier to decide if you trust an institution than if you trust some
random social media username.

## Okay, but how do I know if the image has Provenance?

You can upload an image to the desktop application or to the website. Encourage
your favourite social media site to embed provenance into their applications!

## I'm a social media site, how should I integrate Provenance?

Images should be shown with either a moving logo displayed over the image (to
prevent bad actors from just putting the logo into the image) or a static logo
displayed adjacent to the image (in such a way that it could not be mistaken
for being part of the image).

---

AI-generated images, product reviews, and news have made it difficult to
distinguish truth from fiction. The Provenance Protocol allows people to
_delegate_ your trust to a third party.

One problem with signing a document with your private key is that it proves
that you wrote the document, but that's only worth something _if people know
who you are_. If a stranger gives you an unbelievable story, you don't really
care if they signed it or not because you don't trust the person who did the
signing.

This problem can be fixed by hosting the signer's public key on a third-party
server. It makes it possible for previously untrusted individuals to "borrow"
the trustworthiness of larger entities. The individual signs a document with
their private key, and then asks the third party server to host the
individual's public key on their system. The individual then shares the
document on the public internet. Anyone who comes across this document might
not trust the individual, but if they trust the third party then they can query
the third party's servers, receive the individual's public key, and verify that
(1) the individual wrote the document and (2) the third party hosted the
individual's public key and therefore trusts the individual.

This can be used more broadly, as the third party could be social media
websites who automatically generate and host key pairs for their users, and the
individuals could be the users. So you can see an image and know that Joe
Blogs uploaded this photo to `ShareIt.com`.

- The Provenance Protocol allows people to _delegate_ your trust to a third party
- The Provenance Protocol allows good-faith actors to prove that they created a
  document
- The Provenance Protocol allows anyone to prove that they created an
  image/document, thereby making suspicious any images/documents without such proof.
- The Provenance Protocol proves that some URL contains the public key for the
  person who signed the document in question. This is powerful
  - This is equivalent to signing a document with your private key in the
    regular case where you upload your public key to a public key store
  - But it also allows that URL to be a (hopefully authoritative) third party
    who can act as surety for whoever signed the document. If some random
    person came out with an unbelievable photo and signed it with their
    private key, then that's a bit suspicious and them having signed it isn't
    worth much. But if that
    random person went to `$NEWS_ORGANISATION` who independently verified
    their story and

---

good actors should want to sign the things they produce.
give example of this _not_ being a proof of trust, just a list of people you
can point at

- Why is the Provenance protocol better than just signing a document with your
  private key?
  - it works transparently for images using metadata
  - it's embedded in the file itself, no need to keep track of additional
    metadata

Provenance is a protocol for securely specifying the historical ownership of
images, videos, and more. It conclusively tells you whether or not an image was
AI generated.

This is the reference implementation for the provenance protocol.

# What problem does this solve?

Without provenance-rs:

- You see an image of the pope in a sick-ass puffer jacket.
- You didn't know that the pope had so much swag. Life is good.
- Your friend tells you that the image is actually AI generated.
- You are suspicious, and don't know who to believe.
- After much debate, your friend convinces you that the pope does not have that
  much fashion sense.
- Your disappointment is immeasurable and your day is ruined.

With provenance-rs:

- You see an image of the pope in a sick-ass puffer jacket.
- 🚨 The image does not have have any provenance 🚨. If the photo is real, why
  did nobody claim ownership for taking it? The photographer could probably
  have made a lot of money by selling the photo to news websites, so why didn't
  they attach their name to the photo using `provenance`?
- You decide the image is untrustworthy and move on with your day.

# How does it work?

The idea behind the provenance Protocol is that good actors should be happy to
sign their work. A reputable photojournalist should have no problem with
signing a photo that they took before sharing it online, so that everyone knows
who took the photo (and who to ask questions of if the photo looks suspicious).

A bad actor probably wouldn't want to sign their photo, because they would want
to try and pass it off as being from a reputable source. If the bad actor
signed the photo themselves, then everyone would know it's from an
untrustworthy source and would quickly discount it.

If you come across a photo that looks suspicious and isn't signed by anyone,
you should be suspicious. Why would a good actor not claim ownership over the
photo? Only an untrustworthy actor would not sign a photo, so you should treat
the photo with extreme caution, and expect that it's faked in some way.

# But I don't want everyone to know everything I post

That's okay! The internet is anonymous and that's great. For sending funny
photos or memes to friends or just casually posting things online, you don't
really care about verifying that the meme came from a reputable source. So
there's no need to use the provenance protocol.

However, right now there's no way to know where images and videos came from.
While it's not really important that we know who came up with the latest meme
format, it's vitally important that if we see a suspicious or incriminating
photo online, that we be able to tell who took the photo, if they used
Photoshop to edit the photo, and how that photo got shared around the internet.

Misinformation is rampant on the modern internet, and requiring accountability
is one way to combat it.

# How do I use it?

As an end user, you should request that the services you use (social media, new
websites, etc) should use the provenance protocol so that you can know where
your information comes from. If you see a photo or video that doesn't have
provenance, you should be suspicious

# How does it work? (behind the scenes)

The provenance protocol is designed so that multiple layers of ownership are
assigned to one piece of data as it gets modified and shared. At each step, an
organisation or device's private cryptographic key is used to digitally sign
the piece of data. This signature is then passed along with the original data.
Anybody can confirm, by comparing the signature with the organisation or
device's public key, that the data was signed by that organisation/device, and
that the data hasn't been tampered with after signing.

An example might help:

1. John is walking through a forest and leaves his Canon camera with some tigers. One
   of them uses the camera to take a selfie.
2. As the camera takes the selfie, it uses a private key unique to that camera
   to sign the image, and stores the signature together with the image as one
   file. This signature verifies that the photo came from that Canon camera.
3. John is astonished by the photo, but the white balance is off. He uploads
   the file (containing the original photo + the camera's signature) to his
   computer and fixes the white balance with Adobe Illustrator.
4. When he saves the file, Adobe Illustrator signs the edited photo + camera
   signature pair, thus wrapping the provenance that the original photo was
   taken by that specific camera with the provenance that the original photo
   was (in some manner) edited by Adobe Illustrator.
5. John then uploads the signed and edited photos to his favourite social media
   website `www.share-it.com` under his username `@john`. Share-It has assigned
   their user `@john` a public-private key pair, and signs the uploaded image
   with `@john`'s key pair.
6. Nobody believes the photos at first, but the photo has provenance. They can
   look through the provenance and know for a fact that:
   - the user `@john` uploaded the photo to `www.share-it.com`
   - The photo was edited in Adobe Illustrator
   - The photo was taken by a Canon camera
7. Now it's not guaranteed that you trust the user `@john`, Share-It, Adobe, or
   Canon. They might not handle their secrets securely or some other
   vulnerability. But at least now you know who to point fingers at.

Compare the above to what would happen without the provenance protocol. Some
user `@john` would upload an unbelievable photo to Share-It, and everyone would
just have to kinda trust that the photo's not edited? Or do some serious
digital sleuthing to try and figure out if the photo's been doctored.

# How to I implement it? (as a developer)

This rust crate provides the reference implementation of the provenance
protocol. You can use it as a rust library:

```
cargo add provenance-rs
```

Or download the CLI tool:

```
cargo install provenance-rs
```

# Inspiration

The provenance protocol is inspired by [antigen
presentation](https://en.wikipedia.org/wiki/Antigen_presentation), a process
where healthy cells showcase fragments of their internals to passing immune
cells as proof that the healthy cells haven't been taken over by a virus.
Passing immune cells kill any cells that either 1) aren't presenting any
internal fragments or 2) are presenting the wrong sort of internal fragments.

# FAQ

## I don't want my identity to be attached to everything

That's OK! Provenance is not a requirement, and you can always remove the
provenance data or choose not to add it in the first place. Think of it like
signing your name at the end of a document. You don't bother signing every
little grocery note or post-it, but you _intentionally_ sign some things (like
official letters or instructions) so people know who it's coming from.
