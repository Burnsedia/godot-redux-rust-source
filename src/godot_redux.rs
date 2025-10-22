use godot::prelude::*;
use godot::classes::Object;
use godot::builtin::{Callable, Dictionary, GString, StringName};

#[derive(GodotClass)]
#[class(init, base = Object)]
pub struct GodotRedux {
    #[base]
    base: Base<Object>,

    #[init(val = Dictionary::new())]
    state: Dictionary,

    #[init(val = Callable::invalid())]
    reducer: Callable,

    #[init(val = Vec::new())]
    middleware: Vec<Callable>,

    #[init(val = Vec::new())]
    subscriptions: Vec<Callable>,
}

#[godot_api]
impl GodotRedux {
    /// Creates/overwrites the store state and reducer.
    #[func]
    fn set_state_and_reducer(
        &mut self,
        initial_state: Dictionary,
        reducer_instance: Gd<Object>,
        reducer_method: GString, // <-- take GString
    ) {
        self.state = initial_state;
        // pass owned StringName (by value)
        self.reducer = Callable::from_object_method(
            &reducer_instance,
            StringName::from(reducer_method),
        );
        self.middleware.clear();
        self.subscriptions.clear();
    }

    /// Returns the current state.
    #[func]
    fn state(&self) -> Dictionary {
        self.state.clone()
        // or: self.state.duplicate_shallow()
    }

    /// Dispatches an action to update the state.
    #[func]
    fn dispatch(&mut self, action: i64) {
        if self.middleware.is_empty() {
            self.dispatch_reducer(action);
        } else {
            self.dispatch_middleware(0, action);
        }
    }

    // --- internals ---

    fn dispatch_middleware(&mut self, index: usize, action: i64) {
        if index == self.middleware.len() {
            self.dispatch_reducer(action);
            return;
        }

        let args = &[self.state.to_variant(), action.to_variant()];
        let next_variant = self.middleware[index].call(args);
        let next_action = next_variant.try_to_relaxed::<i64>().ok();

        if let Some(x) = next_action {
            self.dispatch_middleware(index + 1, x);
        }
    }

    fn dispatch_reducer(&mut self, action: i64) {
        let args = &[self.state.to_variant(), action.to_variant()];
        let new_state_v = self.reducer.call(args);

        // FIX: supply a closure arg
        self.state = new_state_v
            .try_to::<Dictionary>()
            .unwrap_or_else(|_| Dictionary::new());

        self.dispatch_subscriptions();
    }

    fn dispatch_subscriptions(&self) {
        let args = &[self.state.to_variant()];
        for sub in &self.subscriptions {
            sub.call(args);
        }
    }

    /// Subscribes to state changes.
    #[func]
    fn subscribe(&mut self, subscriber_instance: Gd<Object>, subscriber_method: GString) {
        let callable = Callable::from_object_method(
            &subscriber_instance,
            StringName::from(subscriber_method), // pass by value
        );
        self.subscriptions.push(callable);
    }

    /// Adds a middleware.
    #[func]
    fn add_middleware(&mut self, middleware_instance: Gd<Object>, middleware_method: GString) {
        let callable = Callable::from_object_method(
            &middleware_instance,
            StringName::from(middleware_method), // pass by value
        );
        self.middleware.push(callable);
    }
}

