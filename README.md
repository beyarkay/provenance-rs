Provenance Protocol
===================

[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-provenance-rs?logo=docs.rs" height="20">](https://docs.rs/provenance-rs)
[<img alt="crates.io" src="https://img.shields.io/badge/crates.io-provenance-rs?logo=crates.io" height="20">](https://crates.io/crates/provenance-rs)
[<img alt="lib.rs" src="https://img.shields.io/badge/lib.rs-provenance-rs?logo=lib.rs" height="20">](https://lib.rs/crates/provenance-rs)

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
protocol. You can `cargo add provenance-rs` to use it as a library.

> TODO: or `cargo install provenance-rs` to install it as a command line tool.

# Inspiration

The provenance protocol is inspired by [antigen
presentation](https://en.wikipedia.org/wiki/Antigen_presentation), a process
where healthy cells showcase fragments of their internals to passing immune
cells as proof that the healthy cells haven't been taken over by a virus.
Passing immune cells kill any cells that either 1) aren't presenting any
internal fragments or 2) are presenting the wrong sort of internal fragments.
