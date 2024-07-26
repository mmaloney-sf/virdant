set -ex
top="lights"
mkdir -p build/
pushd .. 
cargo run -- --compile "examples/${top}.vir" > demo/build/"${top}.v"
popd
iverilog -I .. build/"${top}.v" testbench.v -o "build/${top}"
"./build/${top}"
