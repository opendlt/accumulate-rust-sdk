//! Golden bytes for `authorities` on every body type that carries it.
//!
//! The authorities field number differs per transaction type. Getting it wrong
//! -- or omitting it while the JSON body still carries it -- makes the locally
//! computed transaction hash disagree with the node's, and the network rejects
//! the transaction as *unsigned*. The symptom points nowhere near the cause, so
//! these bytes are pinned and shared verbatim with the Python, Dart and C# SDKs.
use accumulate_client::helpers::marshal_body_for_test;
use serde_json::json;

fn auth() -> serde_json::Value {
    json!(["acc://x.acme/book2"])
}

fn hex_of(body: &serde_json::Value) -> String {
    hex::encode(marshal_body_for_test(body).expect("marshal"))
}

#[test]
fn create_identity_authorities_field_6() {
    let got = hex_of(&json!({"type":"createIdentity","url":"acc://x.acme",
        "keyHash":"aa".repeat(32),"keyBookUrl":"acc://x.acme/book","authorities":auth()}));
    assert_eq!(got, format!("0101020c6163633a2f2f782e61636d650320{}\
04116163633a2f2f782e61636d652f626f6f6b\
06126163633a2f2f782e61636d652f626f6f6b32", "aa".repeat(32)));
}

#[test]
fn create_token_account_authorities_field_7() {
    let got = hex_of(&json!({"type":"createTokenAccount","url":"acc://x.acme/tok",
        "tokenUrl":"acc://ACME","authorities":auth()}));
    assert_eq!(got, "010202106163633a2f2f782e61636d652f746f6b030a6163633a2f2f41434d45\
07126163633a2f2f782e61636d652f626f6f6b32");
}

#[test]
fn create_data_account_authorities_field_3() {
    let got = hex_of(&json!({"type":"createDataAccount","url":"acc://x.acme/d","authorities":auth()}));
    assert_eq!(got, "0104020e6163633a2f2f782e61636d652f64\
03126163633a2f2f782e61636d652f626f6f6b32");
}

#[test]
fn create_token_authorities_field_9() {
    let got = hex_of(&json!({"type":"createToken","url":"acc://x.acme/t","symbol":"TST",
        "precision":2,"authorities":auth()}));
    assert_eq!(got, "0108020e6163633a2f2f782e61636d652f7404035453540502\
09126163633a2f2f782e61636d652f626f6f6b32");
}

#[test]
fn create_key_book_authorities_field_5() {
    let got = hex_of(&json!({"type":"createKeyBook","url":"acc://x.acme/b2",
        "publicKeyHash":"bb".repeat(32),"authorities":auth()}));
    assert_eq!(got, format!("010d020f6163633a2f2f782e61636d652f62320320{}\
05126163633a2f2f782e61636d652f626f6f6b32", "bb".repeat(32)));
}

#[test]
fn authorities_omitted_when_absent() {
    // A body without authorities must be byte-identical to before the field
    // existed, or every previously signed transaction shape would change.
    let got = hex_of(&json!({"type":"createDataAccount","url":"acc://x.acme/d"}));
    assert_eq!(got, "0104020e6163633a2f2f782e61636d652f64");
}
