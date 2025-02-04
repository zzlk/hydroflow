use std::any::Any;
use std::borrow::Cow;
use std::cell::Cell;
use std::collections::VecDeque;
use std::future::Future;
use std::marker::PhantomData;
use std::num::NonZeroUsize;

use hydroflow_lang::graph::serde_graph::SerdeGraph;
use ref_cast::RefCast;
use tokio::runtime::TryCurrentError;
use tokio::sync::mpsc::{self, UnboundedReceiver};

use super::context::Context;
use super::handoff::handoff_list::PortList;
use super::handoff::{Handoff, HandoffMeta};
use super::port::{RecvCtx, RecvPort, SendCtx, SendPort, RECV, SEND};
use super::reactor::Reactor;
use super::state::StateHandle;
use super::subgraph::Subgraph;
use super::{HandoffId, SubgraphId};

/// A Hydroflow graph. Owns, schedules, and runs the compiled subgraphs.
pub struct Hydroflow {
    pub(super) subgraphs: Vec<SubgraphData>,
    pub(super) context: Context,

    /// TODO(mingwei): separate scheduler into its own struct/trait?
    /// Index is stratum, value is FIFO queue for that stratum.
    stratum_queues: Vec<VecDeque<SubgraphId>>,
    event_queue_recv: UnboundedReceiver<SubgraphId>,

    serde_graph: Option<SerdeGraph>,
}
impl Default for Hydroflow {
    fn default() -> Self {
        let (subgraphs, handoffs, states, task_join_handles) = Default::default();
        let stratum_queues = vec![Default::default()]; // Always initialize stratum #0.
        let (event_queue_send, event_queue_recv) = mpsc::unbounded_channel();
        let context = Context {
            handoffs,
            states,

            event_queue_send,

            current_stratum: 0,
            current_tick: 0,

            subgraph_id: SubgraphId(0),

            task_join_handles,
        };
        Self {
            subgraphs,
            context,

            stratum_queues,

            event_queue_recv,

            serde_graph: None,
        }
    }
}
impl Hydroflow {
    /// Create a new empty Hydroflow graph.
    pub fn new() -> Self {
        Default::default()
    }

    /// Create a new empty Hydroflow graph with the given serde_graph JSON string.
    pub fn new_with_graph(serde_graph: &'static str) -> Self {
        let mut graph = Self::new();
        graph.serde_graph = serde_json::from_str(serde_graph)
            .map_err(|e| {
                // TODO: use .inspect_err() when stable.
                eprintln!("Failed to deserialize serde_graph {}", e);
                e
            })
            .ok();

        graph
    }

    pub fn serde_graph(&self) -> Option<&SerdeGraph> {
        self.serde_graph.as_ref()
    }

    /// Returns a reactor for externally scheduling subgraphs, possibly from another thread.
    pub fn reactor(&self) -> Reactor {
        Reactor::new(self.context.event_queue_send.clone())
    }

    // Gets the current tick (local time) count.
    pub fn current_tick(&self) -> usize {
        self.context.current_tick
    }

    // Gets the current stratum nubmer.
    pub fn current_stratum(&self) -> usize {
        self.context.current_stratum
    }

    /// Runs the dataflow until the next tick begins.
    pub fn run_tick(&mut self) {
        let tick = self.current_tick();
        while self.next_stratum() && tick == self.current_tick() {
            self.run_stratum();
        }
    }

    /// Runs the dataflow until no more work is immediately available.
    /// If the dataflow contains loops this method may run forever.
    pub fn run_available(&mut self) {
        // While work is immediately available.
        while self.next_stratum() {
            // And do any work (this also receives events).
            self.run_stratum();
        }
    }

    /// Runs the current stratum of the dataflow until no more work is immediately available.
    pub fn run_stratum(&mut self) {
        // Add any external jobs to ready queue.
        self.try_recv_events();

        while let Some(sg_id) = self.stratum_queues[self.context.current_stratum].pop_front() {
            {
                let sg_data = &mut self.subgraphs[sg_id.0];
                // This must be true for the subgraph to be enqueued.
                assert!(sg_data.is_scheduled.take());

                self.context.subgraph_id = sg_id;
                sg_data.subgraph.run(&mut self.context);
            }

            for &handoff_id in self.subgraphs[sg_id.0].succs.iter() {
                let handoff = &self.context.handoffs[handoff_id.0];
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

            self.try_recv_events();
        }
    }

    /// Go to the next stratum which has work available, possibly the current stratum.
    /// Return true if more work is available, otherwise false if no work is immediately available on any strata.
    pub fn next_stratum(&mut self) -> bool {
        self.try_recv_events();

        let old_stratum = self.context.current_stratum;
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
            }
            // After incrementing, exit if we made a full loop around the strata.
            if old_stratum == self.context.current_stratum {
                // Note: if current stratum had work, the very first loop iteration would've
                // returned true. Therefore we can return false without checking.
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
            self.recv_events()?;
        }
    }

