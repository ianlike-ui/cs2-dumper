// ============================================================================
// source2/mod.rs —— Source 2 引擎结构的根模块
//
// 下面这些子模块分别对应 CS2 引擎（Source 2）里几类底层数据结构：
//   - client：客户端模块（按键输入等）
//   - schema_system：Schema 反射系统（类/字段/枚举的元数据）
//   - tier0：引擎最底层工具（如线程安全列表 TSList）
//   - tier1：更上层的通用容器（接口注册表、UtlVector 动态数组等）
//
// 这些结构体都是从游戏内存里"按原样"读出来的——所以都用 #[repr(C)]
// 精确对齐内存布局，用 Pointer64<T> 表示游戏进程内的指针。
// ============================================================================

pub use client::*;
pub use schema_system::*;
pub use tier0::*;
pub use tier1::*;

pub mod client;
pub mod schema_system;
pub mod tier0;
pub mod tier1;
