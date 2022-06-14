# run a sui devnet fullnode locally

Run a Sui DevNet fullnode locally for testing/experimenting by following the instructions below. This has been tested and should work for:

- linux/amd64
- darwin/amd64 
- darwin/arm64

# prerequisites

- docker / docker compose
    - https://docs.docker.com/get-docker/
    - https://docs.docker.com/compose/install/

# run

Get the latest version of the fullnode config [here](https://github.com/MystenLabs/sui/raw/main/crates/sui-config/data/fullnode-template.yaml), or:

```wget https://github.com/MystenLabs/sui/raw/main/crates/sui-config/data/fullnode-template.yaml```

Get the latest version of the Sui DevNet genesis blob [here](https://github.com/MystenLabs/sui-genesis/raw/main/devnet/genesis.blob), or:

```wget https://github.com/MystenLabs/sui-genesis/raw/main/devnet/genesis.blob```

Update the json-rpc-address and the genesis-file-location in the fullnode config.

- macos:

```sed -i '' -e 's/genesis.blob/\/opt\/sui-node\/genesis.blob/' fullnode-template.yaml```

```sed -i '' -e 's/127.0.0.1/0.0.0.0/' fullnode-template.yaml```

- linux / gnu sed:

```sed -i 's/genesis.blob/\/opt\/sui-node\/genesis.blob/' fullnode-template.yaml```

```sed -i 's/127.0.0.1/0.0.0.0/' fullnode-template.yaml```


Start the fullnode:

```docker-compose up```

# test

Once the fullnode is up and running, test some of the jsonrpc interfaces.

- get the five most recent transactions:

```
curl --location --request POST 'http://127.0.0.1:9000/' \
    --header 'Content-Type: application/json' \
    --data-raw '{ "jsonrpc":"2.0", "id":1, "method":"sui_getRecentTransactions", "params":[5] }'
```

- get detail about a specific transaction:

```
curl --location --request POST 'http://127.0.0.1:9000/' \
    --header 'Content-Type: application/json' \
    --data-raw '{ "jsonrpc":"2.0", "id":1, "method":"sui_getTransaction", "params":["$RECENT_TXN_FROM_ABOVE"] }'
```

# troubleshoot / tips / documentation

Start the fullnode in detached mode:

```docker-compose up -d```

Stop the fullnode using:

```docker-compose stop```

Take everything down, removing the container and volume. Use this to start completely fresh (image, config, or genesis updates):

```docker-compose down --volumes```

Learn more about sui:
- https://docs.sui.io/learn

Learn more about building and running a fullnode natively:
- https://docs.sui.io/build/fullnode

Learn more about docker-compose:
- https://docs.docker.com/compose/gettingstarted/