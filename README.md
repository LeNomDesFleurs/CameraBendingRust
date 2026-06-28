# Nom de plante



## building

install wasm toolchain in your rust compiler :
1. `rustup target add wasm32-unknown-unknown`
*you might have problems here if you installed rust with Homebrew, I recommend using the (rust curl function)[https://rust-lang.org/tools/install/]*
2. `cargo install wasm-pack`
3. `wasm-pack build --target web`
-> `./pkg` will contain the `.wasm` + all the js function needed for the interoperability


Additional info for future me, to make a rust program wasm compilable you need to add
```rust
[lib]
crate-type = ["cdylib", "rlib"]
```
to your `Cargo.toml`, and also a bunch of other stuff if you want to access some DOM properties directly.


Thanks to the [Photon repo](https://github.com/silvia-odwyer/photon/tree/50f4a799adb125d32cc99c2829f0150e73f163aa), one of the few wasm image processing made in rust.
Found a lot of info on the wasm/js scaffolding in this project


