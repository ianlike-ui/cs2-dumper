// ============================================================================
// tier0/ts_list.rs —— 引擎的"线程安全链表"（TSList）内存布局
//
// Source 引擎的一些底层结构用线程安全链表组织。这里定义的是链表头
// （next 指针）和链表基类的布局。本项目目前只用到它的头节点结构。
// ============================================================================

use memflow::types::Pointer64;

// 链表节点（内容在本项目里用不到，只占位）
#[repr(C)]
pub struct TsListNode;

// 链表头：指向第一个节点
#[repr(C)]
pub struct TsListHead {
    pub next: Pointer64<TsListNode>, // 0x0000 下一节点
}

// 链表基类：从 head 开始
#[repr(C)]
pub struct TsListBase {
    pub head: TsListHead, // 0x0000 链表头
}
