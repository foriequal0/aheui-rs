use aheui_macro::aheui;

/// 종료 코드를 리턴합니다.
#[aheui]
fn the_answer() -> i32 {
    밦밠따희
}

#[test]
fn test_the_answer() {
    assert_eq!(42, the_answer());
}

/// 표준출력에 문자열을 쓰는 대신 String으로 리턴합니다.
#[aheui]
fn hello_world_string() -> String {
    밤밣따빠밣밟따뿌;
    빠맣파빨받밤뚜뭏;
    돋밬탕빠맣붏두붇;
    볻뫃박발뚷투뭏붖;
    뫃도뫃희멓뭏뭏붘;
    뫃봌토범더벌뿌뚜;
    뽑뽀멓멓더벓뻐뚠;
    뽀덩벐멓뻐덕더벅;
}

#[test]
fn test_hello_world_string() {
    assert_eq!("Hello, world!\n", &hello_world_string());
}

/// 종료 코드와 출력을 동시에 리턴할 수 있습니다.
#[aheui]
fn fibo() -> (i32, String) {
    반반나빠빠쌈다빠망빠쌈삼파싸사빠발발밖따따쟈하처우;
    ㅇㅇㅇㅇㅇㅇ오어어어어어어어어어어어어어어어어어어
}

#[test]
fn test_fibo() {
    let (code, output) = fibo();
    assert_eq!(144, code);
    assert_eq!(&"23581321345589144233", &output);
}

/// 표준 입력 대신에 인자로 입력을 대신할 수 있습니다. 인자 이름이 input인 경우 `input=arg(input)`을 생략할 수 있습니다.
#[aheui(input=arg(input))]
fn codepoint(input: &str) -> String {
    밯망희
}

#[test]
fn test_codepoint() {
    assert_eq!("46663", &codepoint("뙇"));
}

/// `arg()`로 지정된 인자가 없거나, 이름이 `input` 인 인자가 없으면 표준 입력으로 문자열 입력을 받습니다.
#[aheui]
fn codepoint_stdin() -> String {
    밯망희
}

/// 표준 입력을 강제할 수 있습니다.
#[aheui(input=stdin)]
fn codepoint_stdin2() -> String {
    밯망희
}

/// Rust의 토큰화가 맘에 들지 않으면 doc comment, inline doc comment, 문자열, raw 문자열 형식 등으로
/// 감싸 처리할 수 있습니다.
#[aheui(quote=doc_comment)]
fn alt_quote_doc() -> String {
    /*!
    어듀벊벖버범벅벖떠벋벍떠벑번뻐버떠뻐벚벌버더벊벖떠벛벜버버
    _ㅇ　　ㅏㄴㄴㅕㅇ　　ㅎ　　ㅏ　ㅅ　　ㅔ　ㅇ　　ㅛ　　　\0
    _뿌멓더떠떠떠떠더벋떠벌뻐뻐뻐
    붉차밠밪따따다밠밨따따다　박봃
    받빠따따맣반발따맣아희～
    */
}

#[aheui(quote=doc_comment)]
fn alt_quote_comment() -> String {
    //! 어듀벊벖버범벅벖떠벋벍떠벑번뻐버떠뻐벚벌버더벊벖떠벛벜버버
    //! 　ㅇ　　ㅏㄴㄴㅕㅇ　　ㅎ　　ㅏ　ㅅ　　ㅔ　ㅇ　　ㅛ　　　\0
    //! 　뿌멓더떠떠떠떠더벋떠벌뻐뻐뻐
    //! 붉차밠밪따따다밠밨따따다　박봃
    //! 받빠따따맣반발따맣아희～
}

#[aheui(quote=str)]
fn alt_quote_str() -> String {
    "
    어듀벊벖버범벅벖떠벋벍떠벑번뻐버떠뻐벚벌버더벊벖떠벛벜버버
    　ㅇ　　ㅏㄴㄴㅕㅇ　　ㅎ　　ㅏ　ㅅ　　ㅔ　ㅇ　　ㅛ　　　\\0
    　뿌멓더떠떠떠떠더벋떠벌뻐뻐뻐
    붉차밠밪따따다밠밨따따다　박봃
    받빠따따맣반발따맣아희～
    ";
}

#[aheui(quote=str)]
fn alt_quote_raw_str() -> String {
    r"
    어듀벊벖버범벅벖떠벋벍떠벑번뻐버떠뻐벚벌버더벊벖떠벛벜버버
    　ㅇ　　ㅏㄴㄴㅕㅇ　　ㅎ　　ㅏ　ㅅ　　ㅔ　ㅇ　　ㅛ　　　\0
    　뿌멓더떠떠떠떠더벋떠벌뻐뻐뻐
    붉차밠밪따따다밠밨따따다　박봃
    받빠따따맣반발따맣아희～
    ";
}

#[aheui(quote=str)]
fn alt_quote_raw_str2() -> String {
    r#"
    어듀벊벖버범벅벖떠벋벍떠벑번뻐버떠뻐벚벌버더벊벖떠벛벜버버
    　ㅇ　　ㅏㄴㄴㅕㅇ　　ㅎ　　ㅏ　ㅅ　　ㅔ　ㅇ　　ㅛ　　　\0
    　뿌멓더떠떠떠떠더벋떠벌뻐뻐뻐
    붉차밠밪따따다밠밨따따다　박봃
    받빠따따맣반발따맣아희～
    "#;
}

#[test]
fn test_alt_quotes() {
    let funcs = &[
        alt_quote_doc,
        alt_quote_comment,
        alt_quote_str,
        alt_quote_raw_str,
        alt_quote_raw_str2,
    ];
    for func in funcs {
        assert_eq!("안녕하세요?\n", &func());
    }
}
