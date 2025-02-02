use std::{convert::TryInto, future::Future, sync::Arc, time::Duration};

use anyhow::{anyhow, Result};
use wasmtime::{Caller, FuncType, Linker, Trap, Val, ValType};

use super::{
    get_memory, link_async1_if_match, link_async2_if_match, link_async4_if_match,
    link_async6_if_match, link_async7_if_match, link_if_match,
};
use crate::{
    api::error::IntoTrap,
    module::Module,
    process::{Signal, WasmProcess},
    state::ProcessState,
    EnvConfig, Environment,
};

// Register the process APIs to the linker
pub(crate) fn register(
    linker: &mut Linker<ProcessState>,
    namespace_filter: &[String],
) -> Result<()> {
    link_if_match(
        linker,
        "lunatic::process",
        "create_config",
        FuncType::new([ValType::I64, ValType::I64], [ValType::I64]),
        create_config,
        namespace_filter,
    )?;
    link_if_match(
        linker,
        "lunatic::process",
        "drop_config",
        FuncType::new([ValType::I64], []),
        drop_config,
        namespace_filter,
    )?;
    link_if_match(
        linker,
        "lunatic::process",
        "allow_namespace",
        FuncType::new([ValType::I64, ValType::I32, ValType::I32], []),
        allow_namespace,
        namespace_filter,
    )?;
    link_if_match(
        linker,
        "lunatic::process",
        "create_environment",
        FuncType::new([ValType::I64, ValType::I32], [ValType::I32]),
        create_environment,
        namespace_filter,
    )?;
    link_if_match(
        linker,
        "lunatic::process",
        "drop_environment",
        FuncType::new([ValType::I64], []),
        drop_environment,
        namespace_filter,
    )?;
    link_if_match(
        linker,
        "lunatic::process",
        "add_plugin",
        FuncType::new(
            [ValType::I64, ValType::I32, ValType::I32, ValType::I32],
            [ValType::I32],
        ),
        add_plugin,
        namespace_filter,
    )?;
    link_async4_if_match(
        linker,
        "lunatic::process",
        "add_module",
        FuncType::new(
            [ValType::I64, ValType::I32, ValType::I32, ValType::I32],
            [ValType::I32],
        ),
        add_module,
        namespace_filter,
    )?;
    link_async2_if_match(
        linker,
        "lunatic::process",
        "add_this_module",
        FuncType::new([ValType::I64, ValType::I32], [ValType::I32]),
        add_this_module,
        namespace_filter,
    )?;
    link_if_match(
        linker,
        "lunatic::process",
        "drop_module",
        FuncType::new([ValType::I64], []),
        drop_module,
        namespace_filter,
    )?;
    link_async7_if_match(
        linker,
        "lunatic::process",
        "spawn",
        FuncType::new(
            [
                ValType::I64,
                ValType::I64,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
            ],
            [ValType::I32],
        ),
        spawn,
        namespace_filter,
    )?;
    link_async6_if_match(
        linker,
        "lunatic::process",
        "inherit_spawn",
        FuncType::new(
            [
                ValType::I64,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
            ],
            [ValType::I32],
        ),
        inherit_spawn,
        namespace_filter,
    )?;
    link_if_match(
        linker,
        "lunatic::process",
        "drop_process",
        FuncType::new([ValType::I64], []),
        drop_process,
        namespace_filter,
    )?;
    link_if_match(
        linker,
        "lunatic::process",
        "clone_process",
        FuncType::new([ValType::I64], [ValType::I64]),
        clone_process,
        namespace_filter,
    )?;
    link_async1_if_match(
        linker,
        "lunatic::process",
        "sleep_ms",
        FuncType::new([ValType::I64], []),
        sleep_ms,
        namespace_filter,
    )?;
    link_if_match(
        linker,
        "lunatic::process",
        "die_when_link_dies",
        FuncType::new([ValType::I32], []),
        die_when_link_dies,
        namespace_filter,
    )?;
    link_if_match(
        linker,
        "lunatic::process",
        "this",
        FuncType::new([], [ValType::I64]),
        this,
        namespace_filter,
    )?;
    link_if_match(
        linker,
        "lunatic::process",
        "id",
        FuncType::new([ValType::I64, ValType::I32], []),
        id,
        namespace_filter,
    )?;
    link_if_match(
        linker,
        "lunatic::process",
        "this_env",
        FuncType::new([], [ValType::I64]),
        this_env,
        namespace_filter,
    )?;
    link_if_match(
        linker,
        "lunatic::process",
        "link",
        FuncType::new([ValType::I64, ValType::I64], []),
        link,
        namespace_filter,
    )?;
    link_if_match(
        linker,
        "lunatic::process",
        "unlink",
        FuncType::new([ValType::I64], []),
        unlink,
        namespace_filter,
    )?;
    link_if_match(
        linker,
        "lunatic::process",
        "register",
        FuncType::new(
            [
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I64,
                ValType::I64,
            ],
            [ValType::I32],
        ),
        register_proc,
        namespace_filter,
    )?;
    link_if_match(
        linker,
        "lunatic::process",
        "unregister",
        FuncType::new(
            [
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I64,
            ],
            [ValType::I32],
        ),
        unregister,
        namespace_filter,
    )?;
    link_if_match(
        linker,
        "lunatic::process",
        "lookup",
        FuncType::new(
            [
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
            ],
            [ValType::I32],
        ),
        lookup,
        namespace_filter,
    )?;
    Ok(())
}

