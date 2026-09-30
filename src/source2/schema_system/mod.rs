// ============================================================================
// schema_system/mod.rs —— Schema 反射系统子模块的根
//
// 下面每个文件对应 Source 2 Schema 系统里的一类元数据结构的"内存布局"：
//   - schema_system.rs          ：SchemaSystem 单例（入口）
//   - schema_system_type_scope.rs：类型作用域（每个 dll 一个，含类表/枚举表）
//   - schema_class_info_data.rs ：类绑定（一个类的元数据）
//   - schema_class_field_data.rs：类的一个字段
//   - schema_base_class_info_data.rs：基类关系
//   - schema_enum_info_data.rs  ：枚举绑定
//   - schema_enumerator_info_data.rs：枚举的一个成员
//   - schema_metadata_entry_data.rs：字段上的元数据（如网络变量名）
//   - schema_type.rs            ：类型描述
//
// 所有这些结构都从游戏内存里按 C 布局直接读取（#[repr(C)] + #[derive(Pod)]）。
// ============================================================================

pub use schema_base_class_info_data::*;
pub use schema_class_field_data::*;
pub use schema_class_info_data::*;
pub use schema_enum_info_data::*;
pub use schema_enumerator_info_data::*;
pub use schema_metadata_entry_data::*;
pub use schema_system::*;
pub use schema_system_type_scope::*;
pub use schema_type::*;

pub mod schema_base_class_info_data;
pub mod schema_class_field_data;
pub mod schema_class_info_data;
pub mod schema_enum_info_data;
pub mod schema_enumerator_info_data;
pub mod schema_metadata_entry_data;
pub mod schema_system;
pub mod schema_system_type_scope;
pub mod schema_type;
