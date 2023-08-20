cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --out-dir .\out\ --target web .\target\wasm32-unknown-unknown\release\roguelike.wasm
xcopy /s/y .\assets .\out\assets