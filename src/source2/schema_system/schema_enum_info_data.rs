// ============================================================================
// schema_enum_info_data.rs —— "枚举绑定"的内存布局
//
// 描述一个枚举（如队伍、武器类别等）：名字、所在模块、大小、对齐、
// 成员数量、成员数组、取值范围等。analysis/schemas.rs 的
// read_enum_binding 读的就是这个结构。
// ============================================================================

use memflow::prelude::v1::*;

use super::{SchemaEnumeratorInfoData, SchemaMetadataEntryData, SchemaSystemTypeScope};

// 类型别名：枚举绑定 = 枚举信息数据
pub type SchemaEnumBinding = SchemaEnumInfoData;

#[rustfmt::skip]
#[derive(Pod)]
#[repr(C)]
pub struct SchemaEnumInfoData {
    pub base: Pointer64<SchemaEnumInfoData>,                 // 0x0000 基类信息
    pub name: Pointer64<ReprCString>,                        // 0x0008 枚举名
    pub module_name: Pointer64<ReprCString>,                 // 0x0010 所在模块名
    pub size: u8,                                            // 0x0018 每个枚举值占的字节数
    pub alignment: u8,                                       // 0x0019 对齐
    pub flags: u8,                                           // 0x001A 标志位
    pad_0: [u8; 0x1],                                        // 0x001B 对齐填充
    pub enumerator_count: u16,                               // 0x001C 成员数量
    pub static_metadata_count: u16,                          // 0x001E 静态元数据数量
    pub enumerators: Pointer64<[SchemaEnumeratorInfoData]>,  // 0x0020 成员数组指针
    pub static_metadata: Pointer64<SchemaMetadataEntryData>, // 0x0028 元数据数组指针
    pub type_scope: Pointer64<SchemaSystemTypeScope>,        // 0x0030 所属类型作用域
    pub min_enumerator_value: i64,                           // 0x0038 最小成员值
    pub max_enumerator_value: i64,                           // 0x0040 最大成员值
}
