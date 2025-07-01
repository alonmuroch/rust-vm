use std::rc::Rc;
use std::cell::RefCell;

use crate::memory::MemoryPage;

pub struct MemoryPageManager {
    backing: Rc<RefCell<Vec<u8>>>,
    page_size: usize,
    pages: Vec<usize>, // page start offsets
}

impl MemoryPageManager {
    pub fn new(total_memory_size: usize, page_size: usize) -> Self {
        Self {
            backing: Rc::new(RefCell::new(vec![0u8; total_memory_size])),
            page_size,
            pages: Vec::new(),
        }
    }

    pub fn new_page(&mut self) -> MemoryPage {
        let offset = self.pages.len() * self.page_size;
        if offset + self.page_size > self.backing.borrow().len() {
            panic!("Out of memory: cannot allocate new page");
        }
        self.pages.push(offset);
        MemoryPage::new(Rc::clone(&self.backing))
    }

    /// 🔍 Pretty prints all memory pages linearly, indicating page boundaries
    pub fn dump_all_pages_linear(&self) {
        let mem = self.backing.borrow();
        println!("Dumping memory ({} pages):", self.pages.len());
        for (i, &offset) in self.pages.iter().enumerate() {
            let end = offset + self.page_size;
            let slice = &mem[offset..end];
            println!("\n=== Page {} (offset 0x{:08x}) ===", i, offset);
            for (j, chunk) in slice.chunks(16).enumerate() {
                print!("0x{:08x}: ", offset + j * 16);
                for byte in chunk {
                    print!("{:02x} ", byte);
                }
                println!();
            }
        }
    }
}