//% lunatic::process::create_config(max_memory: u64, max_fuel: u64) -> u64
//%
//% * **max_memory** - Maximum amount of memory in bytes.
//% * **max_fuel**   - Maximum amount of instructions in gallons that processes will be able to run
//%                    before it traps. 1 gallon ~= 10k instructions. The special value of `0` means
//%                    unlimited instructions.
//% * Returns ID of newly created configuration.
//%
//% Create a new configuration for an environment.
fn create_config(mut caller: Caller<ProcessState>, max_memory: u64, max_fuel: u64) -> u64 {
    let max_fuel = if max_fuel != 0 { Some(max_fuel) } else { None };
    let config = EnvConfig::new(max_memory as usize, max_fuel);
    caller.data_mut().resources.configs.add(config)
}

//% lunatic::process::drop_config(config_id: u64)
//%
//% Drops the config resource.
//%
//% Traps:
//% * If the config ID doesn't exist.
fn drop_config(mut caller: Caller<ProcessState>, config_id: u64) -> Result<(), Trap> {
    caller
        .data_mut()
        .resources
        .configs
        .remove(config_id)
        .or_trap("lunatic::process::drop_config")?;
    Ok(())
}

//% lunatic::process::allow_namespace(config_id: u64, namespace_str_ptr: u32, namespace_str_len: u32)
//%
//% Allow using host functions under this namespace with this configuration. Namespaces are strings,
//% e.g. `lunatic::` or `lunatic::process::`.
//%
//% Traps:
//% * If the namespace string is not a valid utf8 string.
//% * If **namespace_str_ptr + namespace_str_len** is outside the memory.
fn allow_namespace(
    mut caller: Caller<ProcessState>,
    config_id: u64,
    namespace_str_ptr: u32,
    namespace_str_len: u32,
) -> Result<(), Trap> {
    let memory = get_memory(&mut caller)?;
    let mut buffer = vec![0; namespace_str_len as usize];
    memory
        .read(&caller, namespace_str_ptr as usize, &mut buffer)
        .or_trap("lunatic::process::allow_namespace")?;
    let namespace =
        std::str::from_utf8(buffer.as_slice()).or_trap("lunatic::process::allow_namespace")?;
    let config = caller
        .data_mut()
        .resources
        .configs
        .get_mut(config_id)
        .or_trap("lunatic::process::allow_namespace")?;
    config.allow_namespace(namespace);
    Ok(())
}

