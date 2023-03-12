use std::{cmp::min, collections::HashMap};

use crate::{machine::Machine, pe, winapi};

pub fn load_exe(
    machine: &mut Machine,
    buf: &[u8],
    cmdline: String,
) -> anyhow::Result<HashMap<u32, String>> {
    let file = pe::parse(&buf)?;

    let base = file.opt_header.ImageBase;
    machine.state.kernel32.image_base = base;
    // TODO: 5k_run.exe specifies SizeOfImage as like 700mb, but then doesn't
    // end up using it.  We might need to figure out uncommitted memory to properly
    // load it.
    let image_size = min(file.opt_header.SizeOfImage, 10 << 20);
    machine.x86.mem.resize((base + image_size) as usize, 0);

    // The first 0x1000 of the PE file itself is loaded at the base address.
    // I cannot find documentation of this but it is what I observe in a debugger,
    // and kkrunchy relies on file data found after the PE header but outside of any section.
    let size = 0x1000 as usize;
    machine
        .state
        .kernel32
        .mappings
        .add(winapi::kernel32::Mapping {
            addr: base,
            size: size as u32,
            desc: format!("PE header"),
            flags: pe::ImageSectionFlags::MEM_READ,
        });
    machine.x86.mem[base as usize..(base as usize + size)].copy_from_slice(&buf[..size]);

    for sec in file.sections {
        let src = sec.PointerToRawData as usize;
        let dst = (base + sec.VirtualAddress) as usize;
        // sec.SizeOfRawData is the amount of data in the file that should be copied to memory.
        // sec.VirtualSize is the in-memory size of the resulting section, which can be:
        // - greater than SizeOfRawData for sections that should be zero-filled (like uninitialized data),
        // - less than SizeOfRawData because SizeOfRawData is padded up to FileAlignment(!).

        let data_size = sec.SizeOfRawData as usize;
        let flags = sec.characteristics()?;

        // Load the section contents from the file.
        // Note: kkrunchy-packed files have a single section marked
        // CODE | INITIALIZED_DATA | UNINITIALIZED_DATA | MEM_EXECUTE | MEM_READ | MEM_WRITE
        // so we ignore the UNINITIALIZED_DATA flag.
        let load_data = flags.contains(pe::ImageSectionFlags::CODE)
            || flags.contains(pe::ImageSectionFlags::INITIALIZED_DATA);
        if load_data && data_size > 0 {
            machine.x86.mem[dst..dst + data_size].copy_from_slice(&buf[src..(src + data_size)]);
        }
        machine
            .state
            .kernel32
            .mappings
            .add(winapi::kernel32::Mapping {
                addr: dst as u32,
                size: sec.VirtualSize as u32,
                desc: format!("{:?} ({:?})", sec.name(), flags),
                flags,
            });
    }

    machine.state.kernel32.init(&mut machine.x86.mem, cmdline);
    machine.x86.regs.fs_addr = machine.state.kernel32.teb;

    let mut stack_size = file.opt_header.SizeOfStackReserve;
    // Zig reserves 16mb stacks, just truncate for now.
    if stack_size > 1 << 20 {
        log::warn!(
            "requested {}mb stack reserve, using 32kb instead",
            stack_size / (1 << 20)
        );
        stack_size = 32 << 10;
    }
    let stack =
        machine
            .state
            .kernel32
            .mappings
            .alloc(stack_size, "stack".into(), &mut machine.x86.mem);
    let stack_end = stack.addr + stack.size - 4;
    machine.x86.regs.esp = stack_end;
    machine.x86.regs.ebp = stack_end;

    let mut labels: HashMap<u32, String> = HashMap::new();
    // Some packed executables don't seem to have the data directory bit.
    if file.header.SizeOfOptionalHeader > 8 {
        const IMAGE_DIRECTORY_ENTRY_IMPORT: usize = 1;
        let imports_data = &file.data_directory[IMAGE_DIRECTORY_ENTRY_IMPORT];
        pe::parse_imports(
            &mut machine.x86.mem[base as usize..],
            imports_data.VirtualAddress as usize,
            |dll, sym, iat_addr| {
                let name = format!("{}!{}", dll, sym.to_string());
                labels.insert(base + iat_addr, format!("{}@IAT", name));

                let handler = winapi::resolve(dll, &sym);
                let addr = machine.shims.add(name.clone(), handler);
                labels.insert(addr, name);

                addr
            },
        )?;

        const IMAGE_DIRECTORY_ENTRY_RESOURCE: usize = 2;
        let res_data = &file.data_directory[IMAGE_DIRECTORY_ENTRY_RESOURCE];
        machine.state.user32.resources_base = res_data.VirtualAddress;
    }

    let entry_point = base + file.opt_header.AddressOfEntryPoint;
    machine.x86.regs.eip = entry_point;

    Ok(labels)
}
