// ============================================================================
// schema_system.rs —— SchemaSystem 单例的内存布局（Schema 分析的"入口"）
//
// 【这个结构体是什么】
// 引擎全局只有一个 SchemaSystem 实例，它持有所有"类型作用域"的动态数组
// （type_scopes）和注册数量。analysis/schemas.rs 的 read_schema_system
// 用特征码找到它，然后从这里出发遍历整个类表。
//
// 【字段都是干什么的】
//   - type_scopes：所有类型作用域的数组（每个 dll 一个）；
//   - registration_count：注册的类/枚举总数（0 说明还没初始化/地址不对）。
// ============================================================================

use memflow::prelude::v1::*;

use super::SchemaSystemTypeScope;

use crate::source2::UtlVector;

// 游戏内存里的 SchemaSystem 结构（字段后的注释是相对偏移）
#[repr(C)]
pub struct SchemaSystem {
    pad_0: [u8; 0x190],                                           // 0x0000 对齐填充
    pub type_scopes: UtlVector<Pointer64<SchemaSystemTypeScope>>, // 0x0190 类型作用域数组
    pad_1: [u8; 0xE0],                                            // 0x01A0 对齐填充
    pub registration_count: i32,                                  // 0x0280 注册数量
}

unsafe impl Pod for SchemaSystem {}