//% lunatic::process::add_plugin(
//%     config_id: u64,
//%     plugin_data_ptr: u32,
//%     plugin_data_len: u32,
//%     id_ptr: u32
//% ) -> u32
//%
//% Returns:
//% * 0 on success
//% * 1 on error   - The error ID is written to **id_ptr**
//%
//% Add plugin to environment configuration.
//%
//% Traps:
//% * If the config ID doesn't exist.
//% * If **id_ptr** is outside the memory.
//% * If **plugin_data_ptr + plugin_data_len** is outside the memory.
fn add_plugin(
    mut caller: Caller<ProcessState>,
    config_id: u64,
    plugin_data_ptr: u32,
    plugin_data_len: u32,
    id_ptr: u32,
) -> Result<u32, Trap> {
    let mut plugin = vec![0; plugin_data_len as usize];
    let memory = get_memory(&mut caller)?;
    memory
        .read(&caller, plugin_data_ptr as usize, plugin.as_mut_slice())
        .or_trap("lunatic::process::add_plugin")?;

    let config = caller
        .data_mut()
        .resources
        .configs
        .get_mut(config_id)
        .or_trap("lunatic::process::add_plugin")?;

    let (error_id, result) = match config.add_plugin(plugin) {
        Ok(()) => (0, 0),
        Err(error) => (caller.data_mut().errors.add(error), 1),
    };

    let memory = get_memory(&mut caller)?;
    memory
        .write(&mut caller, id_ptr as usize, &error_id.to_le_bytes())
        .or_trap("lunatic::process::add_plugin")?;

    Ok(result)
}

//% lunatic::process::create_environment(config_id: u64, id_ptr: u32) -> u32
//%
//% Returns:
//% * 0 on success - The ID of the newly created environment is written to **id_ptr**
//% * 1 on error   - The error ID is written to **id_ptr**
//%
//% Consumes the config and creates a new environment from it.
//%
//% Traps:
//% * If the config ID doesn't exist.
//% * If **id_ptr** is outside the memory.
fn create_environment(
    mut caller: Caller<ProcessState>,
    config_id: u64,
    id_ptr: u32,
) -> Result<u32, Trap> {
    let config = caller
        .data_mut()
        .resources
        .configs
        .get(config_id)
        .or_trap("lunatic::process::create_environment")?
        .clone();

    let (env_or_error_id, result) = match Environment::new(config) {
        Ok(env) => (caller.data_mut().resources.environments.add(env), 0),
        Err(error) => (caller.data_mut().errors.add(error), 1),
    };

    let memory = get_memory(&mut caller)?;
    memory
        .write(&mut caller, id_ptr as usize, &env_or_error_id.to_le_bytes())
        .or_trap("lunatic::process::create_environment")?;

    Ok(result)
}

//% lunatic::process::drop_environment(env_id: u64)
//%
//% Drops the environment resource.
//%
//% Traps:
//% * If the environment ID doesn't exist.
fn drop_environment(mut caller: Caller<ProcessState>, env_id: u64) -> Result<(), Trap> {
    caller
        .data_mut()
        .resources
        .environments
        .remove(env_id)
        .or_trap("lunatic::process::drop_environment")?;
    Ok(())
}

