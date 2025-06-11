use goblin::elf::Elf;

pub struct ElfInfo<'a> {
    pub code: &'a [u8],
    pub sections: Vec<ElfSection<'a>>,
}

pub struct ElfSection<'a> {
    pub name: String,
    pub addr: u64,
    pub size: u64,
    pub data: &'a [u8],
}

pub fn parse_elf_from_bytes<'a>(bytes: &'a [u8]) -> Result<ElfInfo<'a>, goblin::error::Error> {
    let elf = Elf::parse(bytes)?;

    let mut sections = Vec::new();
    for section in elf.section_headers.iter() {
        if let Some(name) = elf.shdr_strtab.get_at(section.sh_name) {
            let offset = section.sh_offset as usize;
            let size = section.sh_size as usize;

            if offset + size <= bytes.len() {
                let data = &bytes[offset..offset + size];
                sections.push(ElfSection {
                    name: name.to_string(),
                    addr: section.sh_addr,
                    size: section.sh_size,
                    data,
                });
            }
        }
    }

    Ok(ElfInfo { code: bytes, sections })
}
