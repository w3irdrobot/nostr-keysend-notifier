# nostr-keysend-notifier

Get notified of keysend messages to your LND node in your nostr DMs!

## Description

Lightning nodes have the ability to receive keysend payments. They are essentially payments pushed to a receiving node from a sending node without the receiving node needing to make an invoice ahead of time. This is done by passing the preimage in a custom record that the receving node can use to unlock the funds. Because there is more space in a message to pass more custom records, there is [TLV registry](https://github.com/satoshisstream/satoshis.stream/blob/main/TLV_registry.md) that has somewhat formalized a set of other custom records for passing around more data, including chat-like messages!

The issue is most people don't even know they are receiving these messages. With the re-emergence of lightning-based advertising like [satograms](https://satogram.xyz/), there is a renewed interet in this form of message passing. To bring these more into the open, this simple project, when run, will listen for new chat message keysends and forward them on to a configured nostr pubkey as a way of making them more visible in the nostr client of your choice.

## Running

To run the project, until there are some binaries built, requires compiling manually using cargo, which can be easily installed using [rustup](https://rustup.rs/).

There is assumed to be a config in the root of the project called `config.toml`. There is an example in the repo for getting up and running quickly.

Once the config is setup correctly, just run the project.

```shell
cargo run
```

## Macaroon

An invoice macaroon has enough permissions to make this work. To bake a new one:

```shell
lncli bakemacaroon invoices:read --save_to invoices_read.macaroon
```
