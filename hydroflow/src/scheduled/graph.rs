//! Module for the [`Hydroflow`] struct and helper items.

use std::any::Any;
use std::borrow::Cow;
use std::cell::Cell;
use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::io::Error;
use std::marker::PhantomData;

use bytes::Bytes;
use hydroflow_lang::diagnostic::{Diagnostic, SerdeSpan};
use hydroflow_lang::graph::HydroflowGraph;
use ref_cast::RefCast;
use tokio::sync::mpsc::{self, UnboundedReceiver};

use super::context::Context;
use super::handoff::handoff_list::PortList;
use super::handoff::{Handoff, HandoffMeta};
use super::port::{RecvCtx, RecvPort, SendCtx, SendPort, RECV, SEND};
use super::reactor::Reactor;
use super::state::StateHandle;
use super::subgraph::Subgraph;
use super::{HandoffId, SubgraphId};
use crate::util::unsync;

/// A Hydroflow graph. Owns, schedules, and runs the compiled subgraphs.
pub struct Hydroflow {
    pub(super) subgraphs: Vec<SubgraphData>,
    pub(super) context: Context,
    handoffs: Vec<HandoffData>,

    /// TODO(mingwei): separate scheduler into its own struct/trait?
    /// Index is stratum, value is FIFO queue for that stratum.
    stratum_queues: Vec<VecDeque<SubgraphId>>,
    /// Receive events, if second arg indicates if it is an external "important" event (true).
    event_queue_recv: UnboundedReceiver<(SubgraphId, bool)>,
    /// If the events have been received for this tick.
    events_received_tick: bool,

    /// See [`Self::meta_graph()`].
    meta_graph: Option<HydroflowGraph>,
    /// See [`Self::diagnostics()`].
    diagnostics: Option<Vec<Diagnostic<SerdeSpan>>>,

    /// TODO:
    /// For now we can just use Bytes and in the future use a better data structure.
    incoming_ports: HashMap<&'static str, unsync::mpsc::Sender<Result<Bytes, Error>>>,
    outgoing_ports: HashMap<&'static str, tokio::sync::mpsc::Receiver<Result<Bytes, Error>>>,
}
impl Default for Hydroflow {
    fn default() -> Self {
        let stratum_queues = vec![Default::default()]; // Always initialize stratum #0.
        let (event_queue_send, event_queue_recv) = mpsc::unbounded_channel();
        let context = Context {
            states: Vec::new(),

            event_queue_send,

            current_stratum: 0,
            current_tick: 0,

            subgraph_id: SubgraphId(0),

            task_join_handles: Vec::new(),
        };
        Self {
            subgraphs: Vec::new(),
            context,
            handoffs: Vec::new(),

            stratum_queues,
            event_queue_recv,
            events_received_tick: false,

            meta_graph: None,
            diagnostics: None,

            incoming_ports: Default::default(),
            outgoing_ports: Default::default(),
        }
    }
}
impl Hydroflow {
    /// Create a new empty Hydroflow graph.
    pub fn new() -> Self {
        Default::default()
    }

    /// Add an input port to the graph
    #[doc(hidden)]
    pub fn __add_in_port_sender(
        &mut self,
        name: &'static str,
        tx: unsync::mpsc::Sender<Result<Bytes, Error>>,
    ) {
        self.incoming_ports.insert(name, tx);
    }

    /// Add an output port to the graph
    #[doc(hidden)]
    pub fn __add_out_port_receiver(
        &mut self,
        name: &'static str,
        tx: tokio::sync::mpsc::Receiver<Result<Bytes, Error>>,
    ) {
        self.outgoing_ports.insert(name, tx);
    }

