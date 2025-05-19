use super::with_regex::regex_preparse;
use super::*;
use bstr::BString;
use pretty_assertions::assert_eq;

use crate::error::{
    EMPTY_CONTENT_LINE, NO_COMMA_ETC, NO_PARAM_NAME, NO_PROPERTY_NAME, NO_PROPERTY_VALUE,
    UNEXPECTED_DOUBLE_QUOTE, UTF8_ERROR,
};

fn equivalent_from_bytes(text: &[u8]) -> Result<Prop, PreparseError> {
    let pre = preparse(text);
    let reg = regex_preparse(text);
    match (pre.is_ok(), reg.is_ok()) {
        (true, true) | (true, false) | (false, true) => {
            assert_eq!(pre, reg, "pre!=reg, text: {text:?}\npre: {pre:#?}\nreg: {reg:#?}");
        }
        (false, false) => {
            let pre = pre.clone().unwrap_err();
            let reg = reg.unwrap_err();
            let ex_pre = pre.segment;
            let rs_pre = pre.reason();
            let ex_reg = reg.segment;
            let rs_reg = pre.reason();
            assert_eq!(
                (ex_pre, rs_pre),
                (ex_reg, rs_reg),
                "pre!=reg, text: {text:?}\npre: {pre:#?}\nreg: {reg:#?}"
            );
        }
    }
    pre
}
fn equivalent(text: &str) -> Result<Prop, PreparseError> {
    let pre = preparse(text.as_bytes());
    let reg = regex_preparse(text.as_bytes());
    match (pre.is_ok(), reg.is_ok()) {
        (true, true) | (true, false) | (false, true) => {
            assert_eq!(pre, reg, "pre!=reg, text: {text}\n{pre:#?}\n{reg:#?}");
        }
        (false, false) => {
            let pre = pre.clone().unwrap_err();
            let reg = reg.unwrap_err();
            let ex_pre = pre.segment;
            let rs_pre = pre.reason();
            let ex_reg = reg.segment;
            let rs_reg = pre.reason();
            assert_eq!(
                (ex_pre, rs_pre),
                (ex_reg, rs_reg),
                "pre!=reg, text: {text}\npre: {pre:#?}\nreg: {reg:#?}"
            );
        }
    }
    pre
}
fn error_for_bytes(text: &[u8]) -> &'static str {
    equivalent_from_bytes(text).unwrap_err().reason()
}

fn error_for(text: &str) -> &'static str {
    equivalent_from_bytes(text.as_bytes()).unwrap_err().reason()
}

fn parse(text: &str) -> StrProp<'_> {
    delocate(&equivalent(text).unwrap())
}

