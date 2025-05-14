@echo off
echo Building Rust code for WASM...
cargo build --target wasm32-unknown-unknown --example simple_game || goto :error

echo Generating WASM/JS bindings...
wasm-bindgen --out-dir pkg --target web --out-name rust_engine target/wasm32-unknown-unknown/debug/examples/simple_game.wasm || goto :error

echo Build successful! Output is in the 'pkg' directory.
goto :eof

:error
echo Build failed!
exit /b 1

:eof
