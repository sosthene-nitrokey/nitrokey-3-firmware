RUNNER := runners/lpc55

build:
	make -C $(RUNNER) build

bacon:
	make -C $(RUNNER) bacon

run:
	make -C $(RUNNER) run

jlink:
	scripts/bump-jlink
	JLinkGDBServer -strict -device LPC55S69 -if SWD -vd

mount-fs:
	scripts/fuse-bee

umount-fs:
	scripts/defuse-bee

license.txt:
	cargo run --release --manifest-path utils/collect-license-info/Cargo.toml -- runners/lpc55/Cargo.toml > license.txt

commands.bd:
	cargo run --release --manifest-path utils/gen-commands-bd/Cargo.toml -- \
		runners/embedded/Cargo.toml \
		runners/embedded/profiles/lpc55.toml \
		> $@
