# Silkroad Packet Decryptor

This tool allows decrypting packet streams that have been captured from Silkroad Online. In Silkroad Online, packets
sent from the client are almost always encrypted as well as a few selected packets from the server. There are already
some tools in existence that place themselves between client and server, performing a Man-in-the-Middle attack, to
extract the decrypted raw packet data. While this approach is quite efficient and can support multiple client instances,
it also has a downside. These tools often require a user to modify the Silkroad Online program to use the local tool as
the endpoint or proxy, either beforehand or by running it through a so-called "loader".

With this tool, I propose an alternative for analyzing encrypted packets. This tool allows a user to capture the packet
stream using conventional tools, such as wireshark, run the game normally and then later decrypt the packets. To achieve
this, the tool takes advantage of the key exchange having only a small amount of entropy that is kept secret. Anyone who
could witness the key exchange could feasibly brute force the secret part and thus recreate the secret key material
after the fact. The only requirement for this technique to work is that the handshake/key exchange needs to be witnessed
and thus included in the captured packet stream.

## Usage

This tool is kept rather simple. The main operation performed by this tool, is taking a captured packet stream in `pcap`
format, decrypts the contained packets, and writes them to a new file with a `-decrypted` suffix in the name. To run the
tool in this manner, use the following:

```shell
silkroad-packet-decryptor /path/to/file.pcap
```

This will only decrypt the traffic with the login/gateway server, as it does not analyze the stream to figure out the
used agent/game server. However, if the port is know, it can be passed as another argument and the tool will decrypt the
stream as well. If the game server used the port `22233`, the command would look as follows:

```shell
silkroad-packet-decryptor /path/to/file.pcap 22233
```

As the tool does a brute force attack on the key exchange, the more processing power it has available, the faster it can
break the missing key material. By default, the tool uses as many threads as there are physical cores available.
However, modern CPUs often allow multiple threads to be executed on the same physical core. The tool can thus be
instructed to use more threads, by passing the `--threads` flag with the suggested number of threads to use. It is
advised to not use more than twice as many threads as there are cores, due to the resulting switching overhead between
threads. If the tool should use `10` threads, it could be invoked like this:

```shell
silkroad-packet-decryptor --threads 10 /path/to/file.pcap 22233
```

To display a short help to list all these options, the `--help` flag can be provided.

## Why it works

When examining the encryption using the protocol by Silkroad Online, a blowfish cypher is used. As this is a symmetric
block cypher, it requires that a key is shared between the two parties beforehand. This is often done through an
asymmetric key exchange, where two parties create a public and private key material, exchange the public part and
combine that with their private part to have a shared secret. Silkroad does this as well, however, only a small part of
the full data is actually kept secret. In other sources that explain how the security works, this part is often called
`x` and is used to derive a public value `a`. The other two parts that influence `a` are called `g` and `p` which are
both publicly shared. Thus, to generate the secret key using the public parts `g`, `p`, and `a`, we only need to brute
force `x`. `x` is a random 32bit number, however, only 31 bits are actually used. The operation performed to derive `a`
is multiplication with modulo, making it impossible to calculate the inverse, but is simple enough to run a brute force
attack. This effectively checks every number from 0 to 2^31 if, together with `p` and `g`, the result matches `a`. Often
this only applies to a single number, but can sometimes result in multiple possible matches. In that case, we consult
the client challenge material which allows us to verify which of the candidates was used by the client. With the
verified match, we now have everything that made up the state of the server and can thus recreate the shared secret key
used in the exchange.

On modern systems, this may only take a minute or two to go through 2^31 - or ~2*10^9 - possibilities and is thus quite
acceptable for hte breaking speed. This could be improved through GPU acceleration, but for the presented use case is
enough to satisfy.