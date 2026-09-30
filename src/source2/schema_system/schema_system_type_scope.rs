// ============================================================================
// schema_system_type_scope.rs —— "类型作用域"的内存布局
//
// 【什么是类型作用域？】
// Schema 系统把类按"所属模块"分组，每个模块（dll）一个 TypeScope。
// 它里面有两个大哈希表：
//   - class_bindings：这个模块的所有类（按类名哈希）
//   - enum_bindings：这个模块的所有枚举
// analysis/schemas.rs 的 read_type_scopes 就是遍历 SchemaSystem 的
// type_scopes 列表，再逐个读这两个哈希表。
//
// 【UtlTsHash 是什么？】
// 这是 Source 引擎自己的"线程安全哈希表"（定义在 tier1/utl_ts_hash.rs），
// 提供 element() 方法按下标取元素。
// ============================================================================

use std::ffi::c_char;

use memflow::prelude::v1::*;

use super::{SchemaClassBinding, SchemaEnumBinding};

use crate::source2::UtlTsHash;

#[derive(Pod)]
#[repr(C)]
pub struct SchemaSystemTypeScope {
    pad_0: [u8; 0x8],                                   // 0x0000 对齐填充
    pub name: [c_char; 256],                            // 0x0008 作用域名（= 模块名，256 字节 C 字符串）
    pub global_scope: Pointer64<SchemaSystemTypeScope>, // 0x0108 全局作用域指针
    pad_1: [u8; 0x450],                                 // 0x0110 对齐填充
    pub class_bindings: UtlTsHash<SchemaClassBinding>,  // 0x0560 类哈希表
    pub enum_bindings: UtlTsHash<SchemaEnumBinding>,    // 0x1DD0 枚举哈希表
}
