# To run integration tests

```shell
# Run the Avail node locally in dev mode
./nexus/host/tests/scripts/run_local_avail.sh

cd nexus/host

# To run the host tests
cargo test --test host
# or (If you have nexus cli installed)
nexus_cli test all --dev

# To run e2e integration test with mock geth adapter
cargo test --test integration
# or (If you have nexus cli installed)
nexus_cli test integration --dev
```