//% lunatic::process::add_module(
//%     env_id: u64,
//%     module_data_ptr: u32,
//%     module_data_len: u32,
//%     id_ptr: u32
//% ) -> u64
//%
//% Returns:
//% * 0 on success - The ID of the newly created module is written to **id_ptr**
//% * 1 on error   - The error ID is written to **id_ptr**
//%
//% Adds a module to the environment. This function will also JIT compile the module.
//%
//% Traps:
//% * If the env ID doesn't exist.
//% * If **module_data_ptr + module_data_len** is outside the memory.
//% * If **id_ptr** is outside the memory.
fn add_module(
    mut caller: Caller<ProcessState>,
    env_id: u64,
    module_data_ptr: u32,
    module_data_len: u32,
    id_ptr: u32,
) -> Box<dyn Future<Output = Result<u32, Trap>> + Send + '_> {
    Box::new(async move {
        let mut module = vec![0; module_data_len as usize];
        let memory = get_memory(&mut caller)?;
        memory
            .read(&caller, module_data_ptr as usize, module.as_mut_slice())
            .or_trap("lunatic::process::add_module")?;
        let env = caller
            .data_mut()
            .resources
            .environments
            .get_mut(env_id)
            .or_trap("lunatic::process::add_module")?;
        let (mod_or_error_id, result) = match env.create_module(module).await {
            Ok(module) => (caller.data_mut().resources.modules.add(module), 0),
            Err(error) => (caller.data_mut().errors.add(error), 1),
        };
        memory
            .write(&mut caller, id_ptr as usize, &mod_or_error_id.to_le_bytes())
            .or_trap("lunatic::process::add_module")?;
        Ok(result)
    })
}

//% lunatic::process::add_this_module(
//%     env_id: u64,
//%     id_ptr: u32,
//% ) -> u32
//%
//% Returns:
//% * 0 on success - The ID of the newly created module is written to **id_ptr**
//% * 1 on error   - The error ID is written to **id_ptr**
//%
//% Adds the module, that the currently running process was spawned from, to a new environment.
//% This function will also JIT compile the module.
//%
//% Traps:
//% * If the env ID doesn't exist.
//% * If **id_ptr** is outside the memory.
fn add_this_module(
    mut caller: Caller<ProcessState>,
    env_id: u64,
    id_ptr: u32,
) -> Box<dyn Future<Output = Result<u32, Trap>> + Send + '_> {
    Box::new(async move {
        let module = caller.data().module.clone();
        let env = caller
            .data_mut()
            .resources
            .environments
            .get_mut(env_id)
            .or_trap("lunatic::process::add_this_module")?;
        let (mod_or_error_id, result) = match env.create_module(module.data()).await {
            Ok(module) => (caller.data_mut().resources.modules.add(module), 0),
            Err(error) => (caller.data_mut().errors.add(error), 1),
        };
        let memory = get_memory(&mut caller)?;
        memory
            .write(&mut caller, id_ptr as usize, &mod_or_error_id.to_le_bytes())
            .or_trap("lunatic::process::add_this_module")?;
        Ok(result)
    })
}

//% lunatic::process::drop_module(mod_id: i64)
//%
//% Drops the module resource.
//%
//% Traps:
//% * If the module ID doesn't exist.
fn drop_module(mut caller: Caller<ProcessState>, mod_id: u64) -> Result<(), Trap> {
    caller
        .data_mut()
        .resources
        .modules
        .remove(mod_id)
        .or_trap("lunatic::process::drop_module")?;
    Ok(())
}

//% lunatic::process::spawn(
//%     link: i64,
//%     module_id: u64,
//%     func_str_ptr: u32,
//%     func_str_len: u32,
//%     params_ptr: u32,
//%     params_len: u32,
//%     id_u64_ptr: u32
//% ) -> u32
//%
//% Returns:
//% * 0 on success - The ID of the newly created process is written to **id_u64_ptr**
//% * 1 on error   - The error ID is written to **id_u64_ptr**
//%
//% Spawns a new process using the passed in function inside a module as the entry point.
//% If **link** is not 0, it will link the child and parent processes. The value
//% of the **link** argument will be used as the link-tag for the child.
//%
//%
//% The function arguments are passed as an array with the following structure:
//% [0 byte = type ID; 1..17 bytes = value as u128, ...]
//% The type ID follows the WebAssembly binary convention:
//%  - 0x7F => i32
//%  - 0x7E => i64
//%  - 0x7B => v128
//% If any other value is used as type ID, this function will trap.
//%
//% Traps:
//% * If the module ID doesn't exist.
//% * If the function string is not a valid utf8 string.
//% * If the params array is in a wrong format.
//% * If **func_str_ptr + func_str_len** is outside the memory.
//% * If **params_ptr + params_len** is outside the memory.
//% * If **id_ptr** is outside the memory.
#[allow(clippy::too_many_arguments)]
fn spawn(
    mut caller: Caller<ProcessState>,
    link: i64,
    module_id: u64,
    func_str_ptr: u32,
    func_str_len: u32,
    params_ptr: u32,
    params_len: u32,
    id_ptr: u32,
) -> Box<dyn Future<Output = Result<u32, Trap>> + Send + '_> {
    Box::new(async move {
        let module = caller
            .data()
            .resources
            .modules
            .get(module_id)
            .or_trap("lunatic::process::spawn")?
            .clone();
        spawn_from_module(
            &mut caller,
            link,
            module,
            func_str_ptr,
            func_str_len,
            params_ptr,
            params_len,
            id_ptr,
        )
        .await
    })
}

