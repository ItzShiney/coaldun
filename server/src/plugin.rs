use libloader::libloading::{
    Error as LibError,
    os::windows::{Library, Symbol as LibSymbol},
};
use state::State;
use std::ffi::c_void;

type PluginData = *mut c_void;

pub struct Plugin {
    pub plugin: PluginData,
    pub uninit: LibSymbol<fn(PluginData)>,
    pub handle_event: LibSymbol<fn(PluginData, &mut State)>,
}

impl Plugin {
    pub fn new(library: Library, state: &mut State) -> Result<Self, LibError> {
        unsafe {
            let uninit = library.get(b"uninit")?;
            let init = library.get::<fn(&mut State) -> _>(b"init")?;
            let handle_event = library.get(b"handle_event")?;

            Ok(Self {
                plugin: init(state),
                handle_event,
                uninit,
            })
        }
    }
}

impl Plugin {
    #[expect(dead_code)]
    pub fn handle_event(&mut self, state: &mut State) {
        (self.handle_event)(self.plugin, state);
    }
}

impl Drop for Plugin {
    fn drop(&mut self) {
        (self.uninit)(self.plugin);
    }
}
