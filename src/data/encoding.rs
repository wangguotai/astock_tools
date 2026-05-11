/// GBK/GB18030 → UTF-8 编码解码工具
///
/// 腾讯实时行情(qt.gtimg.cn)和新浪API返回GBK编码，
/// 必须用 encoding_rs 解码，不能用 reqwest 的 .text() (它假设UTF-8)
use encoding_rs::GBK;

/// 将GBK编码的字节解码为UTF-8字符串
pub fn decode_gbk(bytes: &[u8]) -> String {
    let (decoded, _, _) = GBK.decode(bytes);
    decoded.into_owned()
}
