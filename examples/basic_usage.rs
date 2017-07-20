extern crate nvml_wrapper as nvml;
// This is used to pretty-print bytes. Nothing you need to be concerned about.
extern crate number_prefix;

use nvml::NVML;
// You would probably want your own error setup in your own code; here we just
// use the wrapper's error types.
use nvml::error::*;
use nvml::enum_wrappers::device::{TemperatureSensor, Clock};
use number_prefix::{decimal_prefix, Standalone, Prefixed};

fn main() {
    match actual_main() {
        Ok(()) => {},
        Err(e) => println!("\n{:?}. \nDescription: {:?}\n", e, e.description())
    }
}

// We write a function so that we can return a `Result` and use `?`
fn actual_main() -> Result<()> {
    let nvml = NVML::init()?;

    // Grabbing the first device in the system, whichever one that is.
    // If you want to ensure you get the same physical device across reboots,
    // get devices via UUID or PCI bus IDs.
    let device = nvml.device_by_index(0)?;

    // Now we can do whatever we want, like getting some data...
    let name = device.name()?;
    let temperature = device.temperature(TemperatureSensor::Gpu)?;
    let mem_info = device.memory_info()?;
    let graphics_clock = device.clock_info(Clock::Graphics)?;
    let mem_clock = device.clock_info(Clock::Memory)?;
    let link_gen = device.current_pcie_link_gen()?;
    let link_width = device.current_pcie_link_width()?;
    let max_link_gen = device.max_pcie_link_gen()?;
    let max_link_width = device.max_pcie_link_width()?;

    // And we can use that data (here we just print it)
    print!("\n\n");
    println!(
        "Your {name} is currently sitting at {temperature} °C with a \
        graphics clock of {graphics_clock} MHz and a memory clock of {mem_clock} \
        MHz. Memory usage is {used_mem} out of an available {total_mem}. \
        Right now the device is connected via a PCIe gen {link_gen} x{link_width} \
        interface; the max your hardware supports is PCIe gen {max_link_gen} \
        x{max_link_width}.",
        name=name, temperature=temperature, graphics_clock=graphics_clock,
        mem_clock=mem_clock, used_mem=format_bytes(mem_info.used),
        total_mem=format_bytes(mem_info.total), link_gen=link_gen,
        link_width=link_width, max_link_gen=max_link_gen,
        max_link_width=max_link_width
    );

    if device.is_multi_gpu_board()? {
        println!();
        println!("This device is on a multi-GPU board.")
    } else {
        println!();
        println!("This device is not on a multi-GPU board.")
    }

    print!("\n\n");
    Ok(())
}

// Function used to pretty-format bytes. Not relevant to this library.
fn format_bytes(num: u64) -> String {
    match decimal_prefix(num as f32) {
        Standalone(bytes)   => bytes.to_string(),
        Prefixed(prefix, n) => format!("{:.2} {}B", n, prefix)
    }
}
