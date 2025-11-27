use crate::game;
use memory::Extensions;
use win32::Machine;
use win32_winapi::calling_convention::{ABIReturn, FromStack};
use x86::Register;

/// Ghidra's default base address for executables
pub const GHIDRA_IMAGE_BASE: u32 = 0x00400000;

/// Translate a Ghidra address to the actual runtime address
///
/// Ghidra shows addresses relative to the preferred base (usually 0x00400000),
/// but the actual loaded address might be different. This function performs:
/// runtime_addr = actual_base + (ghidra_addr - GHIDRA_IMAGE_BASE)
pub fn translate_address(machine: &Machine, ghidra_addr: u32) -> u32 {
    let state = win32::kernel32::get_state(machine);
    let actual_base = state.image_base;
    let offset = ghidra_addr.wrapping_sub(GHIDRA_IMAGE_BASE);
    let runtime_addr = actual_base.wrapping_add(offset);

    log::debug!(
        "Address translation: Ghidra {:#x} -> Runtime {:#x} (base: {:#x}, offset: {:#x})",
        ghidra_addr,
        runtime_addr,
        actual_base,
        offset
    );

    runtime_addr
}

macro_rules! hook {
    // ==================== STDCALL ====================

    // stdcall with no arguments
    ($machine:expr, $addr:expr, $func:path, stdcall) => {{
        let addr = $crate::hooks::translate_address($machine, $addr);
        log::info!("Installing hook at {:#x}", addr);
        $machine.add_function_hook(addr, |machine: &mut Machine| {
            let cpu = machine.emu.x86.cpu_mut();
            let mem = machine.memory.mem();
            let esp = cpu.regs.get32(Register::ESP);
            let return_addr = mem.get_pod::<u32>(esp);

            // Call the Rust function
            let return_value = $func();

            match ABIReturn::from(return_value) {
                ABIReturn::U32(value) => cpu.regs.set32(Register::EAX, value),
                _ => panic!("Unsupported return type"),
            }

            // Clean up stack (stdcall: pop return address, no args)
            cpu.regs.set32(Register::ESP, esp + 4);

            // Jump to return address
            cpu.regs.eip = return_addr;
        });
    }};

    // ==================== CDECL ======================

    // cdecl with single + variadic argument
    ($machine:expr, $addr:expr, $func:path, cdecl, $arg1_type:ty, ...) => {{
        let addr = $crate::hooks::translate_address($machine, $addr);
        log::info!("Installing hook at {:#x}", addr);
        $machine.add_function_hook(addr, |machine: &mut Machine| {
            let cpu = machine.emu.x86.cpu_mut();
            let mem = machine.memory.mem();
            let esp = cpu.regs.get32(Register::ESP);
            let return_addr = mem.get_pod::<u32>(esp);

            let arg1 = <$arg1_type>::from_stack(mem, esp + 4);

            // Call the Rust function
            let return_value = $func(arg1, (mem, esp + 8));

            match ABIReturn::from(return_value) {
                ABIReturn::U32(value) => cpu.regs.set32(Register::EAX, value),
                _ => panic!("Unsupported return type"),
            }

            // Clean up stack (cdecl: pop return address)
            cpu.regs.set32(Register::ESP, esp + 4);

            // Jump to return address
            cpu.regs.eip = return_addr;
        });
    }};

    // cdecl with single argument
    ($machine:expr, $addr:expr, $func:path, cdecl, $arg1_type:ty) => {{
        let addr = $crate::hooks::translate_address($machine, $addr);
        log::info!("Installing hook at {:#x}", addr);
        $machine.add_function_hook(addr, |machine: &mut Machine| {
            let cpu = machine.emu.x86.cpu_mut();
            let mem = machine.memory.mem();
            let esp = cpu.regs.get32(Register::ESP);
            let return_addr = mem.get_pod::<u32>(esp);

            let arg1 = <$arg1_type>::from_stack(mem, esp + 4);

            // Call the Rust function
            let return_value = $func(arg1);

            match ABIReturn::from(return_value) {
                ABIReturn::U32(value) => cpu.regs.set32(Register::EAX, value),
                _ => panic!("Unsupported return type"),
            }

            // Clean up stack (cdecl: pop return address)
            cpu.regs.set32(Register::ESP, esp + 4);

            // Jump to return address
            cpu.regs.eip = return_addr;
        });
    }};

    // cdecl with two arguments
    ($machine:expr, $addr:expr, $func:path, cdecl, $arg1_type:ty, $arg2_type:ty) => {{
        let addr = $crate::hooks::translate_address($machine, $addr);
        log::info!("Installing hook at {:#x}", addr);
        $machine.add_function_hook(addr, |machine: &mut Machine| {
            let cpu = machine.emu.x86.cpu_mut();
            let mem = machine.memory.mem();
            let esp = cpu.regs.get32(Register::ESP);
            let return_addr = mem.get_pod::<u32>(esp);

            let arg1 = <$arg1_type>::from_stack(mem, esp + 4);
            let arg2 = <$arg2_type>::from_stack(mem, esp + 8);

            // Call the Rust function
            let return_value = $func(arg1, arg2);

            match ABIReturn::from(return_value) {
                ABIReturn::U32(value) => cpu.regs.set32(Register::EAX, value),
                _ => panic!("Unsupported return type"),
            }

            // Clean up stack (cdecl: pop return address)
            cpu.regs.set32(Register::ESP, esp + 4);

            // Jump to return address
            cpu.regs.eip = return_addr;
        });
    }};
}

/// Install all Deimos Rising game-specific hooks
#[rustfmt::skip]
pub fn install_hooks(machine: &mut Machine) {
    hook!(machine, 0x004655e0, game::quicktime::initialize_qtml, stdcall);
    hook!(machine, 0x0046d140, game::quicktime::enter_movies, stdcall);
    hook!(machine, 0x00465880, game::quicktime::open_a_default_component, stdcall);
    hook!(machine, 0x00465860, game::quicktime::close_component, stdcall);

    hook!(machine, 0x00463450, game::init::win95_allow_one_instance, cdecl, u32);
    hook!(machine, 0x00462fa0, game::init::get_directx_version, stdcall);

    hook!(machine, 0x00450610, game::app::app_log, cdecl, Option<&str>, ...);

    hook!(machine, 0x0044d5f0, game::mem::clear, cdecl, Option<&mut [u8]>);

    fn pak_tag_get_info_from_file_name(name: Option<&str>, tag: Option<&mut game::pak::Tag>) -> bool {
        tag.unwrap().get_info_from_file_name(name.unwrap())
    }
    hook!(machine, 0x004038b0, pak_tag_get_info_from_file_name, cdecl, Option<&str>, Option<&mut game::pak::Tag>);
}