//% lunatic::process::inherit_spawn(
//%     link: i64,
//%     func_str_ptr: u32,
//%     func_str_len: u32,
//%     params_ptr: u32,
//%     params_len: u32,
//%     id_ptr: u32
//% ) -> u32
//%
//% Returns:
//% * 0 on success - The ID of the newly created process is written to **id_ptr**
//% * 1 on error   - The error ID is written to **id_ptr**
//%
//% Spawns a new process using the same module as the parent.
//% If **link** is not 0, it will link the child and parent processes. The value
//% of the **link** argument will be used as the link-tag for the child.
//%
//% The function arguments are passed as an array with the following structure:
//% [0 byte = type ID; 1..17 bytes = value as u128, ...]
//% The type ID follows the WebAssembly binary convention:
//%  - 0x7F => i32
//%  - 0x7E => i64
//%  - 0x7B => v128
//% If any other value is used as type ID, this function will trap.
//%
//% Traps:
//% * If the function string is not a valid utf8 string.
//% * If the params array is in a wrong format.
//% * If **func_str_ptr + func_str_len** is outside the memory.
//% * If **params_ptr + params_len** is outside the memory.
//% * If **id_ptr** is outside the memory.
fn inherit_spawn(
    mut caller: Caller<ProcessState>,
    link: i64,
    func_str_ptr: u32,
    func_str_len: u32,
    params_ptr: u32,
    params_len: u32,
    id_ptr: u32,
) -> Box<dyn Future<Output = Result<u32, Trap>> + Send + '_> {
    Box::new(async move {
        let module = caller.data().module.clone();
        spawn_from_module(
            &mut caller,
            link,
            module,
            func_str_ptr,
            func_str_len,
            params_ptr,
            params_len,
            id_ptr,
        )
        .await
    })
}

#[allow(clippy::too_many_arguments)]
async fn spawn_from_module(
    mut caller: &mut Caller<'_, ProcessState>,
    link: i64,
    module: Module,
    func_str_ptr: u32,
    func_str_len: u32,
    params_ptr: u32,
    params_len: u32,
    id_ptr: u32,
) -> Result<u32, Trap> {
    let memory = get_memory(&mut caller)?;
    let func_str = memory
        .data(&caller)
        .get(func_str_ptr as usize..(func_str_ptr + func_str_len) as usize)
        .or_trap("lunatic::process::(inherit_)spawn")?;
    let function = std::str::from_utf8(func_str).or_trap("lunatic::process::(inherit_)spawn")?;
    let params = memory
        .data(&caller)
        .get(params_ptr as usize..(params_ptr + params_len) as usize)
        .or_trap("lunatic::process::(inherit_)spawn")?;
    let params = params
        .chunks_exact(17)
        .map(|chunk| {
            let value = u128::from_le_bytes(chunk[1..].try_into()?);
            let result = match chunk[0] {
                0x7F => Val::I32(value as i32),
                0x7E => Val::I64(value as i64),
                0x7B => Val::V128(value),
                _ => return Err(anyhow!("Unsupported type ID")),
            };
            Ok(result)
        })
        .collect::<Result<Vec<_>>>()?;
    // Should processes be linked together?
    let link = match link {
        0 => None,
        tag => {
            let id = caller.data().id;
            let signal_mailbox = caller.data().signal_mailbox.clone();
            let process = WasmProcess::new(id, signal_mailbox);
            Some((Some(tag), process))
        }
    };
    let (proc_or_error_id, result) = match module.spawn(function, params, link).await {
        Ok((_, process)) => (
            caller.data_mut().resources.processes.add(Arc::new(process)),
            0,
        ),
        Err(error) => (caller.data_mut().errors.add(error), 1),
    };
    memory
        .write(
            &mut caller,
            id_ptr as usize,
            &proc_or_error_id.to_le_bytes(),
        )
        .or_trap("lunatic::process::(inherit_)spawn")?;
    Ok(result)
}

