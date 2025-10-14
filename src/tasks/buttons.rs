use defmt::*;
use embassy_time::Timer;
use crate::drivers::buttons::{ButtonInputs, ButtonState, ButtonEvent, ButtonId};

#[embassy_executor::task]
pub async fn button_task(inputs: ButtonInputs) {
    info!("Button task started!");
    log::info!("USB Logger: Button monitoring task started");

    let mut button_state = ButtonState::new();

    // Main button polling loop
    loop {
        // Poll buttons every 10ms for responsive debouncing
        let events = button_state.update(&inputs);

        // Process any button events
        for event in events {
            match event {
                ButtonEvent::Pressed(button) => {
                    let button_name = button_name(button);
                    info!("Button {} pressed", button_name);
                    log::info!("BUTTON PRESSED: {}", button_name);
                }
                ButtonEvent::Released(button) => {
                    let button_name = button_name(button);
                    info!("Button {} released", button_name);
                    log::info!("BUTTON RELEASED: {}", button_name);
                }
                ButtonEvent::Toggled(button, state) => {
                    let button_name = button_name(button);
                    let state_text = if state { "ON" } else { "OFF" };
                    info!("Toggle button {} is now {}", button_name, state_text);
                    log::info!("TOGGLE: {} is now {}", button_name, state_text);
                }
            }
        }

        // Wait before next poll
        Timer::after_millis(10).await;
    }
}

fn button_name(button: ButtonId) -> &'static str {
    match button {
        ButtonId::CruiseDown => "Cruise Down",
        ButtonId::CruiseUp => "Cruise Up",
        ButtonId::Reverse => "Reverse",
        ButtonId::PushToTalk => "Push-to-Talk",
        ButtonId::Horn => "Horn",
        ButtonId::PowerSave => "Power Save",
        ButtonId::Rearview => "Rearview",
        ButtonId::LeftTurn => "Left Turn",
        ButtonId::RightTurn => "Right Turn",
        ButtonId::Lock => "Lock",
    }
}