// Formatter định dạng đầu ra cho CLI (Table, JSON, YAML)
// Đảm bảo dữ liệu JSON/YAML xuất sạch ra stdout để hỗ trợ Automation Scripts

use serde::Serialize;

use crate::args::OutputFormat;

pub struct Formatter;

impl Formatter {
    /// In dữ liệu Serialize ra stdout theo định dạng được yêu cầu
    pub fn print<T: Serialize>(data: &T, format: OutputFormat) {
        match format {
            OutputFormat::Json => {
                let json_str =
                    serde_json::to_string_pretty(data).unwrap_or_else(|_| "{}".to_string());
                println!("{json_str}");
            }
            OutputFormat::Yaml => {
                let yaml_str = serde_yaml::to_string(data).unwrap_or_else(|_| "".to_string());
                print!("{yaml_str}");
            }
            OutputFormat::Table => {
                let json_val = serde_json::to_value(data).unwrap_or(serde_json::Value::Null);
                if let serde_json::Value::String(s) = json_val {
                    println!("{s}");
                } else {
                    let json_str = serde_json::to_string_pretty(data).unwrap_or_default();
                    println!("{json_str}");
                }
            }
        }
    }

    /// In thông báo lỗi ra stderr
    pub fn print_error(msg: &str) {
        eprintln!("Error: {msg}");
    }
}