//% lunatic::process::drop_process(process_id: u64)
//%
//% Drops the process handle. This will not kill the process, it just removes the handle that
//% references the process and allows us to send messages and signals to it.
//%
//% Traps:
//% * If the process ID doesn't exist.
fn drop_process(mut caller: Caller<ProcessState>, process_id: u64) -> Result<(), Trap> {
    caller
        .data_mut()
        .resources
        .processes
        .remove(process_id)
        .or_trap("lunatic::process::drop_process")?;
    Ok(())
}

//% lunatic::process::clone_process(process_id: u64) -> u64
//%
//% Clones a process returning the ID of the clone.
//%
//% Traps:
//% * If the process ID doesn't exist.
fn clone_process(mut caller: Caller<ProcessState>, process_id: u64) -> Result<u64, Trap> {
    let process = caller
        .data()
        .resources
        .processes
        .get(process_id)
        .or_trap("lunatic::process::clone_process")?
        .clone();
    let id = caller.data_mut().resources.processes.add(process);
    Ok(id)
}

//% lunatic::process::sleep_ms(millis: u64)
//%
//% Suspend process for `millis`.
fn sleep_ms(_: Caller<ProcessState>, millis: u64) -> Box<dyn Future<Output = ()> + Send + '_> {
    Box::new(async move {
        async_std::task::sleep(Duration::from_millis(millis)).await;
    })
}

//% lunatic::process::die_when_link_dies(trap: u32)
//%
//% Defines what happens to this process if one of the linked processes notifies us that it died.
//%
//% There are 2 options:
//% 1. `trap == 0` the received signal will be turned into a signal message and put into the mailbox.
//% 2. `trap != 0` the process will die and notify all linked processes of its death.
//%
//% The default behaviour for a newly spawned process is 2.
fn die_when_link_dies(mut caller: Caller<ProcessState>, trap: u32) {
    caller
        .data_mut()
        .signal_mailbox
        .try_send(Signal::DieWhenLinkDies(trap != 0))
        .expect("The signal is sent to itself and the receiver must exist at this point");
}

//% lunatic::process::this() -> u64
//%
//% Create a process handle to itself and return resource ID.
fn this(mut caller: Caller<ProcessState>) -> u64 {
    let id = caller.data().id;
    let signal_mailbox = caller.data().signal_mailbox.clone();
    let process = WasmProcess::new(id, signal_mailbox);
    caller.data_mut().resources.processes.add(Arc::new(process))
}

//% lunatic::process::id(process_id: u64, u128_ptr: u32)
//%
//% Returns UUID of a process as u128_ptr.
//%
//% Traps:
//% * If the process ID doesn't exist.
//% * If **u128_ptr** is outside the memory space.
fn id(mut caller: Caller<ProcessState>, process_id: u64, u128_ptr: u32) -> Result<(), Trap> {
    let id = caller
        .data()
        .resources
        .processes
        .get(process_id)
        .or_trap("lunatic::process::id")?
        .id()
        .as_u128();
    let memory = get_memory(&mut caller)?;
    memory
        .write(&mut caller, u128_ptr as usize, &id.to_le_bytes())
        .or_trap("lunatic::process::id")?;
    Ok(())
}

