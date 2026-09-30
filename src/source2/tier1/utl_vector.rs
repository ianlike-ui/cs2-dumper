// ============================================================================
// tier1/utl_vector.rs —— Valve 的"动态数组"（UtlVector<T>）
//
// ----------------------------------------------------------------------------
// 🎓 概念卡 ㉘：泛型 <T>（类型参数化）
// ----------------------------------------------------------------------------
// 【是什么】把"类型"也变成参数：写一次逻辑，用的时候指定类型。
// UtlVector<T> 里的 T 可以是任何类型：UtlVector<Pointer64<Foo>> 装指针，
// UtlVector<Bar> 装 Bar。编译器会为每种用到的类型各生成一份代码。
// 【为什么在这里】Source 引擎里"变长数组"到处都是，但装的元素类型各不相
// 同。用泛型写一个，所有场景复用，不用为每种类型抄一遍。
// 【最小示例】
//   struct Box<T> { value: T }
//   let a = Box { value: 42 };          // T = i32（自动推断）
//   let b = Box { value: "hi" };        // T = &str
// 【自己动手要点】
// 1) impl<T: Pod> 里的 ": Pod" 是"类型约束"：T 必须实现了 Pod 才能用这些方法；
// 2) 泛型 + trait 约束是 Rust 最常见的组合，看不懂就拆开看：先看 T 是什么，
//    再看约束要求了什么。
// ----------------------------------------------------------------------------
//
// 类似 std::vector：元素个数 + 数据指针。element() 从目标进程内存
// 读取第 index 个元素（跨进程读取）。
// ============================================================================

use memflow::prelude::v1::*;

#[repr(C)]
pub struct UtlVector<T> {
    pub count: i32,           // 0x0000 元素个数
    pad_0: [u8; 0x4],         // 0x0004 对齐填充
    pub data: Pointer64<[T]>, // 0x0008 数据起始指针
}

impl<T: Pod> UtlVector<T> {
    // 读取第 index 个元素（越界返回错误）
    pub fn element(&self, mem: &mut impl MemoryView, index: usize) -> Result<T> {
        if index >= self.count as usize {
            return Err(ErrorKind::OutOfBounds.into());
        }

        mem.read_ptr(self.data.at(index as _)).data_part()
    }
}

unsafe impl<T: 'static> Pod for UtlVector<T> {}
