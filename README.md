# Sunshine

To run sunshine locally:

```sh
cargo build --release
git clone https://github.com/sunshine-protocol/secrets
docker-compose up
target/release/sunshine-cli key set
```

## Deployment steps

```sh
cargo build --release
target/release/sunshine-node build-spec --chain staging > ./chains/staging.json
scp target/release/sunshine-node sunshine@51.11.244.93:/sunshine/sunshine-node
scp chains/staging.json sunshine@51.11.244.93:/sunshine/chain.json

ssh sunshine@51.11.244.93
> systemctl stop sunshine
> rm -rf /sunshine/db
> systemctl start sunshine
```
