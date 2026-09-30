// ============================================================================
// schema_class_field_data.rs —— "类字段"的内存布局
//
// 【这个结构体是什么】
// 描述一个类的某个字段：名字、类型、以及最重要的——**字段在类实例里的偏移**
// （offset）。比如分析结果显示 C_BasePlayerPawn.m_iHealth 的偏移是 0x34C，
// 外部工具就能去"玩家实例地址 + 0x34C"处读血量。整个 cs2-dumper 的价值
// 很大一部分就体现在这个 offset 字段上。
//
// 【r#type 的 # 是什么意思？】
// Rust 里 type 是关键字，不能直接当标识符用。r#type 表示"我就是要用
// type 这个名字"，让字段可以对应游戏里就叫 type 的成员。
// ============================================================================

use memflow::prelude::v1::*;

use super::{SchemaMetadataEntryData, SchemaType};

#[derive(Pod)]
#[repr(C)]
pub struct SchemaClassFieldData {
    pub name: Pointer64<ReprCString>,                 // 0x0000 字段名
    pub r#type: Pointer64<SchemaType>,                // 0x0008 字段类型描述
    pub offset: i32,                                  // 0x0010 ★字段在类里的字节偏移（最核心）
    pub metadata_count: i32,                          // 0x0014 元数据数量
    pub metadata: Pointer64<SchemaMetadataEntryData>, // 0x0018 元数据数组
}
