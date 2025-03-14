use crate::builder::CanBeAddedToModel;
use crate::{Eventhdlr, Model, ProblemCreated};

/// A builder for easily creating event handlers. It can be created using the `eventhdlr` function.
pub struct EventHdlrBuilder<E: Eventhdlr> {
    name: Option<String>,
    desc: Option<String>,
    eventhdlr: E,
}

impl<E: Eventhdlr> EventHdlrBuilder<E> {
    /// Creates a new `EventHdlrBuilder` with the given event handler.
    ///
    /// # Defaults
    /// - `name`: empty string
    /// - `desc`: empty string
    pub fn new(eventhdlr: E) -> Self {
        EventHdlrBuilder {
            name: None,
            desc: None,
            eventhdlr,
        }
    }

    /// Sets the name of the event handler.
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    /// Sets the description of the event handler.
    pub fn desc(mut self, desc: &str) -> Self {
        self.desc = Some(desc.to_string());
        self
    }
}

/// Creates a new default `EventHdlrBuilder` from an event handler.
/// This function allows you to write:
/// ```rust
/// use russcip::prelude::*;
/// use russcip::{Event, EventMask, Eventhdlr, ProblemCreated, SCIPEventhdlr, Solving};
///
/// struct MyEventHandler;
/// impl Eventhdlr for MyEventHandler {
///     fn get_type(&self) -> EventMask {
///         todo!()
///     }
///
///     fn execute(&mut self, model: Model<Solving>, eventhdlr: SCIPEventhdlr, event: Event) {
///         todo!()
///     }
///
/// }
/// let ev = eventhdlr(MyEventHandler {}).name("My Event Handler");
/// let mut model = Model::default();
/// model.add(eventhdlr(MyEventHandler {}));
/// ```
pub fn eventhdlr<E: Eventhdlr>(ev: E) -> EventHdlrBuilder<E> {
    EventHdlrBuilder::new(ev)
}

impl<E: Eventhdlr + 'static> CanBeAddedToModel<ProblemCreated> for EventHdlrBuilder<E> {
    type Return = ();

    fn add(self, model: &mut Model<ProblemCreated>) {
        let name = self.name.unwrap_or_else(|| "".into());
        let desc = self.desc.unwrap_or_else(|| "".into());
        let eventhdlr = Box::new(self.eventhdlr);
        model.include_eventhdlr(&name, &desc, eventhdlr);
    }
}
