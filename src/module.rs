/*! Wasm modules */

use anyhow::{anyhow, Result};
use async_std::channel::unbounded;
use async_std::task::JoinHandle;
use log::trace;
use uuid::Uuid;
use wasmtime::{Store, Val};

use std::sync::Arc;

use crate::{
    environment::UNIT_OF_COMPUTE_IN_INSTRUCTIONS,
    mailbox::MessageMailbox,
    process::{self, Process, Signal, WasmProcess},
    state::ProcessState,
    Environment,
};

/// A compiled WebAssembly module that can be used to spawn [`WasmProcesses`][0].
///
/// Modules are created from [`Environments`](crate::environment::Environment).
///
/// [0]: crate::WasmProcess
#[derive(Clone)]
pub struct Module {
    inner: Arc<InnerModule>,
}

struct InnerModule {
    data: Vec<u8>,
    env: Environment,
    wasmtime_module: wasmtime::Module,
}

impl Module {
    pub(crate) fn new(data: Vec<u8>, env: Environment, wasmtime_module: wasmtime::Module) -> Self {
        Self {
            inner: Arc::new(InnerModule {
                data,
                env,
                wasmtime_module,
            }),
        }
    }

    /// Spawns a new process from the module.
    ///
    /// A `Process` is created from a `Module`, an entry `function` and an array of arguments. The
    /// configuration of the environment will define some characteristics of the process, such as
    /// maximum memory, fuel and available host functions.
    ///
    /// After it's spawned the process will keep running in the background. A process can be killed
    /// by sending a `Signal::Kill` to it. If you would like to block until the process is finished
    /// you can `.await` on the returned `JoinHandle<()>`.
    pub async fn spawn(
        &self,
        function: &str,
        params: Vec<Val>,
        link: Option<(Option<i64>, WasmProcess)>,
    ) -> Result<(JoinHandle<()>, WasmProcess)> {
        // TODO: Switch to new_v1() for distributed Lunatic to assure uniqueness across nodes.
        let id = Uuid::new_v4();
        trace!("Spawning process: {}", id);
        let signal_mailbox = unbounded::<Signal>();
        let message_mailbox = MessageMailbox::default();
        let state = ProcessState::new(
            id,
            self.clone(),
            signal_mailbox.0.clone(),
            message_mailbox.clone(),
            self.environment().config(),
        )?;

        let mut store = Store::new(self.environment().engine(), state);
        store.limiter(|state| state);

        // Trap if out of fuel
        store.out_of_fuel_trap();
        // Define maximum fuel
        match self.environment().config().max_fuel() {
            Some(max_fuel) => {
                store.out_of_fuel_async_yield(max_fuel, UNIT_OF_COMPUTE_IN_INSTRUCTIONS)
            }
            // If no limit is specified use maximum
            None => store.out_of_fuel_async_yield(u64::MAX, UNIT_OF_COMPUTE_IN_INSTRUCTIONS),
        };

        let instance = self
            .environment()
            .linker()
            .instantiate_async(&mut store, self.wasmtime_module())
            .await?;
        let entry = instance
            .get_func(&mut store, function)
            .map_or(Err(anyhow!("Function '{}' not found", function)), |func| {
                Ok(func)
            })?;

        let fut = async move { entry.call_async(&mut store, &params).await };
        let child_process = process::new(fut, id, signal_mailbox.1, message_mailbox);
        let child_process_handle = WasmProcess::new(id, signal_mailbox.0.clone());

        // **Child link guarantees**:
        // The link signal is going to be put inside of the child's mailbox and is going to be
        // processed before any child code can run. This means that any failure inside the child
        // Wasm code will be correctly reported to the parent.
        //
        // We assume here that the code inside of `process::new()` will not fail during signal
        // handling.
        //
        // **Parent link guarantees**:
        // A `tokio::task::yield_now()` call is executed to allow the parent to link the child
        // before continuing any further execution. This should force the parent to process all
        // signals right away.
        //
        // The parent could have received a `kill` signal in its mailbox before this function was
        // called and this signal is going to be processed before the link is established (FIFO).
        // Only after the yield function we can guarantee that the child is going to be notified
        // if the parent fails. This is ok, as the actual spawning of the child happens after the
        // call, so the child wouldn't even exist if the parent failed before.
        if let Some((tag, process)) = link {
            // Send signal to itself to perform the linking
            process.send(Signal::Link(None, Arc::new(child_process_handle.clone())));
            // Suspend itself to process all new signals
            async_std::task::yield_now().await;
            // Send signal to child to link it
            signal_mailbox
                .0
                .try_send(Signal::Link(tag, Arc::new(process)))
                .expect("receiver must exist at this point");
        }

        // Spawn a background process
        trace!("Process size: {}", std::mem::size_of_val(&child_process));
        let join = async_std::task::spawn(child_process);
        Ok((join, child_process_handle))
    }

    pub fn environment(&self) -> &Environment {
        &self.inner.env
    }

    pub fn wasmtime_module(&self) -> &wasmtime::Module {
        &self.inner.wasmtime_module
    }

    /// The raw WebAssembly data that the Module was created from.
    pub fn data(&self) -> Vec<u8> {
        self.inner.data.clone()
    }
}
