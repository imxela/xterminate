use crate::input::KeyCode;
use crate::input::KeyState;

#[derive(Debug)]
pub struct Keybind {
    keys: Vec<KeyCode>
}

impl Keybind {
    /// Creates a new [Keybind] with no set keys.
    pub fn empty() -> Self {
        Self { 
            keys: Vec::new()
        }
    }

    /// Creates a new [Keybind] with the specified keys.
    pub fn new(keys: Vec<KeyCode>) -> Self {
        Self { 
            keys
        }
    }

    /// Adds a new key to the [Keybind].
    /// 
    /// # Returns
    /// 
    /// Returns `&self` to allow for mathod chaining.
    pub fn add(&mut self, key: KeyCode) -> &Self {
        self.keys.push(key);

        self
    }

    /// Returns true if all the keys ([KeyCode]s) in this 
    /// [Keybind] are currently pressed.
    pub fn triggered(&self, state: &mut KeyState) -> bool {
        for key in &self.keys {
            if state.released(*key) {
                return false
            }
        }

        // All keys are pressed!
        true
    }
}