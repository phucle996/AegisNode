// Strong types cho các Định danh (Identifiers) trong hệ thống
// Tránh việc sử dụng String thô rải rác dễ gây lỗi logic race condition hoặc nhầm lẫn

use serde::{Deserialize, Serialize};
use uuid::Uuid;

macro_rules! define_id {
    ($name:ident, $doc:expr) => {
        #[doc = $doc]
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(pub String);

        impl $name {
            /// Tạo ID mới ngẫu nhiên bằng UUID v4
            pub fn new_v4() -> Self {
                Self(Uuid::new_v4().to_string())
            }

            /// Trả về chuỗi tham chiếu &str
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<String> for $name {
            fn from(s: String) -> Self {
                Self(s)
            }
        }

        impl From<&str> for $name {
            fn from(s: &str) -> Self {
                Self(s.to_string())
            }
        }
    };
}

define_id!(NodeId, "Định danh duy nhất của một Linux Node");
define_id!(PolicyId, "Định danh duy nhất của một Firewall Policy");
define_id!(RuleId, "Định danh duy nhất của một Firewall Rule");
define_id!(SnapshotId, "Định danh bản Snapshot trạng thái nftables");
define_id!(
    ExecutionId,
    "Định danh lượt thực thi Change Plan / Safe Apply"
);
define_id!(BlockEntryId, "Định danh của một entry bị chặn IP");
