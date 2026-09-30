// ============================================================================
// tier1/mod.rs —— tier1 层子模块的根
//
// tier1 提供引擎的通用容器和注册机制：
//   - interface.rs        ：接口注册节点（InterfaceReg）
//   - utl_memory.rs       ：动态内存块（UtlMemory）
//   - utl_memory_pool.rs  ：内存池（UtlMemoryPool）
//   - utl_ts_hash.rs      ：线程安全哈希表（UtlTsHash）
//   - utl_vector.rs       ：动态数组（UtlVector）
// 这些是 Valve 源码里著名的 Utl* 系列容器。
// ============================================================================

pub use interface::*;
pub use utl_memory::*;
pub use utl_memory_pool::*;
pub use utl_ts_hash::*;
pub use utl_vector::*;

pub mod interface;
pub mod utl_memory;
pub mod utl_memory_pool;
pub mod utl_ts_hash;
pub mod utl_vector;