    /// Runs the dataflow graph forever.
    ///
    /// TODO(mingwei): Currently blockes forever, no notion of "completion."
    pub async fn run_async(&mut self) -> Option<!> {
        loop {
            // Run any work which is immediately available.
            self.run_available();
            // Only when there is absolutely no work available in any stratum.
            // Do we yield to wait for more events.
            self.recv_events_async().await;
        }
    }

    /// Enqueues subgraphs triggered by external events without blocking.
    ///
    /// Returns the number of subgraphs enqueued.
    pub fn try_recv_events(&mut self) -> usize {
        let mut enqueued_count = 0;
        while let Ok(sg_id) = self.event_queue_recv.try_recv() {
            let sg_data = &self.subgraphs[sg_id.0];
            if !sg_data.is_scheduled.replace(true) {
                self.stratum_queues[sg_data.stratum].push_back(sg_id);
                enqueued_count += 1;
            }
        }
        enqueued_count
    }

    /// Enqueues subgraphs triggered by external events, blocking until at
    /// least one subgraph is scheduled.
    pub fn recv_events(&mut self) -> Option<NonZeroUsize> {
        loop {
            let sg_id = self.event_queue_recv.blocking_recv()?;
            let sg_data = &self.subgraphs[sg_id.0];
            if !sg_data.is_scheduled.replace(true) {
                self.stratum_queues[sg_data.stratum].push_back(sg_id);

                // Enqueue any other immediate events.
                return Some(NonZeroUsize::new(self.try_recv_events() + 1).unwrap());
            }
        }
    }

    /// Enqueues subgraphs triggered by external events asynchronously, waiting
    /// until at least one subgraph is scheduled.
    pub async fn recv_events_async(&mut self) -> Option<NonZeroUsize> {
        loop {
            let sg_id = self.event_queue_recv.recv().await?;
            let sg_data = &self.subgraphs[sg_id.0];
            if !sg_data.is_scheduled.replace(true) {
                self.stratum_queues[sg_data.stratum].push_back(sg_id);

                // Enqueue any other immediate events.
                return Some(NonZeroUsize::new(self.try_recv_events() + 1).unwrap());
            }
        }
    }

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
        F: 'static + for<'ctx> FnMut(&'ctx Context, R::Ctx<'ctx>, W::Ctx<'ctx>),
    {
        self.add_subgraph_stratified(name, 0, recv_ports, send_ports, subgraph)
    }

    /// Adds a new compiled subgraph with the specified inputs and outputs.
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
        F: 'static + for<'ctx> FnMut(&'ctx Context, R::Ctx<'ctx>, W::Ctx<'ctx>),
    {
        let sg_id = SubgraphId(self.subgraphs.len());

        let (mut subgraph_preds, mut subgraph_succs) = Default::default();
        recv_ports.set_graph_meta(
            &mut *self.context.handoffs,
            None,
            Some(sg_id),
            &mut subgraph_preds,
        );
        send_ports.set_graph_meta(
            &mut *self.context.handoffs,
            Some(sg_id),
            None,
            &mut subgraph_succs,
        );

        let subgraph = move |context: &mut Context| {
            let recv = recv_ports.make_ctx(&*context.handoffs);
            let send = send_ports.make_ctx(&*context.handoffs);
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
            + for<'ctx> FnMut(&'ctx Context, &'ctx [&'ctx RecvCtx<R>], &'ctx [&'ctx SendCtx<W>]),
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
            + for<'ctx> FnMut(&'ctx Context, &'ctx [&'ctx RecvCtx<R>], &'ctx [&'ctx SendCtx<W>]),
    {
        let sg_id = SubgraphId(self.subgraphs.len());

        let subgraph_preds = recv_ports.iter().map(|port| port.handoff_id).collect();
        let subgraph_succs = send_ports.iter().map(|port| port.handoff_id).collect();

        for recv_port in recv_ports.iter() {
            self.context.handoffs[recv_port.handoff_id.0]
                .succs
                .push(sg_id);
        }
        for send_port in send_ports.iter() {
            self.context.handoffs[send_port.handoff_id.0]
                .preds
                .push(sg_id);
        }

        let subgraph = move |context: &mut Context| {
            let recvs: Vec<&RecvCtx<R>> = recv_ports
                .iter()
                .map(|hid| hid.handoff_id)
                .map(|hid| context.handoffs.get(hid.0).unwrap())
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
                .map(|hid| context.handoffs.get(hid.0).unwrap())
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
        let handoff_id = HandoffId(self.context.handoffs.len());

        // Create and insert handoff.
        let handoff = H::default();
        self.context
            .handoffs
            .push(HandoffData::new(name.into(), handoff));

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
    pub fn spawn_task<Fut>(&mut self, future: Fut) -> Result<(), TryCurrentError>
    where
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.context.spawn_task(future)
    }

    pub fn abort_tasks(&mut self) {
        self.context.abort_tasks()
    }

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
