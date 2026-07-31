//! Dump marshaled transaction bodies as hex, for cross-SDK byte comparison.
use accumulate_client::codec::signing::*;
use serde_json::json;

fn main() {
    let a = json!(["acc://x.acme/book2"]);
    let cases = vec![
        ("createIdentity", json!({"type":"createIdentity","url":"acc://x.acme","keyHash":"aa".repeat(32),"keyBookUrl":"acc://x.acme/book","authorities":a})),
        ("createTokenAccount", json!({"type":"createTokenAccount","url":"acc://x.acme/tok","tokenUrl":"acc://ACME","authorities":a})),
        ("createDataAccount", json!({"type":"createDataAccount","url":"acc://x.acme/d","authorities":a})),
        ("createToken", json!({"type":"createToken","url":"acc://x.acme/t","symbol":"TST","precision":2,"authorities":a})),
        ("createKeyBook", json!({"type":"createKeyBook","url":"acc://x.acme/b2","publicKeyHash":"bb".repeat(32),"authorities":a})),
    ];
    for (name, body) in cases {
        let bytes = accumulate_client::helpers::marshal_body_for_test(&body).unwrap();
        println!("{} {}", name, hex::encode(bytes));
    }
}
