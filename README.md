<!-- omit in toc -->
# Bria
The bridge from your applications to the bitcoin network.

Bria enables transaction batching and UTXO management providing the liquidity of on-chain UTXOs to multiple consumers.

<details>
<summary>Table of Contents</summary>

- [Key features](#key-features)
- [Demo video](#demo-video)
- [Quickstart](#quickstart)
  - [Install](#install)
  - [Demo walkthough](#demo-walkthough)
- [Build from source](#build-from-source)
- [Setup](#setup)
  - [Configuration](#configuration)
  - [Bria daemon](#bria-daemon)
  - [Bootstrap](#bootstrap)
  - [Usage](#usage)
- [Developing](#developing)
- [License](#license)

</details>

## Key features
- multi account / multi wallet / multi queue
  - you can configure multiple wallets scoped to an account
  - signing via multiple supported remote signers including feeding PSBTs manually
  - transaction batching via configurable payout queues (check the demo for details)

- cloud ready - intended for use as part of a distributed system
  - designed to be horizontally scalable
  - support for idempotent operations via external IDs
  - embed and update external metadata on addresses and payouts to reference external data
  - globally ordered event sequence can be streamed to achieve guaranteed eventual consistency

- advanced accounting via an embedded ledger
  - internal use of double sided bookkeeping
  - database dump of ledger conforms with accounting best practices
  - great for accountants / auditors to know exactly what is going on

- secure by design
  - extensive automated testing (unit + integration in rust, end-to-end using BATS)
  - all sensitive credentials (like remote signer config) encrypted at rest to prevent db leaks comprimising funds

## Demo video
<a href="https://www.loom.com/share/53e38dc7d1694b11a09b08fc32c584c8">
    <img src="https://cdn.loom.com/sessions/thumbnails/53e38dc7d1694b11a09b08fc32c584c8-1689716086737-with-play.gif" alt="Understanding Bria: Transaction Batching and UTXO Management [🎥]" width="300">
</a>

## Quickstart
### Install
* Requirements on Debian / Ubuntu Linux
  ```
  # git, make, direnv
  sudo apt install git make direnv
  # Docker with the Compose plugin is needed to run the preconfigured environment
  # https://docs.docker.com/desktop/install/linux-install/
  # https://docs.docker.com/compose/install/linux/
  ```
* Download and install compiled release binary
  ```
  # use the latest version from https://github.com/GaloyMoney/bria/releases
  version=0.1.40
  # on linux
  build=unknown-linux-musl
  # on MacOS use:
  # build=apple-darwin

  # download
  wget https://github.com/GaloyMoney/bria/releases/download/${version}/bria-x86_64-${build}-${version}.tar.gz

  # unpack the binary
  tar -xvzf bria-x86_64-${build}-${version}.tar.gz --strip-components 1

  # move the binary to /usr/local/bin
  sudo mv ./bria /usr/local/bin/
  ```
* Download the source code
  ```
  git clone https://github.com/GaloyMoney/bria
  cd bria
  ```
### Demo walkthough
* Start the preconfigured dependencies with Docker Compose
  ```
  docker compose up -d integration-deps
  ```
* Provide a database encryption key
  ```
  export SIGNER_ENCRYPTION_KEY="0000000000000000000000000000000000000000000000000000000000000000"
  ```
* Start the bria daemon with the [default configuration](tests/e2e/bria.local.yml) and bootstrap
  ```
  bria daemon --config ./tests/e2e/bria.local.yml postgres://user:password@127.0.0.1:5432/pg dev
  ```
* Create aliases to work with the docker containers
  ```
  alias bitcoin_cli="docker exec bria-bitcoind-1 bitcoin-cli"
  alias bitcoin_signer_cli="docker exec bria-bitcoind-signer-1 bitcoin-cli"
  ```
* Initialize the local bitcoind on regtest
  ```
  bitcoin_cli createwallet "default"
  bitcoin_cli generatetoaddress 200 "$(bitcoin_cli getnewaddress)"
  ```
* Create a bitcoind wallet using a [sample private descriptor](tests/e2e/bitcoind_signer_descriptors.json)
  ```
  bitcoin_signer_cli createwallet "default"
  bitcoin_signer_cli -rpcwallet=default importdescriptors "$(cat tests/e2e/bitcoind_signer_descriptors.json)"
  ```
* Create a Bria account
  ```
  bria admin create-account --name default
  ```
* Import the wallet used in the signer bitcoind with it's public descriptor
  ```
  bria create-wallet -n default descriptors -d "wpkh([6f2fa1b2/84'/0'/0']tpubDDDDGYiFda8HfJRc2AHFJDxVzzEtBPrKsbh35EaW2UGd5qfzrF2G87ewAgeeRyHEz4iB3kvhAYW1sH6dpLepTkFUzAktumBN8AXeXWE9nd1/0/*)#l6n08zmr" \
      -c "wpkh([6f2fa1b2/84'/0'/0']tpubDDDDGYiFda8HfJRc2AHFJDxVzzEtBPrKsbh35EaW2UGd5qfzrF2G87ewAgeeRyHEz4iB3kvhAYW1sH6dpLepTkFUzAktumBN8AXeXWE9nd1/1/*)#wwkw6htm"
  ```
* Create an address
  ```
  bria new-address -w default --external-id my-id --metadat "{"hello": "world"}"
  ```
* Send funds to the wallet
  ```
  bitcoin_cli -regtest sendtoaddress bcrt1qntvhlxgk8jh0a48w49f3z9edlwhv52zz3j9kw9 1
  ```
* Create a payout queue
  ```
  bria create-payout-queue -n my-queue --tx-priority next-block --interval-trigger 10
  ```
* Submit payouts
  ```
  bria submit-payout -w default --queue-name my-queue --destination bcrt1qxcpz7ytf3nwlhjay4n04nuz8jyg3hl4ud02t9t --amount 100000
  bria submit-payout -w default --queue-name my-queue --destination bcrt1qxcpz7ytf3nwlhjay4n04nuz8jyg3hl4ud02t9t --amount 150000
  ```
* Check the wallet balance and all events with metadata (press CTRL+C t end the stream)
  ```
  bria wallet-balance -w default
  bria watch-events --after 0 --one-shot
  ```
* Check the wallet balance and the events again
  ```
  bria wallet-balance -w default
  bria watch-events --after 0 --one-shot --augment
  ```
* Mine two blocks
  ```
  bitcoin_cli -generate 2
  ```
* Check the wallet balance and all events with metadata (press CTRL+C t end the stream)
  ```
  bria wallet-balance -w default
  bria watch-events --after 0
  ```
* Sign
  ```
  bria set-signer-config \
    --xpub "68bfb290 " bitcoind \
    --endpoint "localhost:18543" \
    --rpc-user "rpcuser" \
    --rpc-password "rpcpassword"
  ```
* Mine two blocks
  ```
  bitcoin_cli -generate 2
  ```
* Check the wallet balance with now completed payouts
  ```
  bria wallet-balance -w default
  ```
* More info in the {Video demo above](#demo-video) and the help of the commands
  ```
  bria --help
  bria <COMMAND> --help
  ```

## Build from source
* Install the Rust toolchain
  ```
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  source "$HOME/.cargo/env"
  ```
* Download the source code
  ```
  git clone https://github.com/GaloyMoney/bria
  ```
* Build
  ```
  cd bria
  make build
  ```
* Add the location of the binary to the PATH
  ```
  PATH="${PATH}:$(pwd)/target/debug"
  ```

## Setup
### Configuration
* Connect to the dependencies following the example in the [docker-compose file](docker-compose.yml)
- Database
  - postgres - to store the internal state - run it locally
- Blockhchain source
  - electrum server - run locally or connect to a public server (e.g., `ssl://electrum.blockstream.info:50002` )
- Signers
  - bitcoind-signer
  - lnd
  - manual PSBT feeding is possible with others
- Fee-estimation
  - mempool.space API - self-hostable or use a public instance (e.g., `https://mempool.space` )
- Telemetry
  - otel-agent - optional for observability

* Example [terraform provider configuration](https://github.com/GaloyMoney/terraform-provider-bria/blob/main/example/main.tf)
* Provide the database connection parameters in environment variables
  ```
  # create a .envrc file
  cat <<EOF > .envrc
  export PG_HOST=127.0.0.1
  export PG_CON=postgres://user:password@${PG_HOST}:5432/pg
  EOF

  direnv allow
  ```
* Provide the database encryption key
  ```
  export SIGNER_ENCRYPTION_KEY="0000000000000000000000000000000000000000000000000000000000000000"
  ```
* Create a minimal config file
  ```
  cat <<EOF > config.yml
  app:
    blockchain:
      network: regtest
      electrum_url: localhost:50001
    fees:
      mempool_space:
        url: http://localhost:8999
  EOF
  ```

### Bria daemon
* start the Bria daemon with the config
  ```
  bria daemon -c config.yml run
  ```

### Bootstrap
* create an admin API key (needed once only - stored in the .bria folder)
  ```
  bria admin bootstrap
  ```

### Usage
* create an account (creates also a profile API key scoped to the account)
  ```
  bria admin create-account -n default
  ```
* create a wallet for the bria account
  ```
  bria  create-wallet --help
  ```
  ```
  Create a wallet from imported xpubs

  Usage: bria create-wallet [OPTIONS] --name <NAME> [API_KEY] <COMMAND>

  Commands:
    wpkh             Initialize the wallet via wpkh
    descriptors      Initialize the wallet via descriptors
    sorted-multisig
    help             Print this message or the help of the given subcommand(s)
  ```
* For further options and subcommands see:
  ```
  bria --help
  bria <COMMAND> --help
  ```

## Developing
For developing all dependencies are run via docker compose

* To run the tests make sure `PG_CON` is pointing to the PG instance inside docker:
  ```
  # create an .envrc file
  cat <<EOF > .envrc
  export PG_HOST=127.0.0.1
  export PG_CON=postgres://user:password@${PG_HOST}:5432/pg
  EOF

  direnv allow
  ```
* Add the dev dependencies
  ```
  make install-dev-deps
  ```
* Run the tests via
  ```
  make reset-deps next-watch
  ```
* For bash based e2e tests we use [bats](https://bats-core.readthedocs.io/en/stable/) as a test runner.
Run the tests via:
  ```
  make e2e
  ```
* If your e2e tests stall and you want to inspect the state (or just want to play around locally) then:
  ```
  make local-daemon
  ```
* Will bring up the daemon and you can run cli commands against it eg:
  ```
  cargo run --bin bria admin list-accounts
  ```

## License
[Mozilla Public License 2.0](LICENSE)
