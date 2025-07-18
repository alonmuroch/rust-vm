use vm::memory_page::MemoryPage;
use std::{cell::RefCell, rc::Rc};

#[derive(Debug)]
pub struct MemoryPageManager {
    pub page_size: usize,
    max_pages: usize,
    pages: Vec<Rc<RefCell<MemoryPage>>>,
}

impl MemoryPageManager {
    pub fn new(max_pages: usize, page_size: usize) -> Self {
        assert!(max_pages != 0, "Max pages == 0");
        assert!(page_size != 0, "Page size == 0");

        Self {
            page_size,
            max_pages,
            pages: Vec::with_capacity(max_pages),
        }
    }

    /// Creates and owns a new page. Returns a mutable reference to it.
    pub fn new_page(&mut self) -> Rc<RefCell<MemoryPage>> {
        if self.pages.len() >= self.max_pages {
            panic!("Out of memory: maximum page count ({}) reached", self.max_pages);
        }

        let page = Rc::new(RefCell::new(MemoryPage::new(self.page_size)));
        self.pages.push(Rc::clone(&page));
        return page;
    }

    pub fn pop_page(&mut self) {
        self.pages.pop();
    }

    /// Pretty-prints all memory pages linearly, indicating page boundaries
    pub fn dump_all_pages_linear(&self) {
        println!("Dumping memory ({} pages):", self.pages.len());
        for (i, page_rc) in self.pages.iter().enumerate() {
            println!("\n=== Page {} ===", i);

            let page = page_rc.borrow();
            let mem = page.mem();

            for (j, chunk) in mem.chunks(16).enumerate() {
                print!("0x{:04x}: ", j * 16);

                // Print hex representation
                for byte in chunk {
                    print!("{:02x} ", byte);
                }

                // Pad spacing if chunk is less than 16 bytes
                for _ in chunk.len()..16 {
                    print!("   ");
                }

                // Print ASCII representation
                print!(" |");
                for byte in chunk {
                    let ch = *byte;
                    let display_char = if ch.is_ascii_graphic() || ch == b' ' {
                        ch as char
                    } else {
                        '.'
                    };
                    print!("{}", display_char);
                }
                println!("|");
            }
        }
}



    pub fn get_page(&self, index: usize) -> Option<Rc<RefCell<MemoryPage>>> {
        self.pages.get(index).cloned() // ✅ clone the Rc (increases refcount)
    }

    pub fn first_page(&self) -> Option<Rc<RefCell<MemoryPage>>> {
        self.pages.first().cloned() // ✅ clone the Rc (increases refcount)
    }

    pub fn top_page(&self) -> Option<Rc<RefCell<MemoryPage>>> {
        self.pages.last().cloned() // ✅ clone the Rc (increases refcount)
    }   

}
