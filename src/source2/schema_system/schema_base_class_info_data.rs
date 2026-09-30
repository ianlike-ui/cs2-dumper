// ============================================================================
// schema_base_class_info_data.rs —— "基类关系"的内存布局
//
// 记录一个类的基类信息。分析时用于找出类的父类名
// （SchemaBaseClassInfoData 里的 class 指向一个 SchemaBaseClass，
// 它有 name 字段）。
// ============================================================================

use memflow::prelude::v1::*;

// 基类信息节点
#[repr(C)]
pub struct SchemaBaseClassInfoData {
    pad_0: [u8; 0x18],                     // 0x0000 对齐填充
    pub class: Pointer64<SchemaBaseClass>, // 0x0018 指向基类描述
}

unsafe impl Pod for SchemaBaseClassInfoData {}

// 单个基类的描述
#[repr(C)]
pub struct SchemaBaseClass {
    pad_0: [u8; 0x10],                // 0x0000 对齐填充
    pub name: Pointer64<ReprCString>, // 0x0010 基类名
}

unsafe impl Pod for SchemaBaseClass {}
