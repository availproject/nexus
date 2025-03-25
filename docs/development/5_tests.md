# To run integration tests

```shell
# Run the Avail node locally in dev mode
./integration/scripts/run_local_avail.sh

# To run the host tests
cd nexus/host
cargo test
# or (If you have nexus cli installed)
nexus_cli test host

# To run e2e integration test with mock geth adapter
cd integration
cargo test
# or (If you have nexus cli installed)
nexus_cli test integration
```