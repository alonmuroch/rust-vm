use std::rc::Rc;

use super::MemoryPage;
use vm::memory::Memory;

/// Manages a stack of memory pages allocated in order.
#[derive(Debug)]
pub struct StackedMemory {
    pub page_size: usize,
    max_pages: usize,
    pages: Vec<Memory>,
}

impl StackedMemory {
    pub fn new(max_pages: usize, page_size: usize) -> Self {
        assert!(max_pages != 0, "max_pages must be > 0");
        assert!(page_size != 0, "page_size must be > 0");

        Self {
            page_size,
            max_pages,
            pages: Vec::with_capacity(max_pages),
        }
    }

    /// Creates and owns a new page.
    pub fn new_page(&mut self) -> Memory {
        if self.pages.len() >= self.max_pages {
            panic!(
                "out of memory: maximum page count ({}) reached",
                self.max_pages
            );
        }

        let page: Memory = Rc::new(MemoryPage::new(self.page_size));
        self.pages.push(Rc::clone(&page));
        page
    }

    pub fn pop_page(&mut self) {
        self.pages.pop();
    }

    pub fn get_page(&self, index: usize) -> Option<Memory> {
        self.pages.get(index).cloned()
    }

    pub fn top_page(&self) -> Option<Memory> {
        self.pages.last().cloned()
    }

    pub fn count(&self) -> usize {
        self.pages.len()
    }
}