    /// Take all the input port senders.
    pub fn take_port_senders(
        &mut self,
    ) -> HashMap<&'static str, unsync::mpsc::Sender<Result<Bytes, Error>>> {
        std::mem::take(&mut self.incoming_ports)
    }

    /// Take all the output port receivers.
    pub fn take_port_receivers(
        &mut self,
    ) -> HashMap<&'static str, tokio::sync::mpsc::Receiver<Result<Bytes, Error>>> {
        std::mem::take(&mut self.outgoing_ports)
    }

    /// Assign the `HydroflowGraph` via JSON string.
    #[doc(hidden)]
    pub fn __assign_meta_graph(&mut self, meta_graph_json: &str) {
        let mut meta_graph: HydroflowGraph =
            serde_json::from_str(meta_graph_json).expect("Failed to deserialize graph.");

        let mut op_inst_diagnostics = Vec::new();
        meta_graph.insert_node_op_insts_all(&mut op_inst_diagnostics);
        assert!(op_inst_diagnostics.is_empty());

        assert!(self.meta_graph.replace(meta_graph).is_none());
    }
    /// Assign the diagnostics via JSON string.
    #[doc(hidden)]
    pub fn __assign_diagnostics(&mut self, diagnostics_json: &'static str) {
        let diagnostics: Vec<Diagnostic<SerdeSpan>> =
            serde_json::from_str(diagnostics_json).expect("Failed to deserialize diagnostics.");

        assert!(self.diagnostics.replace(diagnostics).is_none());
    }

    /// Return a handle to the meta `HydroflowGraph` if set. The `HydroflowGraph is a
    /// representation of all the operators, subgraphs, and handoffs in this `Hydroflow` instance.
    /// Will only be set if this graph was constructed using a surface syntax macro.
    pub fn meta_graph(&self) -> Option<&HydroflowGraph> {
        self.meta_graph.as_ref()
    }

    /// Returns any diagnostics generated by the surface syntax macro. Each diagnostic is a pair of
    /// (1) a `Diagnostic` with span info reset and (2) the `ToString` version of the diagnostic
    /// with original span info.
    /// Will only be set if this graph was constructed using a surface syntax macro.
    pub fn diagnostics(&self) -> Option<&[Diagnostic<SerdeSpan>]> {
        self.diagnostics.as_deref()
    }

    /// Returns a reactor for externally scheduling subgraphs, possibly from another thread.
    /// Reactor events are considered to be external events.
    pub fn reactor(&self) -> Reactor {
        Reactor::new(self.context.event_queue_send.clone())
    }

    /// Gets the current tick (local time) count.
    pub fn current_tick(&self) -> usize {
        self.context.current_tick
    }

    /// Gets the current stratum nubmer.
    pub fn current_stratum(&self) -> usize {
        self.context.current_stratum
    }

    /// Runs the dataflow until the next tick begins.
    /// Returns true if any work was done.
    pub fn run_tick(&mut self) -> bool {
        let mut work_done = false;
        // While work is immediately available *on the current tick*.
        while self.next_stratum(true) {
            work_done = true;
            // Do any work.
            self.run_stratum();
        }
        work_done
    }

    /// Runs the dataflow until no more (externally-triggered) work is immediately available.
    /// Runs at least one tick of dataflow, even if no external events have been received.
    /// If the dataflow contains loops this method may run forever.
    /// Returns true if any work was done.
    pub fn run_available(&mut self) -> bool {
        let mut work_done = false;
        // While work is immediately available.
        while self.next_stratum(false) {
            work_done = true;
            // Do any work.
            self.run_stratum();
        }
        work_done
    }

    /// Runs the dataflow until no more (externally-triggered) work is immediately available.
    /// Runs at least one tick of dataflow, even if no external events have been received.
    /// If the dataflow contains loops this method may run forever.
    /// Returns true if any work was done.
    /// Yields repeatedly to allow external events to happen.
    pub async fn run_available_async(&mut self) -> bool {
        let mut work_done = false;
        // While work is immediately available.
        while self.next_stratum(false) {
            work_done = true;
            // Do any work.
            self.run_stratum();

            // Yield between each stratum to receive more events.
            // TODO(mingwei): really only need to yield at start of ticks though.
            tokio::task::yield_now().await;
        }
        work_done
    }

    /// Runs the current stratum of the dataflow until no more local work is available (does not receive events).
    /// Returns true if any work was done.
    pub fn run_stratum(&mut self) -> bool {
        let mut work_done = false;

        while let Some(sg_id) = self.stratum_queues[self.context.current_stratum].pop_front() {
            work_done = true;
            {
                let sg_data = &mut self.subgraphs[sg_id.0];
                // This must be true for the subgraph to be enqueued.
                assert!(sg_data.is_scheduled.take());

                self.context.subgraph_id = sg_id;
                sg_data.subgraph.run(&mut self.context, &mut self.handoffs);
            }

            for &handoff_id in self.subgraphs[sg_id.0].succs.iter() {
                let handoff = &self.handoffs[handoff_id.0];
                if !handoff.handoff.is_bottom() {
                    for &succ_id in handoff.succs.iter() {
                        let succ_sg_data = &self.subgraphs[succ_id.0];
                        if succ_sg_data.is_scheduled.get() {
                            // Skip if task is already scheduled.
                            continue;
                        }
                        succ_sg_data.is_scheduled.set(true);
                        self.stratum_queues[succ_sg_data.stratum].push_back(succ_id);
                    }
                }
            }
        }
        work_done
    }

    /// Go to the next stratum which has work available, possibly the current stratum.
    /// Return true if more work is available, otherwise false if no work is immediately
    /// available on any strata.
    ///
    /// This will receive external events when at the start of a tick.
    ///
    /// If `current_tick_only` is set to `true`, will only return `true` if work is immediately
    /// available on the *current tick*.
    ///
    /// If this returns false then the graph will be at the start of a tick (at stratum 0, can
    /// receive more external events).
    pub fn next_stratum(&mut self, current_tick_only: bool) -> bool {
        if 0 == self.context.current_stratum && !self.events_received_tick {
            // Add any external jobs to ready queue.
            self.try_recv_events();
        }

        // The stratum we will stop searching at, i.e. made a full loop around.
        let mut end_stratum = self.context.current_stratum;

        loop {
            // If current stratum has work, return true.
            if !self.stratum_queues[self.context.current_stratum].is_empty() {
                return true;
            }

            // Increment stratum counter.
            self.context.current_stratum += 1;
            if self.context.current_stratum >= self.stratum_queues.len() {
                self.context.current_stratum = 0;
                self.context.current_tick += 1;
                if current_tick_only {
                    self.events_received_tick = false;
                    return false;
                } else {
                    let (_num_events, has_external) = self.try_recv_events();
                    if has_external {
                        // Do a full loop more to find where events have been added.
                        end_stratum = 0;
                        continue;
                    }
                }
            }

            // After incrementing, exit if we made a full loop around the strata.
            if end_stratum == self.context.current_stratum {
                // Note: if current stratum had work, the very first loop iteration would've
                // returned true. Therefore we can return false without checking.
                // Also means nothing was done so we can reset the stratum to zero and wait for
                // events.
                self.events_received_tick = false;
                self.context.current_stratum = 0;
                return false;
            }
        }
    }

    /// Runs the dataflow graph forever.
    ///
    /// TODO(mingwei): Currently blockes forever, no notion of "completion."
    pub fn run(&mut self) -> Option<!> {
        loop {
            self.run_tick();
        }
    }

    /// Runs the dataflow graph forever.
    ///
    /// TODO(mingwei): Currently blockes forever, no notion of "completion."
    pub async fn run_async(&mut self) -> Option<!> {
        loop {
            // Run any work which is immediately available.
            self.run_available_async().await;
            // When no work is available yield until more events occur.
            self.recv_events_async().await;
        }
    }

    /// Enqueues subgraphs triggered by events without blocking.
    ///
    /// Returns the number of subgraphs enqueued, and if any were external.
    pub fn try_recv_events(&mut self) -> (usize, bool) {
        self.events_received_tick = true;

        let mut events_has_external = false;
        let mut enqueued_count = 0;
        while let Ok((sg_id, is_external)) = self.event_queue_recv.try_recv() {
            let sg_data = &self.subgraphs[sg_id.0];
            events_has_external |= is_external;
            if !sg_data.is_scheduled.replace(true) {
                self.stratum_queues[sg_data.stratum].push_back(sg_id);
                enqueued_count += 1;
            }
        }
        (enqueued_count, events_has_external)
    }

    /// Enqueues subgraphs triggered by external events, blocking until at
    /// least one subgraph is scheduled **from an external event**.
    pub fn recv_events(&mut self) -> Option<usize> {
        self.events_received_tick = true;

        let mut count = 0;
        let mut external = false;
        while !external {
            let (sg_id, is_external) = self.event_queue_recv.blocking_recv()?;
            external |= is_external;
            let sg_data = &self.subgraphs[sg_id.0];
            if !sg_data.is_scheduled.replace(true) {
                self.stratum_queues[sg_data.stratum].push_back(sg_id);
                count += 1;
            }
        }
        debug_assert!(external);
        // Enqueue any other immediate events.
        let (extra_count, _extra_external) = self.try_recv_events();
        Some(count + extra_count)
    }

    /// Enqueues subgraphs triggered by external events asynchronously, waiting until at least one
    /// subgraph is scheduled **from an external event**. Returns the number of subgraphs enqueued,
    /// which may be zero if an external event scheduled an already-scheduled subgraph.
    ///
    /// Returns `None` if the event queue is closed, but that should not happen normally.
    pub async fn recv_events_async(&mut self) -> Option<usize> {
        self.events_received_tick = true;

        let mut count = 0;
        let mut external = false;
        while !external {
            let (sg_id, is_external) = self.event_queue_recv.recv().await?;
            external |= is_external;
            let sg_data = &self.subgraphs[sg_id.0];
            if !sg_data.is_scheduled.replace(true) {
                self.stratum_queues[sg_data.stratum].push_back(sg_id);
                count += 1;
            }
        }
        debug_assert!(external);
        // Enqueue any other immediate events.
        let (extra_count, _extra_external) = self.try_recv_events();
        Some(count + extra_count)
    }

    /// Adds a new compiled subgraph with the specified inputs and outputs in stratum 0.
    pub fn add_subgraph<Name, R, W, F>(
        &mut self,
        name: Name,
        recv_ports: R,
        send_ports: W,
        subgraph: F,
    ) -> SubgraphId
    where
        Name: Into<Cow<'static, str>>,
        R: 'static + PortList<RECV>,
        W: 'static + PortList<SEND>,
        F: 'static + for<'ctx> FnMut(&'ctx mut Context, R::Ctx<'ctx>, W::Ctx<'ctx>),
    {
        self.add_subgraph_stratified(name, 0, recv_ports, send_ports, subgraph)
    }

    /// Adds a new compiled subgraph with the specified inputs, outputs, and stratum number.
    ///
    /// TODO(mingwei): add example in doc.
    pub fn add_subgraph_stratified<Name, R, W, F>(
        &mut self,
        name: Name,
        stratum: usize,
        recv_ports: R,
        send_ports: W,
        mut subgraph: F,
    ) -> SubgraphId
    where
        Name: Into<Cow<'static, str>>,
        R: 'static + PortList<RECV>,
        W: 'static + PortList<SEND>,
        F: 'static + for<'ctx> FnMut(&'ctx mut Context, R::Ctx<'ctx>, W::Ctx<'ctx>),
    {
        let sg_id = SubgraphId(self.subgraphs.len());

        let (mut subgraph_preds, mut subgraph_succs) = Default::default();
        recv_ports.set_graph_meta(&mut *self.handoffs, None, Some(sg_id), &mut subgraph_preds);
        send_ports.set_graph_meta(&mut *self.handoffs, Some(sg_id), None, &mut subgraph_succs);

        let subgraph = move |context: &mut Context, handoffs: &mut Vec<HandoffData>| {
            let recv = recv_ports.make_ctx(&*handoffs);
            let send = send_ports.make_ctx(&*handoffs);
            (subgraph)(context, recv, send);
        };
        self.subgraphs.push(SubgraphData::new(
            name.into(),
            stratum,
            subgraph,
            subgraph_preds,
            subgraph_succs,
            true,
        ));
        self.init_stratum(stratum);
        self.stratum_queues[stratum].push_back(sg_id);

        sg_id
    }

    /// Adds a new compiled subgraph with a variable number of inputs and outputs of the same respective handoff types.
    pub fn add_subgraph_n_m<Name, R, W, F>(
        &mut self,
        name: Name,
        recv_ports: Vec<RecvPort<R>>,
        send_ports: Vec<SendPort<W>>,
        subgraph: F,
    ) -> SubgraphId
    where
        Name: Into<Cow<'static, str>>,
        R: 'static + Handoff,
        W: 'static + Handoff,
        F: 'static
            + for<'ctx> FnMut(&'ctx mut Context, &'ctx [&'ctx RecvCtx<R>], &'ctx [&'ctx SendCtx<W>]),
    {
        self.add_subgraph_stratified_n_m(name, 0, recv_ports, send_ports, subgraph)
    }

    /// Adds a new compiled subgraph with a variable number of inputs and outputs of the same respective handoff types.
    pub fn add_subgraph_stratified_n_m<Name, R, W, F>(
        &mut self,
        name: Name,
        stratum: usize,
        recv_ports: Vec<RecvPort<R>>,
        send_ports: Vec<SendPort<W>>,
        mut subgraph: F,
    ) -> SubgraphId
    where
        Name: Into<Cow<'static, str>>,
        R: 'static + Handoff,
        W: 'static + Handoff,
        F: 'static
            + for<'ctx> FnMut(&'ctx mut Context, &'ctx [&'ctx RecvCtx<R>], &'ctx [&'ctx SendCtx<W>]),
    {
        let sg_id = SubgraphId(self.subgraphs.len());

        let subgraph_preds = recv_ports.iter().map(|port| port.handoff_id).collect();
        let subgraph_succs = send_ports.iter().map(|port| port.handoff_id).collect();

        for recv_port in recv_ports.iter() {
            self.handoffs[recv_port.handoff_id.0].succs.push(sg_id);
        }
        for send_port in send_ports.iter() {
            self.handoffs[send_port.handoff_id.0].preds.push(sg_id);
        }

        let subgraph = move |context: &mut Context, handoffs: &mut Vec<HandoffData>| {
            let recvs: Vec<&RecvCtx<R>> = recv_ports
                .iter()
                .map(|hid| hid.handoff_id)
                .map(|hid| handoffs.get(hid.0).unwrap())
                .map(|h_data| {
                    h_data
                        .handoff
                        .any_ref()
                        .downcast_ref()
                        .expect("Attempted to cast handoff to wrong type.")
                })
                .map(RefCast::ref_cast)
                .collect();

            let sends: Vec<&SendCtx<W>> = send_ports
                .iter()
                .map(|hid| hid.handoff_id)
                .map(|hid| handoffs.get(hid.0).unwrap())
                .map(|h_data| {
                    h_data
                        .handoff
                        .any_ref()
                        .downcast_ref()
                        .expect("Attempted to cast handoff to wrong type.")
                })
                .map(RefCast::ref_cast)
                .collect();

            (subgraph)(context, &recvs, &sends)
        };
        self.subgraphs.push(SubgraphData::new(
            name.into(),
            stratum,
            subgraph,
            subgraph_preds,
            subgraph_succs,
            true,
        ));
        self.init_stratum(stratum);
        self.stratum_queues[stratum].push_back(sg_id);

        sg_id
    }

    /// Makes sure stratum STRATUM is initialized.
    fn init_stratum(&mut self, stratum: usize) {
        if self.stratum_queues.len() <= stratum {
            self.stratum_queues
                .resize_with(stratum + 1, Default::default);
        }
    }

    /// Creates a handoff edge and returns the corresponding send and receive ports.
    pub fn make_edge<Name, H>(&mut self, name: Name) -> (SendPort<H>, RecvPort<H>)
    where
        Name: Into<Cow<'static, str>>,
        H: 'static + Handoff,
    {
        let handoff_id = HandoffId(self.handoffs.len());

        // Create and insert handoff.
        let handoff = H::default();
        self.handoffs.push(HandoffData::new(name.into(), handoff));

        // Make ports.
        let input_port = SendPort {
            handoff_id,
            _marker: PhantomData,
        };
        let output_port = RecvPort {
            handoff_id,
            _marker: PhantomData,
        };
        (input_port, output_port)
    }

    /// Adds referenceable state into the `Hydroflow` instance. Returns a state handle which can be
    /// used externally or by operators to access the state.
    ///
    /// This is part of the "state API".
    pub fn add_state<T>(&mut self, state: T) -> StateHandle<T>
    where
        T: Any,
    {
        self.context.add_state(state)
    }

    /// Gets a exclusive (mut) ref to the internal context, setting the subgraph ID.
    pub fn context_mut(&mut self, sg_id: SubgraphId) -> &mut Context {
        self.context.subgraph_id = sg_id;
        &mut self.context
    }
}