//% lunatic::process::this_env() -> u64
//%
//% Returns ID of the environment that this process was spawned from.
fn this_env(mut caller: Caller<ProcessState>) -> u64 {
    let env = caller.data().module.environment().clone();
    caller.data_mut().resources.environments.add(env)
}

//% lunatic::process::link(tag: i64, process_id: u64)
//%
//% Link current process to **process_id**. This is not an atomic operation, any of the 2 processes
//% could fail before processing the `Link` signal and may not notify the other.
//%
//% Traps:
//% * If the process ID doesn't exist.
fn link(mut caller: Caller<ProcessState>, tag: i64, process_id: u64) -> Result<(), Trap> {
    let tag = match tag {
        0 => None,
        tag => Some(tag),
    };
    // Create handle to itself
    let id = caller.data().id;
    let signal_mailbox = caller.data().signal_mailbox.clone();
    let this_process = WasmProcess::new(id, signal_mailbox);

    // Send link signal to other process
    let process = caller
        .data()
        .resources
        .processes
        .get(process_id)
        .or_trap("lunatic::process::link")?
        .clone();
    process.send(Signal::Link(tag, Arc::new(this_process)));

    // Send link signal to itself
    caller
        .data_mut()
        .signal_mailbox
        .try_send(Signal::Link(tag, process))
        .expect("The signal is sent to itself and the receiver must exist at this point");
    Ok(())
}

//% lunatic::process::unlink(process_id: u64)
//%
//% Unlink current process from **process_id**. This is not an atomic operation.
//%
//% Traps:
//% * If the process ID doesn't exist.
fn unlink(mut caller: Caller<ProcessState>, process_id: u64) -> Result<(), Trap> {
    // Create handle to itself
    let id = caller.data().id;
    let signal_mailbox = caller.data().signal_mailbox.clone();
    let this_process = WasmProcess::new(id, signal_mailbox);

    // Send unlink signal to other process
    let process = caller
        .data()
        .resources
        .processes
        .get(process_id)
        .or_trap("lunatic::process::link")?
        .clone();
    process.send(Signal::UnLink(Arc::new(this_process)));

    // Send unlink signal to itself
    caller
        .data_mut()
        .signal_mailbox
        .try_send(Signal::UnLink(process))
        .expect("The signal is sent to itself and the receiver must exist at this point");
    Ok(())
}

//% lunatic::process::register(
//%     name_ptr: u32,
//%     name_len: u32,
//%     version_ptr: u32,
//%     version_len: u32,
//%     env_id: u64
//%     process_id: u64
//%  ) -> u32
//%
//% Returns 0 in case of success or 1 if the version string didn't have a correct semver format.
//%
//% Registers process under **name** and **version** inside the specified environment. Processes
//% that are spawned into this environment can look up the process using the **lookup** function.
//%
//% Traps:
//% * If the process ID doesn't exist.
//% * If the environment ID doesn't exist.
//% * If **name_ptr + name_len** is outside the memory.
//% * If **version_ptr + version_len** is outside the memory.
fn register_proc(
    mut caller: Caller<ProcessState>,
    name_ptr: u32,
    name_len: u32,
    version_ptr: u32,
    version_len: u32,
    env_id: u64,
    process_id: u64,
) -> Result<u32, Trap> {
    let memory = get_memory(&mut caller)?;
    let buffer = memory
        .data(&caller)
        .get(name_ptr as usize..(name_ptr + name_len) as usize)
        .or_trap("lunatic::process::register")?;
    let name = std::str::from_utf8(buffer).or_trap("lunatic::process::register")?;
    let name = String::from(name);
    let buffer = memory
        .data(&caller)
        .get(version_ptr as usize..(version_ptr + version_len) as usize)
        .or_trap("lunatic::process::register")?;
    let version = std::str::from_utf8(buffer).or_trap("lunatic::process::register")?;
    let process = caller
        .data()
        .resources
        .processes
        .get(process_id)
        .or_trap("lunatic::process::register")?
        .clone();
    let environment = caller
        .data()
        .resources
        .environments
        .get(env_id)
        .or_trap("lunatic::process::register")?;
    let registry = environment.registry();
    match registry.insert(name, version, process) {
        Ok(()) => Ok(0),
        Err(_) => Ok(1),
    }
}

