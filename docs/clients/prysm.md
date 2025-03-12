# Prysm

## Installation

Download Prysm beacon chain and validator binaries from the Github release page.

```sh
curl -L https://github.com/prysmaticlabs/prysm/releases/download/v5.3.0/beacon-chain-v5.3.0-linux-amd64 -o prysm
chmod +x prysm
curl -L https://github.com/prysmaticlabs/prysm/releases/download/v5.3.0/validator-v5.3.0-linux-amd64 -o prysm_vc
chmod +x prysm_vc
```

## Configuration

### Prysm beacon chain

- `executable`: Specify path of the prysm beacon chain binary to use. This field is required and there is no default value for it.

### Prysm validator

- `executable`: Specify path of the prysm validator binary to use. This field is required and there is no default value for it.
PATH.
