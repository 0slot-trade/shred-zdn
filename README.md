# shred-zdn

The [0slot.trade](https://0slot.trade) Shred Distribution Network (shred-zdn) delivers shreds with exceptional speed. The shred-zdn provides significant benefits to all nodes, including Staked Validators and RPC nodes, Trading Bots, HFTs, Market Makers & DeFi Traders.

## ✨ Features

- 🔌 Connect to the `0slot.trade` Shred ZDN service and receive shreds with exceptional speed
- 🕵️ Sniff local network traffic from your own validator (via interface & TVU port)
- 🚀 Forward shreds to multiple downstream Solana nodes
- 🔒 Authenticated access using ZDN key
- 📊 Built-in stats reporting

## 📦 Build

### Debug build

cargo build

### Release build (optimized for deploy)

cargo r --profile=deploy

## 🚀 Usage

./shred-zdn --help

Receive Shreds from 0slot.trade

Usage: shred-zdn [OPTIONS] --auth <AUTH> --port <PORT> --interface <INTERFACE> --sniffer-port <SNIFFER_PORT> --forwards <FORWARDS>...

Options:
      --auth <AUTH>                  zdn auth key       
      --port <PORT>                  port to receive shreds from 0slot.trade      
      --interface <INTERFACE>        The network interface to sniff for the local validator's traffic (e.g., en0). If the validator is on the same host, use a loopback interface, such as `lo`      
      --sniffer-port <SNIFFER_PORT>  The local validator's shred port to sniff      
      --protocol <PROTOCOL>          Protocol (udp or tcp), case-insensitive [default: udp] [possible values: udp, tcp]      
      --forwards <FORWARDS>...       forward addresses, comma-separated, at lease one      
      --reference <REFERENCE>        reference shred-stream port      
  -h, --help                         Print help  

## Example:

If your Solana validator is running locally:

sudo ./shred-zdn \
  --auth YOUR_AUTH_KEY \
  --port 18888 \
  --interface lo \
  --sniffer-port 8001 \
  --forwards 127.0.0.1:8001

### Note:
Assume shred-zdn is running on the same server with your Solana validator using TVU port 8001 as default, so set the parameter forwards to 127.0.0.1:8001.
To find your Solana TVU port, run solana-validator contact-info (or agave-validator contact-info, depending on the client).

## Required Arguments

| Name             | Description                                                                   |
| ---------------- | ------------------------------------------------------------------------------|
| `--auth`         | ZDN authentication key                                                        |
| `--port`         | Port to receive shreds from 0slot.trade                                       |
| `--interface`    | Network interface to sniff (e.g., `lo`, `en0`, etc.)                          |
| `--sniffer-port` | Local validator's tvu port to sniff (usually `8001`)                          |
| `--forwards`     | List of `ip:tvu-port` targets to forward shreds to (at least one is required) |

## Optional Arguments
| Name          | Description                                            | Default |
| ------------- | ------------------------------------------------------ | ------- |
| `--protocol`  | Transport protocol for shreds from 0slot \[udp or tcp] | udp     |
| `--reference` | Optional reference shred-stream port                   | None    |


## Output
The program prints periodic stats like:

stats: zdn-packet 113380, reference-packet 0, zdn-invalid 0, reference-invalid 0, zdn-first 43651, reference-first 0, forwarded 43650, ms 766.19

## Security
Your Shred ZDN key is required and must be kept private.
Make sure firewall and port configurations are secured on public-facing nodes, and ensure that the port assigned to shred-zdn is properly allowed.

## License
MIT License
