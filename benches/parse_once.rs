use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use hex_literal::hex;
use protofish::{context::MessageInfo, prelude::Context};

fn protofish_parse_once_without_parse(data: &[u8], context: &Context, msg: &MessageInfo) {
    let value = msg.decode(data, context);
    assert_eq!(value.fields.len(), 14);
}

fn protofish_parse_once(data: &[u8]) {
    let context = Context::parse(&[r#"
            syntax = "proto3";
            message Message {}
        "#])
    .unwrap();
    let msg = context.get_message("Message").unwrap();
    let value = msg.decode(data, &context);
    assert_eq!(value.fields.len(), 14);
}

fn pb2json_parse_once(data: &[u8]) {
    let parser = protobuf_to_json::Parser::new();
    let v = parser.parse_once(data);
    assert_eq!(v.fields.len(), 14);
}

fn benchmark_parse_once(c: &mut Criterion) {
    let data = hex!(
        "0a0a6173636f6e2d66756c6c120a6173636f6e2d66756c6c1a1b323032352d30392d30325430393a33373a32362e3033393032385a2203302e312a0474657374421b323032352d30392d30325430393a33373a32362e3033393032385a480068007205302e312e308a016e46756c6c204173636f6e20696d706c656d656e746174696f6e202868617368e280913235362c2041454144e280913132382077697468206e6f6e6365206d61736b696e67202620746167207472756e636174696f6e2c20584f46e280913132382c2043584f46e28091313238292e92012368747470733a2f2f6769746875622e636f6d2f6a6a6b756d2f6173636f6e2d66756c6c9a011a68747470733a2f2f646f63732e72732f6173636f6e2d66756c6ca2012368747470733a2f2f6769746875622e636f6d2f6a6a6b756d2f6173636f6e2d66756c6caa014612222f6170692f76312f6372617465732f6173636f6e2d66756c6c2f76657273696f6e731a202f6170692f76312f6372617465732f6173636f6e2d66756c6c2f6f776e657273"
    );

    let context = Context::parse(&[r#"
            syntax = "proto3";
            message Message {}
        "#])
    .unwrap();
    let msg = context.get_message("Message").unwrap();

    let mut group = c.benchmark_group("parse_once");
    group.throughput(Throughput::Bytes(data.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("protofish-without-parse-context", 1),
        &(&data.as_slice(), &context, &msg),
        |b, &(s, c, m)| b.iter(|| protofish_parse_once_without_parse(s, c, m)),
    );
    group.bench_with_input(
        BenchmarkId::new("protofish", 2),
        &data.as_slice(),
        |b, &s| b.iter(|| protofish_parse_once(s)),
    );
    group.bench_with_input(
        BenchmarkId::new("protobuf-to-json", 3),
        &data.as_slice(),
        |b, &s| b.iter(|| pb2json_parse_once(s)),
    );
    group.finish();
}

criterion_group!(benches, benchmark_parse_once);
criterion_main!(benches);