impl Hydroflow {
    /// Alias for [`Context::spawn_task`].
    pub fn spawn_task<Fut>(&mut self, future: Fut)
    where
        Fut: Future<Output = ()> + 'static,
    {
        self.context.spawn_task(future);
    }

    /// Alias for [`Context::abort_tasks`].
    pub fn abort_tasks(&mut self) {
        self.context.abort_tasks()
    }

    /// Alias for [`Context::join_tasks`].
    pub fn join_tasks(&mut self) -> impl '_ + Future {
        self.context.join_tasks()
    }
}

impl Drop for Hydroflow {
    fn drop(&mut self) {
        self.abort_tasks();
    }
}

/// A handoff and its input and output [SubgraphId]s.
///
/// Internal use: used to track the hydroflow graph structure.
///
/// TODO(mingwei): restructure `PortList` so this can be crate-private.
#[doc(hidden)]
pub struct HandoffData {
    /// A friendly name for diagnostics.
    #[allow(dead_code)] // TODO(mingwei): remove attr once used.
    pub(super) name: Cow<'static, str>,
    /// Crate-visible to crate for `handoff_list` internals.
    pub(super) handoff: Box<dyn HandoffMeta>,
    pub(super) preds: Vec<SubgraphId>,
    pub(super) succs: Vec<SubgraphId>,
}
impl std::fmt::Debug for HandoffData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.debug_struct("HandoffData")
            .field("preds", &self.preds)
            .field("succs", &self.succs)
            .finish_non_exhaustive()
    }
}
impl HandoffData {
    pub fn new(name: Cow<'static, str>, handoff: impl 'static + HandoffMeta) -> Self {
        let (preds, succs) = Default::default();
        Self {
            name,
            handoff: Box::new(handoff),
            preds,
            succs,
        }
    }
}

