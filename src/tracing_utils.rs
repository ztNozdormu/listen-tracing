use bigdecimal::BigDecimal;
use chrono::{NaiveDate, NaiveDateTime};
use serde_json::Value;

/// Option<T> 格式化为 String（ToString 类型）
pub fn fmt_opt<T: ToString>(v: &Option<T>) -> String {
    v.as_ref()
        .map(ToString::to_string)
        .unwrap_or_else(|| "null".to_string())
}

/// Option<T> 格式化为 String，支持自定义默认值
pub fn fmt_opt_or<T: ToString>(v: &Option<T>, default: &str) -> String {
    v.as_ref()
        .map(ToString::to_string)
        .unwrap_or_else(|| default.to_string())
}

/// Option<NaiveDate> 格式化为 YYYY-MM-DD
pub fn fmt_naive_date(v: &Option<NaiveDate>) -> String {
    v.map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "null".to_string())
}

/// Option<NaiveDateTime> 格式化为 YYYY-MM-DD HH:MM:SS
pub fn fmt_naive_datetime(v: &Option<NaiveDateTime>) -> String {
    v.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| "null".to_string())
}

/// Option<BigDecimal> 转换为字符串
pub fn fmt_bigdecimal(v: &Option<BigDecimal>) -> String {
    v.as_ref()
        .map(ToString::to_string)
        .unwrap_or_else(|| "null".to_string())
}

/// Option<serde_json::Value> 转换为字符串
pub fn fmt_json_value(v: &Option<Value>) -> String {
    v.as_ref()
        .map(|v| v.to_string())
        .unwrap_or_else(|| "null".to_string())
}

#[macro_export]
macro_rules! trace_kv {
    ($level:ident, $( $key:expr => $val:expr ),+ $(,)?) => {
        tracing::$level!( $( $key = ?$val ),+ );
    };
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use serde_json::json;
    use crate::setup_tracing;
    use crate::tracing_utils::{fmt_json_value, fmt_naive_date};

    #[tokio::test]
    async fn test_get_coin_data() {
        setup_tracing();

        // 模拟 genesis_date
        let genesis_date = Some(NaiveDate::from_ymd_opt(2020, 5, 1).unwrap());

        // 模拟 categories
        let categories = Some(json!(["DeFi", "Layer 1"]));

        trace_kv!(info,
        "id" => "data_id",
        "symbol" => "BTC",
        "price" => "65000.00",
        "genesis_date" => fmt_naive_date(&genesis_date),
        "categories" => fmt_json_value(&categories),
       );
    }

}

