use crate::game;
use win32::Machine;
use memory::Extensions;
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
        ghidra_addr, runtime_addr, actual_base, offset
    );

    runtime_addr
}

macro_rules! hook {
    // ==================== STDCALL ====================

    // stdcall with no arguments, void return
    ($machine:expr, $addr:expr, $func:path, stdcall, ()) => {{
        let addr =  $crate::hooks::translate_address($machine, $addr);
        log::info!("Installing hook at {:#x}", addr);
        $machine.add_function_hook(addr, |machine: &mut Machine| -> bool {
            let cpu = machine.emu.x86.cpu_mut();
            let mem = machine.memory.mem();
            let esp = cpu.regs.get32(Register::ESP);
            let return_addr = mem.get_pod::<u32>(esp);

            // Call the Rust function
            $func();

            // Clean up stack (stdcall: pop return address, no args)
            cpu.regs.set32(Register::ESP, esp.wrapping_add(4));

            // Jump to return address
            cpu.regs.eip = return_addr;

            true
        });
    }};

    // stdcall with no arguments, returns u32
    ($machine:expr, $addr:expr, $func:path, stdcall, u32) => {{
        let addr =  $crate::hooks::translate_address($machine, $addr);
        log::info!("Installing hook at {:#x}", addr);
        $machine.add_function_hook(addr, |machine: &mut Machine| -> bool {
            let cpu = machine.emu.x86.cpu_mut();
            let mem = machine.memory.mem();
            let esp = cpu.regs.get32(Register::ESP);
            let return_addr = mem.get_pod::<u32>(esp);

            // Call the Rust function
            let return_value: u32 = $func();

            // Set return value in EAX
            cpu.regs.set32(Register::EAX, return_value);

            // Clean up stack (stdcall: pop return address, no args)
            cpu.regs.set32(Register::ESP, esp + 4);

            // Jump to return address
            cpu.regs.eip = return_addr;

            true
        });
    }};

    // ==================== CDECL ======================

    // cdecl with single argument, returns u32
    ($machine:expr, $addr:expr, $func:path, cdecl, u32, $arg_type:ty) => {{
        let addr =  $crate::hooks::translate_address($machine, $addr);
        log::info!("Installing hook at {:#x}", addr);
        $machine.add_function_hook(addr, |machine: &mut Machine| -> bool {
            let cpu = machine.emu.x86.cpu_mut();
            let mem = machine.memory.mem();
            let esp = cpu.regs.get32(Register::ESP);
            let return_addr = mem.get_pod::<u32>(esp);
            let arg = mem.get_pod::<$arg_type>(esp + 4);

            // Call the Rust function
            let return_value: u32 = $func(arg);

            // Set return value in EAX
            cpu.regs.set32(Register::EAX, return_value);

            // Clean up stack (cdecl: pop return address)
            cpu.regs.set32(Register::ESP, esp + 4);

            // Jump to return address
            cpu.regs.eip = return_addr;

            true
        });
    }};
}

/// Install all Deimos Rising game-specific hooks
pub fn install_hooks(machine: &mut Machine) {
    hook!(machine, 0x00463450, game::init::win95_allow_one_instance, cdecl, u32, u32);
    hook!(machine, 0x004655e0, game::init::initialize_qtml, stdcall, u32);
    hook!(machine, 0x0046d140, game::init::enter_movies, stdcall, ());
    hook!(machine, 0x00462fa0, game::init::get_directx_version, stdcall, u32);
    hook!(machine, 0x00465880, game::init::open_a_default_component, stdcall, u32);
}
