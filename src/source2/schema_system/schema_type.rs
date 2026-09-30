// ============================================================================
// schema_type.rs —— "类型描述"的内存布局（Schema 系统里最复杂的结构）
//
// ----------------------------------------------------------------------------
// 🎓 概念卡 ㊰：enum + repr(u8) 与 union 联合体
// ----------------------------------------------------------------------------
// 【enum + repr(u8) 是什么】Rust 的 enum 默认很强大（每个变体可以带数据），
// 但内存布局不固定。`#[repr(u8)]` 强制它退化成"一个 u8 整数"：每个变体
// 对应一个数字（Basic=0、Ptr=1…）。这正是游戏内存里存的形式——一个字节。
// 于是可以放心把游戏内存里的字节直接转成这个枚举。
// 【union 是什么】和 C 的 union 一样：**同一块内存**可以按多种类型解释。
// SchemaTypeUnion 里的同一段 8 字节，按类型类别的不同，可以当"指针"、
// "数组信息"或"模板参数"读。代价：程序员必须自己保证"按对的类型读"。
// 【最小示例】
//   #[repr(u8)]
//   enum Kind { A = 0, B = 1 }             // 就是一个字节 0 或 1
//   #[repr(C)]
//   union Num { int: i32, float: f32 }     // 同一 4 字节两种解释
//   let n = Num { int: 42 };
//   unsafe { n.float }                     // 按 float 解释这 4 字节（可能不是 42.0！）
// 【自己动手要点】
// 1) 用 union 读游戏内存时，要"按游戏实际存的方式"选字段，错了就是垃圾值；
// 2) 访问 union 字段必须 unsafe（编译器无法保证你解释对了）；
// 3) 这就是为什么 analysis 里读枚举值统一按 ulong 读（见 enumerator 卡）。
// ----------------------------------------------------------------------------
//
// 【这个结构体是什么】
// Schema 里每个"类型"（int32、float、C_BasePlayerPawn、数组、模板…）
// 都用一个 SchemaType 描述。它包含：
//   - 类型名（如 "Vector"、"int32"、"C_CSPlayerPawn"）；
//   - 类型类别（type_category）：内置类型/指针/位域/定长数组/原子/类/枚举；
//   - 原子类别（atomic_category）：如果是模板/泛型，具体是哪一种组合；
//   - value（联合体）：不同类别对应不同内容（数组信息、模板参数、类/枚举绑定…）。
//
// 【哪些文件用得到】
// analysis/schemas.rs 读字段类型名（read_class_binding_fields 里读
// type->name）时就会用到。
//
// 【Rust 的 enum + repr(u8) 是什么】
// 类似 C++ 的 enum class：给整数值起名字。这里直接对应游戏内存里的字节值。
// ============================================================================

use memflow::prelude::v1::*;

use super::{SchemaClassBinding, SchemaEnumBinding, SchemaSystemTypeScope};

// 原子类型类别（模板组合方式）
#[repr(u8)]
pub enum SchemaAtomicCategory {
    Basic = 0,     // 基础类型（int、float…）
    T,             // 模板 T
    CollectionOfT, // 集合 T
    TF,            // 模板 T + 固定大小
    TT,            // 两个模板参数
    TTF,           // 两个模板参数 + 固定大小
    I,             // 原子整数
    None,          // 无
}

// 类型类别
#[repr(u8)]
pub enum SchemaTypeCategory {
    BuiltIn = 0,   // 内置类型
    Ptr,           // 指针
    Bitfield,      // 位域
    FixedArray,    // 定长数组
    Atomic,        // 原子/模板
    DeclaredClass, // 声明的类
    DeclaredEnum,  // 声明的枚举
    None,          // 无
}

// 定长数组信息
#[derive(Clone, Copy)]
#[repr(C)]
pub struct SchemaArrayT {
    pub array_size: u32,                // 0x0000 数组长度
    pad_0: [u8; 0x4],                   // 0x0004 对齐填充
    pub element: Pointer64<SchemaType>, // 0x0008 元素类型
}

// 原子整数
#[derive(Clone, Copy)]
#[repr(C)]
pub struct SchemaAtomicI {
    pad_0: [u8; 0x10], // 0x0000 对齐填充
    pub value: u64,    // 0x0010 值
}

// 模板 T
#[derive(Clone, Copy)]
#[repr(C)]
pub struct SchemaAtomicT {
    pub element: Pointer64<SchemaType>,  // 0x0000 元素类型
    pad_0: [u8; 0x8],                    // 0x0008 对齐填充
    pub template: Pointer64<SchemaType>, // 0x0010 模板参数
}

// 模板 TT（两个参数）
#[derive(Clone, Copy)]
#[repr(C)]
pub struct SchemaAtomicTT {
    pad_0: [u8; 0x10],                         // 0x0000 对齐填充
    pub templates: [Pointer64<SchemaType>; 2], // 0x0010 两个模板参数
}

// 模板 TF（一个参数 + 大小）
#[derive(Clone, Copy)]
#[repr(C)]
pub struct SchemaAtomicTF {
    pad_0: [u8; 0x10],                   // 0x0000 对齐填充
    pub template: Pointer64<SchemaType>, // 0x0010 模板参数
    pub size: i32,                       // 0x0018 大小
}

// 模板 TTF（两个参数 + 大小）
#[derive(Clone, Copy)]
#[repr(C)]
pub struct SchemaAtomicTTF {
    pad_0: [u8; 0x10],                         // 0x0000 对齐填充
    pub templates: [Pointer64<SchemaType>; 2], // 0x0010 两个模板参数
    pub size: i32,                             // 0x0020 大小
}

// 类型描述（主结构）
#[repr(C)]
pub struct SchemaType {
    pad_0: [u8; 0x8],                                 // 0x0000 对齐填充
    pub name: Pointer64<ReprCString>,                 // 0x0008 类型名
    pub type_scope: Pointer64<SchemaSystemTypeScope>, // 0x0010 所属类型作用域
    pub type_category: SchemaTypeCategory,            // 0x0018 类型类别
    pub atomic_category: SchemaAtomicCategory,        // 0x0019 原子类别
    pub value: SchemaTypeUnion,                       // 0x0020 具体内容（联合体）
}

unsafe impl Pod for SchemaType {}

// 联合体：按类型类别解释同一段内存
pub union SchemaTypeUnion {
    pub r#type: Pointer64<SchemaType>,          // 指针目标类型
    pub class_binding: Pointer64<SchemaClassBinding>, // 类绑定
    pub enum_binding: Pointer64<SchemaEnumBinding>,   // 枚举绑定
    pub array: SchemaArrayT,                    // 数组信息
    pub atomic: SchemaAtomicT,                  // 模板 T
    pub atomic_tt: SchemaAtomicTT,              // 模板 TT
    pub atomic_tf: SchemaAtomicTF,              // 模板 TF
    pub atomic_ttf: SchemaAtomicTTF,            // 模板 TTF
    pub atomic_i: SchemaAtomicI,                // 原子整数
}
