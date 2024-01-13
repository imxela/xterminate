use crate::input::KeyCode;
use crate::input::KeyState;

#[derive(Debug, Clone)]
pub struct Keybind {
    keys: Vec<KeyCode>,
}

impl Keybind {
    /// Creates a new [`Keybind`] with no set keys.
    #[must_use]
    pub fn empty() -> Self {
        Self { keys: Vec::new() }
    }

    /// Creates a new [`Keybind`] with the specified keys.
    #[must_use]
    pub fn new(keys: Vec<KeyCode>) -> Self {
        Self { keys }
    }

    /// Adds a new key to the [`Keybind`].
    ///
    /// # Returns
    ///
    /// Returns `&self` to allow for mathod chaining.
    pub fn add(&mut self, key: KeyCode) -> &Self {
        self.keys.push(key);

        self
    }

    /// Returns true if all the keys [`KeyCode`]s in this
    /// [`Keybind`] are currently pressed.
    pub fn triggered(&self, state: &mut KeyState) -> bool {
        for key in &self.keys {
            if state.released(*key) {
                return false;
            }
        }

        // All keys are pressed!
        true
    }

    /// Returns the keybinds as a Vec of [`KeyCode`]s.
    #[must_use]
    pub fn keycodes(self) -> Vec<KeyCode> {
        self.keys
    }
}

impl std::fmt::Display for Keybind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut formatted = String::new();

        let mut is_first: bool = true;
        for keycode in &self.keys {
            if !is_first {
                formatted.push_str(" + ");
            }

            formatted.push_str(keycode.to_string().as_str());

            is_first = false;
        }

        write!(f, "{formatted}")
    }
}
