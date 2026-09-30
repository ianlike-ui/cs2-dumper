// ============================================================================
// tier1/utl_ts_hash.rs —— Valve 的"线程安全哈希表"（UtlTsHash）
//
// ----------------------------------------------------------------------------
// 🎓 概念卡 ㉙：常量泛型 const C: usize = 256（编译期就知道的数组长度）
// ----------------------------------------------------------------------------
// 【是什么】普通的 <T> 参数是"类型"，const C: usize 参数是"数字"，
// 而且这个数字在**编译期**就要确定。`[UtlTsHashBucket<D, K>; C]` 表示
// "长度正好是 C 的数组"——C 写死成 256 就是 256 个桶，写死成 512 就是
// 512 个，用同一个结构体生成两种哈希表。
// 【为什么在这里】游戏里不同的哈希表桶数不同，用常量泛型一个定义全覆盖。
// 【最小示例】
//   struct Array<T, const N: usize> { data: [T; N] }   // N 必须编译期已知
//   let a: Array<i32, 3> = Array { data: [1, 2, 3] };  // 长度 3
// 【自己动手要点】
// 1) 数组 [T; N] 的长度必须是"编译期常量"，所以 N 用 const 泛型；
// 2) 函数级 const 泛型写法：fn f<const N: usize>() { ... }。
// ----------------------------------------------------------------------------
//
// 【这个容器是干什么的】
// Schema 系统里，一个模块的所有类/枚举都存在一张 UtlTsHash 里。
// 它内部是一组"桶"（bucket），每个桶里是一条"节点链表"。
// 节点数据存在内存池（entry_mem）里。
//
// 【elements() 在做什么】
// 把哈希表里所有"数据指针"收集成一个 Vec 返回。它同时遍历：
//   1. 已分配元素（allocated：从各桶的链表里收集）；
//   2. 未分配元素（unallocated：从内存池的空闲块链表里收集，
//      这些块里的数据指针实际上也是有效的——Valve 这种实现
//      把"分配出去"的节点也挂在空闲链表上做延迟提交）；
// 最后用 HashSet 去重。
//
// 【const C: usize = 256 是什么】
// Rust 的"常量泛型"：编译期就知道的数组长度（这里默认 256 个桶）。
// ============================================================================

use std::collections::HashSet;

use memflow::prelude::v1::*;

use super::UtlMemoryPool;

// 大块（blob）：从内存池取出的整块节点数据
#[repr(C)]
pub struct UtlTsHashAllocatedBlob<D> {
    pub next: Pointer64<UtlTsHashAllocatedBlob<D>>, // 0x0000 下一个 blob
    pad_0: [u8; 0x8],                               // 0x0008 对齐填充
    pub data: Pointer64<D>,                         // 0x0010 数据指针
    pad_1: [u8; 0x18],                              // 0x0018 对齐填充
}

unsafe impl<D: 'static> Pod for UtlTsHashAllocatedBlob<D> {}

// 链表里的固定节点（含键和数据指针）
#[repr(C)]
pub struct UtlTsHashFixedData<D, K> {
    pub ui_key: K,                                 // 0x0000 键
    pub next: Pointer64<UtlTsHashFixedData<D, K>>, // 0x0008 下一个节点
    pub data: Pointer64<D>,                        // 0x0010 数据指针
}

unsafe impl<D: 'static, K: 'static> Pod for UtlTsHashFixedData<D, K> {}

// 桶：一把"加锁标志" + 两条链表（已提交 / 未提交）
#[repr(C)]
pub struct UtlTsHashBucket<D, K> {
    pub add_lock: usize,                                        // 0x0000 添加锁
    pub first: Pointer64<UtlTsHashFixedData<D, K>>,             // 0x0008 已提交链表头
    pub first_uncommitted: Pointer64<UtlTsHashFixedData<D, K>>, // 0x0010 未提交链表头
}

// 哈希表本体：内存池 + 桶数组
#[repr(C)]
pub struct UtlTsHash<D, const C: usize = 256, K = u64> {
    pub entry_mem: UtlMemoryPool,            // 0x0000 节点内存池
    pub buckets: [UtlTsHashBucket<D, K>; C], // 0x0060 桶数组（默认 256 个）
    pub needs_commit: bool,                  // 0x1860 是否需要提交
    pad_0: [u8; 0x3],                        // 0x1861 对齐填充
    pub contention_check: i32,               // 0x1864 竞争检查计数
    pad_1: [u8; 0x8],                        // 0x1868 对齐填充
}

impl<D: Pod, const C: usize, K: Pod> UtlTsHash<D, C, K> {
    // 收集所有元素的数据指针（去重）
    pub fn elements(&self, mem: &mut impl MemoryView) -> Vec<Pointer64<D>> {
        let allocated = self.allocated_elements(mem);
        let unallocated = self.unallocated_elements(mem);

        let mut result = Vec::with_capacity(allocated.len() + unallocated.len());

        result.extend(allocated);
        result.extend(unallocated);

        let mut seen = HashSet::with_capacity(result.capacity());

        // 去掉同时出现在两条链表里的重复指针
        result.retain(|ptr| seen.insert(ptr.address().to_umem()));

        result
    }

    // 从各桶的链表收集"已分配"元素
    fn allocated_elements(&self, mem: &mut impl MemoryView) -> Vec<Pointer64<D>> {
        let used_count = self.entry_mem.blocks_allocated as usize; // 已用块数 = 预期元素数

        let mut elements = Vec::with_capacity(used_count);

        for bucket in &self.buckets {
            let mut node_ptr = bucket.first_uncommitted; // 从未提交链表开始

            while !node_ptr.is_null() {
                let node = match mem.read_ptr(node_ptr).data_part() {
                    Ok(n) => n,
                    Err(_) => break, // 读失败就停（内存可能变了）
                };

                if !node.data.is_null() {
                    elements.push(node.data);
                }

                if elements.len() >= used_count {
                    break; // 够了就停
                }

                node_ptr = node.next;
            }
        }

        elements
    }

    // 从内存池空闲块链表收集元素
    fn unallocated_elements(&self, mem: &mut impl MemoryView) -> Vec<Pointer64<D>> {
        let free_count = self.entry_mem.peak_allocated as usize;

        let mut elements = Vec::with_capacity(free_count);

        let mut blob_ptr = Pointer64::<UtlTsHashAllocatedBlob<D>>::from(
            self.entry_mem.free_blocks.head.next.address(),
        );

        while !blob_ptr.is_null() {
            let blob = match mem.read_ptr(blob_ptr).data_part() {
                Ok(b) => b,
                Err(_) => break,
            };

            if !blob.data.is_null() {
                elements.push(blob.data);
            }

            if elements.len() >= free_count {
                break;
            }

            blob_ptr = blob.next;
        }

        elements
    }
}

unsafe impl<D: 'static, const C: usize, K: 'static> Pod for UtlTsHash<D, C, K> {}
