// ============================================================================
// tier1/utl_memory.rs —— Valve 的"动态内存块"容器（UtlMemory<T>）
//
// 类似 C 的"指针 + 元素个数"：一段连续内存 + 计数。
// 是 UtlVector 的底层存储。element() 从目标进程内存里读出下标为 index
// 的元素（跨进程读取，不是本地数组）。
// ============================================================================

use memflow::prelude::v1::*;

#[repr(C)]
pub struct UtlMemory<T> {
    pub data: Pointer64<[T]>, // 0x0000 数据起始指针
    pub count: i32,           // 0x0008 元素个数
    pub grow_size: i32,       // 0x000C 扩容增量（负数表示外部分配）
}

impl<T: Pod> UtlMemory<T> {
    // 是否为"外部分配"（负的 grow_size 表示内存不是本容器管的）
    #[inline]
    pub fn is_externally_allocated(&self) -> bool {
        self.grow_size < 0
    }

    // 从目标进程内存读取第 index 个元素（越界返回错误）
    pub fn element(&self, mem: &mut impl MemoryView, index: usize) -> Result<T> {
        if index >= self.count as usize {
            return Err(ErrorKind::OutOfBounds.into()); // 越界保护
        }

        mem.read_ptr(self.data.at(index as _)).data_part() // 读进程内存里的元素
    }
}
