**Upgrade contracts**
****

Required `node.js` and `rust`
1. `git clone --recurse-submodules https://github.com/Mintbase/mintbase-core`
   1. if this fails, repeat process
2. ensure submodules are initialised `git submodule update --init --recursive`
3. default rust stable `rustup default stable`
4. upgrade rust tools `rustup upgrade`
5. download rust wasm tool `rustup target add wasm32-unknown-unknown`
6. open `./integration_tests.sh`
   1. modify `network` variable
   2. modify `root_account` variable
   3. modify `market_account` variable
   4. modify `store_account` variable
7. ensure `~/.root-credentials/<network>` corresponds to `network`
8. ensure you have credentials for `root_account` saved `~/.root-credentials/<network>/<root_account>.json`
9. ensure you have credentials for `market_account` saved `~/.root-credentials/<network>/<market_account>.json`
10. ensure `stores.json` file exists - array of strings
11. run `./integration_tests.sh`
12. `-1` build contracts
13. `1` update `market` and `factory`
14. `13` update all stores in `stores.json`