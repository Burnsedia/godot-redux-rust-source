use godot::prelude::*;
use godot::classes::Object;
use godot::builtin::{Callable, Dictionary, StringName}; // <-- no GString

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
    #[func]
    fn set_state_and_reducer(
        &mut self,
        initial_state: Dictionary,
        reducer_instance: Gd<Object>,
        reducer_method: StringName,         // <-- take StringName by value
    ) {
        self.state = initial_state;
        self.reducer = Callable::from_object_method(&reducer_instance, reducer_method);
        self.middleware.clear();
        self.subscriptions.clear();
    }

    #[func]
    fn state(&self) -> Dictionary {
        self.state.clone()
    }

    #[func]
    fn dispatch(&mut self, action: i64) {
        if self.middleware.is_empty() {
            self.dispatch_reducer(action);
        } else {
            self.dispatch_middleware(0, action);
        }
    }

    fn dispatch_middleware(&mut self, index: usize, action: i64) {
        if index == self.middleware.len() {
            self.dispatch_reducer(action);
            return;
        }
        let args = &[self.state.to_variant(), action.to_variant()];
        if let Some(x) = self.middleware[index].call(args).try_to_relaxed::<i64>().ok() {
            self.dispatch_middleware(index + 1, x);
        }
    }

    fn dispatch_reducer(&mut self, action: i64) {
        let args = &[self.state.to_variant(), action.to_variant()];
        let new_state_v = self.reducer.call(args);
        self.state = new_state_v.try_to::<Dictionary>().unwrap_or_else(|_| Dictionary::new());
        self.dispatch_subscriptions();
    }

    fn dispatch_subscriptions(&self) {
        let args = &[self.state.to_variant()];
        for sub in &self.subscriptions {
            sub.call(args);
        }
    }

    #[func]
    fn subscribe(&mut self, subscriber_instance: Gd<Object>, subscriber_method: StringName) { // <-- StringName
        let callable = Callable::from_object_method(&subscriber_instance, subscriber_method);
        self.subscriptions.push(callable);
    }

    #[func]
    fn add_middleware(&mut self, middleware_instance: Gd<Object>, middleware_method: StringName) { // <-- StringName
        let callable = Callable::from_object_method(&middleware_instance, middleware_method);
        self.middleware.push(callable);
    }
}

