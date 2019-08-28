// In this example we execute a contract funciton exported as "_call"

extern crate metered_wasmi;

use std::env::args;
use std::fs::File;
use metered_wasmi::{ImportsBuilder, Module, ModuleInstance, RuntimeValue, isa, Trap, TrapKind, RuntimeArgs, Externals};

fn load_from_file(filename: &str) -> Module {
    use std::io::prelude::*;
    let mut file = File::open(filename).unwrap();
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();
    Module::from_buffer(buf).unwrap()
}

struct HostExternals {
    gas: u32,
}

impl Externals for HostExternals {
    fn invoke_index(
        &mut self,
        _index: usize,
        _args: RuntimeArgs,
        ) -> Result<Option<RuntimeValue>, Trap> {
        Err(TrapKind::Unreachable.into())
    }

    fn use_gas(
        &mut self,
        _instruction: &isa::Instruction
        ) -> Result<(), TrapKind> {
            Ok(self.gas = self.gas - 1)
    }
}


fn main() {
    let args: Vec<_> = args().collect();
    if args.len() != 3 {
        println!("Usage: {} <wasm file> <arg>", args[0]);
        println!("    wasm file should contain exported `_call` function with single I32 argument");
        return;
    }

    // Here we load module using dedicated for this purpose
    // `load_from_file` function (which works only with modules)
    let module = load_from_file(&args[1]);

    // Intialize deserialized module. It adds module into It expects 3 parameters:
    // - a name for the module
    // - a module declaration
    // - "main" module doesn't import native module(s) this is why we don't need to provide external native modules here
    // This test shows how to implement native module https://github.com/NikVolf/parity-wasm/blob/master/src/interpreter/tests/basics.rs#L197
    let gas_limit = 4;
    let mut host_externals = HostExternals{gas: gas_limit};
    let main = ModuleInstance::new(&module, &ImportsBuilder::default())
        .expect("Failed to instantiate module")
        .run_start(&mut host_externals)
        .expect("Failed to run start function in module");

    // The argument should be parsable as a valid integer
    let argument: i32 = args[2].parse().expect("Integer argument required");

    let result = main.invoke_export("_call", &[RuntimeValue::I32(argument)], &mut host_externals);
    // "_call" export of function to be executed with an i32 argument and prints the result of execution
    println!(
        "Result: {:?}\nGas Used: {:?}",
        result,
        gas_limit - host_externals.gas,
    );
}
