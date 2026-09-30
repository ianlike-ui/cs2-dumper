// ============================================================================
// schema_class_info_data.rs —— "类绑定"的内存布局
//
// ----------------------------------------------------------------------------
// 🎓 概念卡 ㊱：type 类型别名（给类型起短名）
// ----------------------------------------------------------------------------
// 【是什么】`type SchemaClassBinding = SchemaClassInfoData;` 表示
// "SchemaClassBinding 就是 SchemaClassInfoData 的别名"——同一个类型两个名字。
// 作用：让代码读起来更符合领域语言（分析代码里"类绑定"比"类信息数据"更
// 容易懂），也方便以后换底层实现只改一处。
// 【最小示例】
//   type UserId = u64;
//   fn lookup(id: UserId) {}             // 写起来像领域概念，其实还是 u64
// 【自己动手要点】
// 1) 别名不是新类型：u64 和 UserId 可以互相赋值，编译器不会区分；
// 2) 想要"真的新类型"要用 newtype 模式（struct UserId(u64)）。
// ----------------------------------------------------------------------------
//
// 【什么是类绑定？】
// Schema 系统里，每个类（如 C_BasePlayerPawn）对应一个 SchemaClassInfoData
// 结构，记录了这个类的完整元数据：
//   - 类名、模块名（在哪个 dll 里）、二进制名；
//   - 类大小、字段数量、字段数组指针；
//   - 基类信息、静态元数据（网络变量等）。
// analysis/schemas.rs 里 read_class_binding 读的就是这个结构。
//
// 【内存偏移注释的含义】
// 每个字段后面的 // 0x0008 表示这个字段相对结构体开头的字节偏移，
// 这些值来自逆向确认。外部工具读游戏内存时用的就是这类偏移。
// ============================================================================

use memflow::prelude::v1::*;

use super::*;

// 类型别名：类绑定 = 类信息数据
pub type SchemaClassBinding = SchemaClassInfoData;

// rustfmt::skip：保持手写的内存布局注释对齐，不让格式化工具重排
#[rustfmt::skip]
#[derive(Pod)]
#[repr(C)]
pub struct SchemaClassInfoData {
    pub base: Pointer64<SchemaClassInfoData>,                  // 0x0000 基类信息
    pub name: Pointer64<ReprCString>,                          // 0x0008 类名
    pub binary_name: Pointer64<ReprCString>,                   // 0x0010 二进制名
    pub module_name: Pointer64<ReprCString>,                   // 0x0018 所在模块名（dll）
    pub size: i32,                                             // 0x0020 实例大小
    pub field_count: i16,                                      // 0x0024 字段数量
    pub static_metadata_count: i16,                            // 0x0026 静态元数据数量
    pad_0: [u8; 0x2],                                          // 0x0028 对齐填充
    pub alignment: u8,                                         // 0x002A 对齐要求
    pub has_base_class: u8,                                    // 0x002B 是否有基类
    pub total_class_size: i16,                                 // 0x002C 总大小（含基类）
    pub derived_class_size: i16,                               // 0x002E 派生部分大小
    pub fields: Pointer64<[SchemaClassFieldData]>,             // 0x0030 字段数组指针
    pad_1: [u8; 0x8],                                          // 0x0038 对齐填充
    pub base_classes: Pointer64<SchemaBaseClassInfoData>,      // 0x0040 基类列表
    pub static_metadata: Pointer64<[SchemaMetadataEntryData]>, // 0x0048 静态元数据数组
    pub type_scope: Pointer64<SchemaSystemTypeScope>,          // 0x0058 所属类型作用域
    pub r#type: Pointer64<SchemaType>,                         // 0x0060 类型描述
    pad_2: [u8; 0x10],                                         // 0x0068 对齐填充
}
