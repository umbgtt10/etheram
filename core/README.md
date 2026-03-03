# Barechain Core

Core provides the **foundational building blocks** for distributed consensus systems.

## Architectural Evolution

Barechain's architecture has evolved through several insights:

### Phase 1: Six Dimensions (Original)
Initially conceived as six orthogonal concerns:
1. Protocol - Consensus algorithm
2. Storage - Persistent state
3. Cache - Volatile state
4. Transport - P2P communication
5. ExternalInterface - Client communication
6. Timer - Time-based events

### Phase 2: Input/Output Split (Logical Separation)
Each dimension naturally splits by data flow direction:
- Timer → TimerInput + TimerOutput
- ExternalInterface → ExternalInterfaceInput + ExternalInterfaceOutput
- Transport → TransportInput + TransportOutput
- Storage (read queries vs write mutations)
- Cache (read queries vs write updates)

**Result: 10 directional traits instead of 6 combined ones**

### Phase 3: Infrastructure vs Decision (Current Understanding)

Two orthogonal concerns emerged:

**Infrastructure (Fixed Structure):**
- Input polling (3 input dimensions aggregated)
- State management (owns storage + cache)
- Output execution (3 output dimensions aggregated)
- Every node needs these, structure is constant

**Decision (Variable Strategy):**
- Context building (prepare decision inputs)
- Protocol logic (consensus algorithm)
- Action partitioning (classify outputs)
- Number and type of components varies by scheduling approach

**Key Insight:** Infrastructure is about HOW we execute, Decision is about WHAT we execute. These are mutually orthogonal - you can swap scheduling strategies without touching dimensions, and swap dimensions without touching decision logic.

## What Belongs in Core?

Core contains **behavioral building blocks**, not prescriptive blueprints:

### Individual Dimension Traits ✅
```rust
pub trait TimerInput {
    type Event;
    fn poll(&self) -> Option<Self::Event>;
}

pub trait TimerOutput {
    type Event;
    type Duration;
    fn schedule(&self, event: Self::Event, delay: Self::Duration);
}
```

These are simple, composable interfaces that define behavior without prescribing structure.

### Consensus Protocol Trait ✅
```rust
pub trait ConsensusProtocol {
    type Message;
    type MessageSource;
    type Action;
    type Context;
    type ActionCollection: Collection<Item = Self::Action>;

    fn handle_message(&self, source: &Self::MessageSource,
                     message: &Self::Message,
                     ctx: &Self::Context) -> Self::ActionCollection;
}
```

Defines the decision interface without dictating implementation.

### Minimal Node Trait ✅
```rust
pub trait Node {
    type Id: Copy;
    fn step(&mut self) -> bool;  // Returns true if work was done
    fn id(&self) -> Self::Id;
}
```

For multi-chain orchestration - defines behavior (execute steps), not structure (how components are organized).

### Prescriptive Structure Traits ❌
The old 6-dimension Node trait with all associated types was removed because:
- Too prescriptive (dictates HOW, not just WHAT)
- Only one implementation used it (TinyChain)
- Prevented architectural evolution
- Mixed concerns (building blocks vs blueprints)

## Core Execution Model

All nodes share the same execution primitive: **`step()`**

```rust
fn step(&mut self) -> bool {
    // Infrastructure: Poll inputs
    if let Some((source, message)) = self.input.poll() {
        // Decision: Build context
        let context = self.context_builder.build(&self.state, &source, &message);

        // Decision: Apply logic
        let actions = self.brain.handle_message(&source, &message, &context);

        // Decision: Partition actions
        let (mutations, outputs) = self.partitioner.partition(actions);

        // Infrastructure: Apply mutations
        self.state.apply_mutations(&mutations);

        // Infrastructure: Execute outputs
        self.executor.execute_outputs(&outputs);

        return true;
    }
    false
}
```

**Properties:**
- **Non-blocking** - Returns immediately
- **Deterministic** - Same events → same state transitions
- **Testable** - Can be called step-by-step
- **Composable** - Foundation for all execution patterns

## Implementation Examples

### EtheramNode (Infrastructure/Decision Split)
```rust
pub struct EtheramNode<TiIn, EIn, TrIn, TiOut, EOut, TrOut> {
    // === Infrastructure: Orchestration and execution ===
    node_id: NodeId,
    input: InputSources<TiIn, EIn, TrIn>,
    state: EtheramState<Storage, Cache>,
    executor: EtheramExecutor<TiOut, EOut, TrOut>,

    // === Decision: Context → Logic → Partition ===
    context_builder: ContextBuilder,
    brain: EtheramProtocol,
    partitioner: ActionPartitioner,
}
```

Type parameters are for **testing flexibility** (swap in test doubles), not abstraction. The architecture is explicit in the code structure.

### TinyChain (Original 6-Dimension Pattern)
Still follows the six-dimension concept but with its own node structure. Uses dimension traits from core as building blocks.

## Current Status

**Architecture is evolving, not finished.** Key insights:
- ✅ Infrastructure vs Decision orthogonality is real and valuable
- ✅ Input/Output splits make logical sense
- ✅ Core should provide building blocks, not blueprints
- ⚠️ Storage/Cache could be split into Read/Write (not yet done)
- ⚠️ BFT implementation will reveal more patterns
- ⚠️ Scheduler abstraction decision deferred

## Design Philosophy

1. **Traits define behavior, not structure** - Tell WHAT, not HOW
2. **Orthogonality enables independent evolution** - Change one concern without affecting others
3. **Let implementations discover patterns** - Abstract after understanding, not before
4. **Keep core minimal** - Only truly universal building blocks belong here

## When to Use Core Traits

Use Core's dimension traits when you need:
- **Shared implementations** across multiple projects (e.g., TestTimer)
- **Pluggable dimensions** for testing or deployment flexibility
- **Type-enforced separation** of concerns
- **Zero-cost abstractions** with compile-time optimization

Don't use them as a rigid blueprint - compose them into whatever structure fits your needs.