//% lunatic::process::unregister(
//%     name_ptr: u32,
//%     name_len: u32,
//%     version_ptr: u32,
//%     version_len: u32,
//%     env_id: u64
//%  ) -> u32
//%
//% Returns:
//% * 0 if the process was removed
//% * 1 if version string is not a correct semver string
//% * 2 if no match exists
//%
//% Remove process from registry.
//%
//% Traps:
//% * If the environment ID doesn't exist.
//% * If **name_ptr + name_len** is outside the memory.
//% * If **version_ptr + version_len** is outside the memory.
fn unregister(
    mut caller: Caller<ProcessState>,
    name_ptr: u32,
    name_len: u32,
    version_ptr: u32,
    version_len: u32,
    env_id: u64,
) -> Result<u32, Trap> {
    let memory = get_memory(&mut caller)?;
    let buffer = memory
        .data(&caller)
        .get(name_ptr as usize..(name_ptr + name_len) as usize)
        .or_trap("lunatic::process::unregister")?;
    let name = std::str::from_utf8(buffer).or_trap("lunatic::process::unregister")?;
    let buffer = memory
        .data(&caller)
        .get(version_ptr as usize..(version_ptr + version_len) as usize)
        .or_trap("lunatic::process::unregister")?;
    let version = std::str::from_utf8(buffer).or_trap("lunatic::process::unregister")?;
    let environment = caller
        .data()
        .resources
        .environments
        .get(env_id)
        .or_trap("lunatic::process::unregister")?;
    let registry = environment.registry();
    match registry.remove(name, version) {
        Ok(result) => match result {
            Some(_) => Ok(0),
            None => Ok(2),
        },
        Err(_) => Ok(1),
    }
}

//% lunatic::process::lookup(
//%     name_ptr: u32,
//%     name_len: u32,
//%     query_ptr: u32,
//%     query_len: u32,
//%     id_u64_ptr: u32,
//%  ) -> u32
//%
//% Returns:
//% * 0 if the process was successfully returned
//% * 1 if version string is not a correct semver string
//% * 2 if no process was found
//%
//% Returns a process that was registered inside the environment that the caller belongs to.
//% The query can be be an exact version or follow semver query rules (e.g. "^1.1").
//%
//% Traps:
//% * If **name_ptr + name_len** is outside the memory.
//% * If **query_ptr + query_len** is outside the memory.
fn lookup(
    mut caller: Caller<ProcessState>,
    name_ptr: u32,
    name_len: u32,
    query_ptr: u32,
    query_len: u32,
    id_u64_ptr: u32,
) -> Result<u32, Trap> {
    let memory = get_memory(&mut caller)?;
    let buffer = memory
        .data(&caller)
        .get(name_ptr as usize..(name_ptr + name_len) as usize)
        .or_trap("lunatic::process::lookup")?;
    let name = std::str::from_utf8(buffer).or_trap("lunatic::process::lookup")?;
    let buffer = memory
        .data(&caller)
        .get(query_ptr as usize..(query_ptr + query_len) as usize)
        .or_trap("lunatic::process::lookup")?;
    let query = std::str::from_utf8(buffer).or_trap("lunatic::process::lookup")?;
    let registry = caller.data().module.environment().registry();
    let process = match registry.get(name, query) {
        Ok(proc) => proc,
        Err(_) => return Ok(1),
    };
    match process {
        Some(process) => {
            let process_id = caller.data_mut().resources.processes.add(process);
            memory
                .write(&mut caller, id_u64_ptr as usize, &process_id.to_le_bytes())
                .or_trap("lunatic::process::lookup")?;
            Ok(0)
        }
        None => Ok(2),
    }
}
