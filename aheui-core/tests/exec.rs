use aheui_core::*;

#[test]
fn test_echo() {
    let code = OwnedCode::parse(r"밯망희");
    let mut input = std::io::Cursor::new("밯");
    let mut output = Vec::new();
    let res = Env::new(&code, &mut input, &mut output).execute();

    assert_eq!(res, 0);
    assert_eq!(std::str::from_utf8(&output), Ok("48175"));
}

#[test]
fn test_bieup_char() {
    let code = OwnedCode::parse(r"밯맣밯맣밯맣밯맣밯맣밯맣희");
    let mut input = std::io::Cursor::new("1+한글😃😄");
    let mut output = Vec::new();
    let res = Env::new(&code, &mut input, &mut output).execute();
    assert_eq!(res, 0);
    assert_eq!(std::str::from_utf8(&output), Ok("1+한글😃😄"));
}

#[test]
fn test_pieup() {
    let code = OwnedCode::parse(
        r"바밟밟땅밝밝땅팡망망우
숭ㅇㅇㅇㅇㅇㅇㅇㅇㅇ어
밟밟밝밝땅땅바팡망망희",
    );
    let mut input = std::io::Cursor::new("");
    let mut output = Vec::new();
    let res = Env::new(&code, &mut input, &mut output).execute();
    assert_eq!(res, 0);
    assert_eq!(std::str::from_utf8(&output), Ok("81494981"));
}

#[test]
fn test_fibo() {
    let code = OwnedCode::parse(
        r"반반나빠빠쌈다빠망빠쌈삼파싸사빠발발밖따따쟈하처우
ㅇㅇㅇㅇㅇㅇ오어어어어어어어어어어어어어어어어어어",
    );
    let mut input = std::io::Cursor::new("");
    let mut output = Vec::new();
    Env::new(&code, &mut input, &mut output).execute();
    assert_eq!(std::str::from_utf8(&output), Ok("23581321345589144233"));
}

#[test]
fn test_helloworld() {
    let code = OwnedCode::parse(
        r"밤밣따빠밣밟따뿌
빠맣파빨받밤뚜뭏
돋밬탕빠맣붏두붇
볻뫃박발뚷투뭏붖
뫃도뫃희멓뭏뭏붘
뫃봌토범더벌뿌뚜
뽑뽀멓멓더벓뻐뚠
뽀덩벐멓뻐덕더벅",
    );
    let mut input = std::io::Cursor::new("");
    let mut output = Vec::new();
    let res = Env::new(&code, &mut input, &mut output).execute();
    assert_eq!(res, 0);
    assert_eq!(std::str::from_utf8(&output), Ok("Hello, world!\n"));
}

#[test]
fn test_helloworld2() {
    let code = OwnedCode::parse(
        r"어듀벊벖버범벅벖떠벋벍떠벑번뻐버떠뻐벚벌버더벊벖떠벛벜버버
　ㅇ　　ㅏㄴㄴㅕㅇ　　ㅎ　　ㅏ　ㅅ　　ㅔ　ㅇ　　ㅛ　　　\0
　뿌멓더떠떠떠떠더벋떠벌뻐뻐뻐
붉차밠밪따따다밠밨따따다　박봃
받빠따따맣반발따맣아희～",
    );
    let mut input = std::io::Cursor::new("");
    let mut output = Vec::new();
    let res = Env::new(&code, &mut input, &mut output).execute();
    assert_eq!(res, 0);
    assert_eq!(std::str::from_utf8(&output), Ok("안녕하세요?\n"));
}
