## Make weights.rs

### For pallet-template
* ./target/release/node-template benchmark --chain dev --execution=wasm --wasm-execution=compiled --pallet pallet-template --extrinsic do_something --steps 20 --repeat 50 --template=.maintain/frame-weight-template.hbs --output=./pallets/template/src/weights.rs
* [not work]./target/release/node-template benchmark --chain dev --execution=wasm --wasm-execution=compiled --pallet pallet-template --extrinsic * --steps 20 --repeat 50 --template=.maintain/frame-weight-template.hbs --output=./pallets/template/src/weights.rs
  
* ./target/release/node-template benchmark --chain dev --execution=wasm --wasm-execution=compiled --pallet pallet_poe --extrinsic create_claim --steps 20 --repeat 50 --template=.maintain/frame-weight-template.hbs --output=./pallets/poe/src/weights.rs