/// A subgraph along with its predecessor and successor [SubgraphId]s.
///
/// Used internally by the [Hydroflow] struct to represent the dataflow graph
/// structure and scheduled state.
pub(super) struct SubgraphData {
    /// A friendly name for diagnostics.
    #[allow(dead_code)] // TODO(mingwei): remove attr once used.
    pub(super) name: Cow<'static, str>,
    /// This subgraph's stratum number.
    pub(super) stratum: usize,
    /// The actual execution code of the subgraph.
    subgraph: Box<dyn Subgraph>,
    #[allow(dead_code)]
    preds: Vec<HandoffId>,
    succs: Vec<HandoffId>,

    /// If this subgraph is scheduled in [`Hydroflow::stratum_queues`].
    /// [`Cell`] allows modifying this field when iterating `Self::preds` or
    /// `Self::succs`, as all `SubgraphData` are owned by the same vec
    /// `Hydroflow::subgraphs`.
    is_scheduled: Cell<bool>,
}
impl SubgraphData {
    pub fn new(
        name: Cow<'static, str>,
        stratum: usize,
        subgraph: impl 'static + Subgraph,
        preds: Vec<HandoffId>,
        succs: Vec<HandoffId>,
        is_scheduled: bool,
    ) -> Self {
        Self {
            name,
            stratum,
            subgraph: Box::new(subgraph),
            preds,
            succs,
            is_scheduled: Cell::new(is_scheduled),
        }
    }
}

/// Internal struct containing a pointer to [`Hydroflow`]-owned state.
pub(crate) struct StateData {
    pub state: Box<dyn Any>,
}
