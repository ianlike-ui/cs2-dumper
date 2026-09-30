// ============================================================================
// schema_metadata_entry_data.rs —— "元数据条目"的内存布局
//
// 【什么是元数据？】
// 字段除了名字/类型/偏移，还可以附加一些"标注"，比如：
//   - MNetworkChangeCallback：网络变量变化时的回调名；
//   - MNetworkVarNames：网络变量名 + 类型名。
// analysis/schemas.rs 的 read_class_binding_metadata 会按名字解释它们。
// 值又是一个联合体（SchemaNetworkValueUnion）：不同标注类型存不同内容。
// ============================================================================

use std::ffi::c_char;

use memflow::prelude::v1::*;

// 一条元数据：名字 + 值
#[derive(Pod)]
#[repr(C)]
pub struct SchemaMetadataEntryData {
    pub name: Pointer64<ReprCString>,                 // 0x0000 元数据名（如 MNetworkVarNames）
    pub network_value: Pointer64<SchemaNetworkValue>, // 0x0008 值
}

// 值的包装（一层间接，方便指针对齐）
#[derive(Pod)]
#[repr(C)]
pub struct SchemaNetworkValue {
    pub value: SchemaNetworkValueUnion, // 0x0000 实际值（联合体）
}

// 联合体：不同类型的元数据存不同内容（字符串指针 / 整数 / 结构体…）
#[repr(C)]
pub union SchemaNetworkValueUnion {
    pub name_ptr: Pointer64<ReprCString>, // 名字指针
    pub int_value: i32,                   // 整数值
    pub float_value: f32,                 // 浮点值
    pub ptr_value: Pointer64<()>,         // 通用指针
    pub var_value: SchemaVarName,         // 变量名（名字 + 类型名）
    pub name_value: [c_char; 32],         // 内嵌 32 字节名字
}

unsafe impl Pod for SchemaNetworkValueUnion {}

// 变量名：名字 + 类型名（MNetworkVarNames 用）
#[derive(Clone, Copy, Pod)]
#[repr(C)]
pub struct SchemaVarName {
    pub name: Pointer64<ReprCString>,      // 0x0000 变量名
    pub type_name: Pointer64<ReprCString>, // 0x0008 类型名
}
