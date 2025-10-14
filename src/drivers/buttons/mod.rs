use defmt::info;
use embassy_stm32::gpio::{Input, Pull};
use embassy_stm32::peripherals::{PD12, PE14, PE0, PE4, PD14, PE2, PE8, PE12, PE6, PE10};
use embassy_stm32::Peri;

/// Button identifiers
#[derive(Debug, Clone, Copy, PartialEq)]
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

/// All button inputs
pub struct ButtonInputs {
    pub cruise_down: Input<'static>,
    pub cruise_up: Input<'static>,
    pub reverse: Input<'static>,
    pub push_to_talk: Input<'static>,
    pub horn: Input<'static>,
    pub power_save: Input<'static>,
    pub rearview: Input<'static>,
    pub left_turn: Input<'static>,
    pub right_turn: Input<'static>,
    pub lock: Input<'static>,
}

impl ButtonInputs {
    /// Initialize all button inputs with pull-up resistors
    pub fn new(
        pd12: Peri<'static, PD12>,
        pe14: Peri<'static, PE14>,
        pe0: Peri<'static, PE0>,
        pe4: Peri<'static, PE4>,
        pd14: Peri<'static, PD14>,
        pe2: Peri<'static, PE2>,
        pe8: Peri<'static, PE8>,
        pe12: Peri<'static, PE12>,
        pe6: Peri<'static, PE6>,
        pe10: Peri<'static, PE10>,
    ) -> Self {
        info!("Initializing button inputs");

        Self {
            cruise_down: Input::new(pd12, Pull::Up),
            cruise_up: Input::new(pe14, Pull::Up),
            reverse: Input::new(pe0, Pull::Up),
            push_to_talk: Input::new(pe4, Pull::Up),
            horn: Input::new(pd14, Pull::Up),
            power_save: Input::new(pe2, Pull::Up),
            rearview: Input::new(pe8, Pull::Up),
            left_turn: Input::new(pe12, Pull::Up),
            right_turn: Input::new(pe6, Pull::Up),
            lock: Input::new(pe10, Pull::Up),
        }
    }
}

/// Button state tracker with debouncing
pub struct ButtonState {
    // Current debounced states (true = pressed, assuming active-low buttons)
    pub states: [bool; 10],
    // Raw states for debouncing
    raw_states: [bool; 10],
    // Debounce counters
    debounce_counters: [u8; 10],
    // Toggle states for toggle-mode buttons
    pub toggle_states: [bool; 3], // left_turn, right_turn, lock
}

impl ButtonState {
    pub fn new() -> Self {
        Self {
            states: [false; 10],
            raw_states: [false; 10],
            debounce_counters: [0; 10],
            toggle_states: [false; 3],
        }
    }

    /// Update button states with debouncing
    /// Returns a vector of button events that occurred
    pub fn update(&mut self, inputs: &ButtonInputs) -> heapless::Vec<ButtonEvent, 10> {
        let mut events = heapless::Vec::new();

        // Read current raw states (inverted because pull-up)
        let raw = [
            !inputs.cruise_down.is_high(),  // 0
            !inputs.cruise_up.is_high(),     // 1
            !inputs.reverse.is_high(),       // 2
            !inputs.push_to_talk.is_high(),  // 3
            !inputs.horn.is_high(),          // 4
            !inputs.power_save.is_high(),    // 5
            !inputs.rearview.is_high(),      // 6
            !inputs.left_turn.is_high(),     // 7
            !inputs.right_turn.is_high(),    // 8
            !inputs.lock.is_high(),          // 9
        ];

        // Debounce each button
        for i in 0..10 {
            if raw[i] != self.raw_states[i] {
                // State changed, reset debounce counter
                self.debounce_counters[i] = 0;
                self.raw_states[i] = raw[i];
            } else if self.debounce_counters[i] < 5 {
                // Same state, increment counter
                self.debounce_counters[i] += 1;

                // Check if debounced
                if self.debounce_counters[i] == 5 && self.states[i] != raw[i] {
                    // State has been stable for 5 cycles, update
                    self.states[i] = raw[i];

                    // Generate events
                    let button_id = match i {
                        0 => ButtonId::CruiseDown,
                        1 => ButtonId::CruiseUp,
                        2 => ButtonId::Reverse,
                        3 => ButtonId::PushToTalk,
                        4 => ButtonId::Horn,
                        5 => ButtonId::PowerSave,
                        6 => ButtonId::Rearview,
                        7 => ButtonId::LeftTurn,
                        8 => ButtonId::RightTurn,
                        9 => ButtonId::Lock,
                        _ => continue,
                    };

                    // Check if this is a toggle button
                    match button_id {
                        ButtonId::LeftTurn | ButtonId::RightTurn | ButtonId::Lock => {
                            if self.states[i] {
                                // Button pressed, toggle the state
                                let toggle_idx = match button_id {
                                    ButtonId::LeftTurn => 0,
                                    ButtonId::RightTurn => 1,
                                    ButtonId::Lock => 2,
                                    _ => continue,
                                };
                                self.toggle_states[toggle_idx] = !self.toggle_states[toggle_idx];
                                let _ = events.push(ButtonEvent::Toggled(button_id, self.toggle_states[toggle_idx]));
                            }
                        }
                        _ => {
                            // Regular button
                            if self.states[i] {
                                let _ = events.push(ButtonEvent::Pressed(button_id));
                            } else {
                                let _ = events.push(ButtonEvent::Released(button_id));
                            }
                        }
                    }
                }
            }
        }

        events
    }
}