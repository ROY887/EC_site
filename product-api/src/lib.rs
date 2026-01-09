pub fn format_price(price: i32) -> String {
    format!("${:.2}", price as f64 / 100.0)
}