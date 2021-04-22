#! /bin/bash
tar --transform "s/^/sgtk\//" -cf sgtk.tar.gz src Cargo.toml .cargo tests ffi/src ffi/Cargo.toml ffi/.cargo f4m/src f4m/Cargo.toml f4m/.cargo geng-search
scp sgtk.tar.gz bt399@login-cpu.hpc.cam.ac.uk:~/toroidal-search

ssh bt399@login-cpu.hpc.cam.ac.uk <<EOF
cd toroidal-search
tar -xf sgtk.tar.gz
cd sgtk/f4m
cargo build --release
cd ../ffi
cargo build --release
cd ../geng-search
make clean
make toroidal-geng
EOF
