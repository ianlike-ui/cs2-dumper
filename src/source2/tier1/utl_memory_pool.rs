// ============================================================================
// tier1/utl_memory_pool.rs —— Valve 的"内存池"（UtlMemoryPool）布局
//
// 【什么是内存池？】
// 游戏里大量对象频繁创建/销毁，直接 malloc/free 太慢且碎片化。
// 内存池的做法：一次性申请一大块（blob），切分成很多固定大小的小块
// （block），用完放回空闲链表（free_blocks），要时再取。
//
// 本项目目前没有直接遍历内存池，但这个结构出现在一些引擎对象里，
// 所以定义好布局备用。
// ============================================================================

use memflow::prelude::v1::*;

use crate::source2::TsListBase;

// 池的增长模式
#[repr(u32)]
pub enum MemoryPoolGrowType {
    None = 0, // 不分配新块
    Fast,     // 新块越来越大
    Slow,     // 新块大小不变
}

// 一块内存池"大块"（blob）
#[derive(Pod)]
#[repr(C)]
pub struct UtlMemoryPoolBlob {
    pub next: Pointer64<UtlMemoryPoolBlob>, // 0x0000 下一个 blob
    pub size: i32,                          // 0x0008 大小
    pub data: [u8; 1],                      // 0x000C 数据起始
    pad_0: [u8; 0x3],                       // 0x000D 对齐填充
}

// 内存池
#[repr(C)]
pub struct UtlMemoryPool {
    pub block_size: i32,                         // 0x0000 每小块大小
    pub blocks_per_blob: i32,                    // 0x0004 每大块含多少小块
    pub grow_mode: MemoryPoolGrowType,           // 0x0008 增长模式
    pub blocks_allocated: i32,                   // 0x000C 已分配块数
    pub peak_allocated: i32,                     // 0x0010 峰值
    pub alignment: u16,                          // 0x0014 对齐
    pub blob_count: u16,                         // 0x0016 大块数量
    pad_0: [u8; 0x2],                            // 0x0018 对齐填充
    pub free_blocks: TsListBase,                 // 0x0020 空闲块链表
    pad_1: [u8; 0x20],                           // 0x0028 对齐填充
    pub blob_head: Pointer64<UtlMemoryPoolBlob>, // 0x0048 大块链表头
    pub total_size: i32,                         // 0x0050 总大小
    pad_2: [u8; 0xC],                            // 0x0054 对齐填充
}
