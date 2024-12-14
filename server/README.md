# (insecure) Provenance Server

This crate is an example server that provide provenance information for the
provenance protocol. It is not secure, and will happily send you private keys
in plain text. It's purpose is to provide an example implementation of a server
that can generate key pairs for a given user, and provide the public key for a
given user on request.

You can start it up by running

```
cargo run
```

You can (insecurely) ask the server to generate a key for a given user using:

```
$ curl http://localhost:8000/generate_key/my_username
{
  "verification": "hKYtxMDjaZ1UDnNsETXiygEs_nVPkd1DPmcXgajEaFY=",
  "signing": "L1Bke2467a4upIbpEnqydEKVG9T7IFgJmvWPcAe4NBw="
}
```

You can ask for the public key for a given user like so:

```
$ curl http://localhost:8000/provenance/my_username
{
  "verification_url": "http://127.0.0.1:8000/my_username/provenance",
  "verification_key": "hKYtxMDjaZ1UDnNsETXiygEs_nVPkd1DPmcXgajEaFY=",
  "metadata": {
    "username": "my_username"
  }
}
```

You can then use the verification key to verify that the user `my_username`
signed any document that has provenance.
