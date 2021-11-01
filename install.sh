
cargo build --release

if [ $CARGO_TARGET_DIR ];then
	sudo cp CARGO_TARGET_DIR/targets/release/c0i /usr/bin
else
	sudo cp ./targets/release/c0i /usr/bin
fi