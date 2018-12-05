#!/usr/bin/env bash

push () {
    pushd $1 >/dev/null 2>&1
}

pop () {
    popd $1 >/dev/null 2>&1
}

mkdir -p src/kvproto

push proto

protoc --rust_out=../src/kvproto kvrpcpb.proto
protoc -I. --rust_out=../src/kvproto --grpc_out=../src/kvproto --plugin=protoc-gen-grpc=`which grpc_rust_plugin` kvpb.proto

pop

push src/kvproto
MOD_RS=`mktemp`
rm -f mod.rs
cat <<EOF > ${MOD_RS}
extern crate futures;
extern crate grpcio;
extern crate protobuf;
extern crate raft;
use raft::eraftpb;

EOF
for file in `ls *.rs`
    do
    base_name=$(basename $file ".rs")
    echo "pub mod $base_name;" >> ${MOD_RS}
done
mv ${MOD_RS} mod.rs
pop

for f in src/kvproto/*; do
python <<EOF
import re
with open("$f") as reader:
    src = reader.read()

res = re.sub('::protobuf::rt::read_proto3_enum_with_unknown_fields_into\(([^,]+), ([^,]+), &mut ([^,]+), [^\)]+\)\?', 'if \\\\1 == ::protobuf::wire_format::WireTypeVarint {\\\\3 = \\\\2.read_enum()?;} else { return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type)); }', src)

with open("$f", "w") as writer:
    writer.write(res)
EOF
done
