RUST_SRC_FILES=$(shell find rust/* -type f | grep -v target)
GO_SRC_FILES=$(shell find golang/* -type f | grep -v go-libp2p-perf)

all: rust/target/release/server rust/target/release/client rust/private.pk8 golang/go-libp2p-perf

rust/target/release/server: $(RUST_SRC_FILES)
	cd rust && cargo build --release --bin server

rust/target/release/client: $(RUST_SRC_FILES)
	cd rust && cargo build --release --bin client

rust/private.pk8:
	openssl genrsa -out rust/private.pem 2048
	openssl pkcs8 -in rust/private.pem -inform PEM -topk8 -out rust/private.pk8 -outform DER -nocrypt
	rm rust/private.pem

golang/go-libp2p-perf: $(GO_SRC_FILES)
	cd golang && go build

clean:
	git clean -Xfd
