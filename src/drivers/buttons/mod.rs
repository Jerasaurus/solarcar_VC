use defmt::info;
use embassy_stm32::gpio::{Flex, Pull, Pin};
use embassy_stm32::Peri;

/// Button identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ButtonId {
    CruiseDown,
    CruiseUp,
    Reverse,
    PushToTalk,
    Horn,
    PowerSave,
    Rearview,
    LeftTurn,  // Toggle mode
    RightTurn, // Toggle mode
    Lock,      // Toggle mode
}

/// Button event types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ButtonEvent {
    Pressed(ButtonId),
    Released(ButtonId),
    Toggled(ButtonId, bool), // (button, new_state)
}

/// Button type - regular or toggle
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ButtonType {
    Regular,
    Toggle,
}

/// Button configuration with pin
pub struct Button {
    pub id: ButtonId,
    pub name: &'static str,
    pub button_type: ButtonType,
    pub pin: Flex<'static>,
}

impl Button {
    /// Create a regular button
    pub fn regular<P: Pin>(id: ButtonId, name: &'static str, pin: Peri<'static, P>) -> Self {
        let mut flex = Flex::new(pin);
        flex.set_as_input(Pull::Up);
        Self {
            id,
            name,
            button_type: ButtonType::Regular,
            pin: flex,
        }
    }

    /// Create a toggle button
    pub fn toggle<P: Pin>(id: ButtonId, name: &'static str, pin: Peri<'static, P>) -> Self {
        let mut flex = Flex::new(pin);
        flex.set_as_input(Pull::Up);
        Self {
            id,
            name,
            button_type: ButtonType::Toggle,
            pin: flex,
        }
    }
}

/// All button inputs
pub struct ButtonInputs {
    buttons: heapless::Vec<Button, 16>,  // Support up to 16 buttons
}

impl ButtonInputs {
    /// Initialize button inputs from a list of buttons
    /// This allows you to define buttons and their pins together in main.rs
    pub fn new(buttons: impl IntoIterator<Item = Button>) -> Self {
        info!("Initializing button inputs");

        let mut button_vec = heapless::Vec::new();
        for button in buttons {
            let _ = button_vec.push(button);
        }

        Self { buttons: button_vec }
    }

    /// Read the state of a specific button (returns true if pressed)
    pub fn is_pressed(&self, id: ButtonId) -> bool {
        if let Some(button) = self.buttons.iter().find(|b| b.id == id) {
            // Inverted because we use pull-up resistors (active low)
            return !button.pin.is_high();
        }
        false
    }
}

/// Button state tracker with debouncing
pub struct ButtonState {
    // Current debounced states (true = pressed)
    states: heapless::Vec<bool, 16>,
    // Raw states for debouncing
    raw_states: heapless::Vec<bool, 16>,
    // Debounce counters
    debounce_counters: heapless::Vec<u8, 16>,
    // Toggle states for toggle-mode buttons
    toggle_states: heapless::FnvIndexMap<ButtonId, bool, 8>,
}

impl ButtonState {
    pub fn new(inputs: &ButtonInputs) -> Self {
        let mut states = heapless::Vec::new();
        let mut raw_states = heapless::Vec::new();
        let mut debounce_counters = heapless::Vec::new();

        // Initialize vectors with the right number of buttons
        for _button in &inputs.buttons {
            let _ = states.push(false);
            let _ = raw_states.push(false);
            let _ = debounce_counters.push(0);
        }

        // Initialize toggle states for toggle buttons
        let mut toggle_states = heapless::FnvIndexMap::new();
        for button in &inputs.buttons {
            if button.button_type == ButtonType::Toggle {
                let _ = toggle_states.insert(button.id, false);
            }
        }

        Self {
            states,
            raw_states,
            debounce_counters,
            toggle_states,
        }
    }

    /// Update button states with debouncing
    /// Returns a vector of button events that occurred
    pub fn update(&mut self, inputs: &ButtonInputs) -> heapless::Vec<ButtonEvent, 10> {
        let mut events = heapless::Vec::new();

        // Read and debounce each button
        for (i, button) in inputs.buttons.iter().enumerate() {
            // Read current raw state
            let raw = inputs.is_pressed(button.id);

            if raw != self.raw_states[i] {
                // State changed, reset debounce counter
                self.debounce_counters[i] = 0;
                self.raw_states[i] = raw;
            } else if self.debounce_counters[i] < 5 {
                // Same state, increment counter
                self.debounce_counters[i] += 1;

                // Check if debounced
                if self.debounce_counters[i] == 5 && self.states[i] != raw {
                    // State has been stable for 5 cycles, update
                    self.states[i] = raw;

                    // Generate events based on button type
                    match button.button_type {
                        ButtonType::Toggle => {
                            if self.states[i] {
                                // Button pressed, toggle the state
                                if let Some(toggle_state) = self.toggle_states.get_mut(&button.id) {
                                    *toggle_state = !*toggle_state;
                                    let _ = events.push(ButtonEvent::Toggled(button.id, *toggle_state));
                                }
                            }
                        }
                        ButtonType::Regular => {
                            if self.states[i] {
                                let _ = events.push(ButtonEvent::Pressed(button.id));
                            } else {
                                let _ = events.push(ButtonEvent::Released(button.id));
                            }
                        }
                    }
                }
            }
        }

        events
    }

    /// Get the current toggle state of a toggle button
    pub fn get_toggle_state(&self, id: ButtonId) -> Option<bool> {
        self.toggle_states.get(&id).copied()
    }
}