// Test error messages
fn error_is(text: &str, expected: &str) {
    assert_eq!(error_for(text), expected, "text: |{text}|");
}
#[test]
fn property_name_only() {
    error_is("A", NO_PROPERTY_VALUE);
}
#[test]
fn property_name_semicolon_only() {
    error_is("A;", NO_PARAM_NAME);
}
#[test]
fn no_property_value() {
    error_is("A;B=", NO_PROPERTY_VALUE);
    error_is("A;B=c", NO_PROPERTY_VALUE);
}
#[test]
fn quotes_allow_punctuation_in_values() {
    error_is(r#"A;B=",C=:""#, NO_PROPERTY_VALUE);
    error_is(r#"A;B=":C=:""#, NO_PROPERTY_VALUE);
    error_is(r#"A;B=";C=:""#, NO_PROPERTY_VALUE);
}
#[test]
fn forbid_embedded_dquotes() {
    error_is(r#"A;B=ab"c":val"#, UNEXPECTED_DOUBLE_QUOTE);
}
#[test]
fn forbid_space_after_ending_dquote() {
    error_is(r#"A;B="c" ,"d":val"#, NO_COMMA_ETC);
}
#[test]
fn property_name_required() {
    error_is(":foo", NO_PROPERTY_NAME);
    error_is("/foo", NO_PROPERTY_NAME);
}
#[test]
fn forbid_empty_content_line() {
    error_is("", EMPTY_CONTENT_LINE);
}
#[test]
fn value_required() {
    error_is("K", NO_PROPERTY_VALUE);
}
#[test]
fn parameter_name_required() {
    error_is("Foo;=bar:", NO_PARAM_NAME);
    error_is("Foo;/:", NO_PARAM_NAME);
}
#[test]
fn must_be_utf8_len_2() {
    let mut bad = BString::from("FOO:bá");
    //let mut bad = BString::from("abc𒀁");
    let len = bad.len();
    bad[len - 1] = b'a';
    assert_eq!(error_for_bytes(bad.as_slice()), UTF8_ERROR, "text: {:?}", bad);
}
#[test]
fn must_be_utf8_len_4() {
    let mut bad = BString::from("abc𒀁");
    let len = bad.len();
    bad[len - 2] = b'a';
    assert_eq!(error_for_bytes(bad.as_slice()), UTF8_ERROR, "text: {:?}", bad);
}
// Tests for the result returned
#[derive(Debug, PartialEq)]
struct StrParam<'a> {
    name: &'a str,
    values: Vec<&'a str>,
}
#[derive(Debug, PartialEq)]
struct StrProp<'a> {
    name: &'a str,
    parameters: Vec<StrParam<'a>>,
    value: &'a str,
}
fn delocate<'a>(prop: &Prop<'a>) -> StrProp<'a> {
    StrProp {
        name: prop.name.val,
        value: prop.value.val,
        parameters: prop
            .parameters
            .iter()
            .map(|param| StrParam {
                name: param.name.val,
                values: param.values.iter().map(|value| value.val).collect(),
            })
            .collect(),
    }
}
fn as_expected(text: &str, expected: StrProp) {
    assert_eq!(parse(text), expected, "text: |{text}|");
}

#[test]
fn minimal() {
    let text = "-:";
    let expected = StrProp { name: "-", value: "", parameters: Vec::new() };
    as_expected(text, expected);
}
#[test]
fn attach() {
    let text =
        "ATTACH;FMTTYPE=text/plain;ENCODING=BASE64;VALUE=BINARY:VGhlIHF1aWNrIGJyb3duIGZveAo=";
    let expected = StrProp {
        name: "ATTACH",
        value: "VGhlIHF1aWNrIGJyb3duIGZveAo=",
        parameters: vec![
            StrParam { name: "FMTTYPE", values: vec!["text/plain"] },
            StrParam { name: "ENCODING", values: vec!["BASE64"] },
            StrParam { name: "VALUE", values: vec!["BINARY"] },
        ],
    };
    as_expected(text, expected);
}
#[test]
fn vanilla() {
    let text = "FOO;BAR=baz:bex";
    let expected = StrProp {
        name: "FOO",
        value: "bex",
        parameters: vec![StrParam { name: "BAR", values: vec!["baz"] }],
    };
    as_expected(text, expected);
}
#[test]
fn non_ascii() {
    let text = r#"FOO;BAR=íííí,,"óu":béééé"#;
    let expected = StrProp {
        name: "FOO",
        value: "béééé",
        parameters: vec![StrParam { name: "BAR", values: vec!["íííí", "", "óu"] }],
    };
    as_expected(text, expected);
}
#[test]
fn comma_comma_comma() {
    let text = "FOO;BAR=,,,:bex";
    let expected = StrProp {
        name: "FOO",
        value: "bex",
        parameters: vec![StrParam { name: "BAR", values: vec!["", "", "", ""] }],
    };
    as_expected(text, expected);
}
#[test]
fn empty_param_value_list() {
    let text = "FOO;BAR=:bex";
    let expected = StrProp {
        name: "FOO",
        value: "bex",
        parameters: vec![StrParam { name: "BAR", values: vec![""] }],
    };
    as_expected(text, expected);
}

// Comparisons
fn compare(text: &[u8]) {
    let _ = equivalent_from_bytes(text);
}
#[test]
fn two_a() {
    compare("2;a=:".as_bytes());
}
#[test]
fn two_a_quote_lt() {
    compare(r#"2;a="<":"#.as_bytes());
}
#[test]
fn two_a_quote_lt_and_a_trailing_quote() {
    compare(r#"2;a="<":""#.as_bytes());
}
#[test]
fn leading_x7f() {
    compare(b"\x7f");
}
#[test]
fn z_comma() {
    compare("z,".as_bytes());
}
#[test]
fn null_dash() {
    compare(b"\x00-");
}
#[test]
fn z_semi_two() {
    compare("z;2".as_bytes());
}
#[test]
fn unpaired_quote() {
    compare("2;4=\"".as_bytes());
}
#[test]
fn unpaired_quote_bang() {
    compare("2;A=\"!".as_bytes());
}
#[test]
fn zero_255() {
    compare(b"\x00\xFF");
}
#[test]
fn bytes_239_0() {
    compare(b"\xEF\x00");
}
#[test]
fn y_semi_z_semi_ctrl_r() {
    compare(b"y;z=;\x12");
}
#[test]
fn semi_255() {
    compare(b";\xFF");
}
#[test]
fn two_4_equal_tab_ctrl_a() {
    compare(b"2;4=\"\t\x01");
}
#[test]
fn z_quote() {
    compare(b"z\"");
}
#[test]
fn three_z_ux() {
    compare("3zǙ".as_bytes());
}
#[test]
fn six_t_null() {
    compare(b"6:t\0");
}
