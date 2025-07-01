use vm::memory_page::MemoryPage;
use std::rc::Rc;
pub struct MemoryPageManager {
    pub page_size: usize,
    max_pages: usize,
    pages: Vec<Rc<MemoryPage>>,
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
    pub fn new_page(&mut self) -> Rc<MemoryPage> {
        if self.pages.len() >= self.max_pages {
            panic!("Out of memory: maximum page count ({}) reached", self.max_pages);
        }

        let page = Rc::new(MemoryPage::new(self.page_size));
        self.pages.push(Rc::clone(&page));
        return page;
    }

    /// Pretty-prints all memory pages linearly, indicating page boundaries
    pub fn dump_all_pages_linear(&self) {
        println!("Dumping memory ({} pages):", self.pages.len());
        for (i, page) in self.pages.iter().enumerate() {
            println!("\n=== Page {} ===", i);
            let mem = page.mem();
            for (j, chunk) in mem.chunks(16).enumerate() {
                print!("0x{:04x}: ", j * 16);
                for byte in chunk {
                    print!("{:02x} ", byte);
                }
                println!();
            }
        }
    }

    pub fn get_page(&self, index: usize) -> Option<Rc<MemoryPage>> {
        self.pages.get(index).cloned() // ✅ clone the Rc (increases refcount)
    }

    pub fn first_page(&self) -> Option<Rc<MemoryPage>> {
        self.pages.first().cloned() // ✅ clone the Rc (increases refcount)
    }   

}
