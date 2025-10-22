mod godot_redux;
use godot::prelude::*;

fn init(handle: InitHandle) {
    handle.add_class::<godot_redux::GodotRedux>();
}

godot_init!(init);
