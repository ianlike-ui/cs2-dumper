// ============================================================================
// schema_enumerator_info_data.rs —— "枚举成员"的内存布局
//
// 描述枚举里的一个成员：名字 + 值。
// 值是一个"联合体"（union）：同一个 8 字节可以按 u8/u16/u32/u64 解释，
// 具体按哪个解释取决于枚举的 size 字段（见 read_enum_binding_members，
// 那里统一按 ulong 读）。
// ============================================================================

use memflow::prelude::v1::*;

use super::SchemaMetadataEntryData;

#[derive(Pod)]
#[repr(C)]
pub struct SchemaEnumeratorInfoData {
    pub name: Pointer64<ReprCString>,                 // 0x0000 成员名
    pub value: SchemaEnumeratorInfoDataUnion,         // 0x0008 成员值（联合体）
    pub metadata_count: i32,                          // 0x0010 元数据数量
    pad_0: [u8; 0x4],                                 // 0x0014 对齐填充
    pub metadata: Pointer64<SchemaMetadataEntryData>, // 0x0018 元数据数组
}

// 联合体：同一段内存按不同整数类型解释（Rust 的 union 类似 C 的 union）
#[repr(C)]
pub union SchemaEnumeratorInfoDataUnion {
    pub uchar: u8,   // 按 1 字节解释
    pub ushort: u16, // 按 2 字节解释
    pub uint: u32,   // 按 4 字节解释
    pub ulong: u64,  // 按 8 字节解释
}

unsafe impl Pod for SchemaEnumeratorInfoDataUnion {}
