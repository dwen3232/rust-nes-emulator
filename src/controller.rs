use bitflags::bitflags;

bitflags! {
    // https://www.nesdev.org/wiki/Standard_controller
    #[derive(Debug, Clone, Copy)]
    pub struct ControllerState: u8 {
        const A        = 0b00000001;
        const B        = 0b00000010;
        const SELECT   = 0b00000100;
        const START    = 0b00001000;
        const UP       = 0b00010000;
        const DOWN     = 0b00100000;
        const LEFT     = 0b01000000;
        const RIGHT    = 0b10000000;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Controller {
    strobe: bool,
    cur_flag: u8,
    pub controller_state: ControllerState,
}

impl Default for Controller {
    fn default() -> Self {
        Self::new()
    }
}

impl Controller {
    pub fn new() -> Self {
        Controller {
            strobe: false,
            cur_flag: 1,
            controller_state: ControllerState::from_bits_retain(0),
        }
    }

    pub fn set_controller_state(&mut self, state: ControllerState) {
        self.controller_state = state;
    }

    pub fn read(&mut self) -> u8 {
        if self.cur_flag == 0 {
            return 1;
        }
        let cur_flag = ControllerState::from_bits_retain(self.cur_flag);
        let value = if self.controller_state.contains(cur_flag) {
            1
        } else {
            0
        };
        if !self.strobe {
            self.cur_flag <<= 1;
        }
        value
    }

    pub fn peek(&self) -> u8 {
        if self.cur_flag == 0 {
            return 1;
        }
        let cur_flag = ControllerState::from_bits_retain(self.cur_flag);
        if self.controller_state.contains(cur_flag) {
            1
        } else {
            0
        }
    }

    pub fn write(&mut self, data: u8) {
        self.cur_flag = 1;
        self.strobe = (data & 1) == 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_read_reset_at_end() {
        // not a real test
        let mut controller = Controller::new();
        controller.set_controller_state(ControllerState::from_bits_retain(0b1010_0101));
        // Buttons
        assert_eq!(1, controller.read());
        assert_eq!(0, controller.read());
        assert_eq!(1, controller.read());
        assert_eq!(0, controller.read());
        assert_eq!(0, controller.read());
        assert_eq!(1, controller.read());
        assert_eq!(0, controller.read());
        assert_eq!(1, controller.read());
        // Always 1
        for _ in 0..10 {
            assert_eq!(1, controller.read());
        }
        controller.write(1);
        controller.write(0);
        assert_eq!(1, controller.read());
        assert_eq!(0, controller.read());
        assert_eq!(1, controller.read());
        assert_eq!(0, controller.read());
        assert_eq!(0, controller.read());
        assert_eq!(1, controller.read());
        assert_eq!(0, controller.read());
        assert_eq!(1, controller.read());
    }

    #[test]
    pub fn test_read_strobe_on() {
        // not a real test
        let mut controller = Controller::new();
        controller.set_controller_state(ControllerState::from_bits_retain(0b0010_0100));
        // Buttons
        assert_eq!(0, controller.read());
        assert_eq!(0, controller.read());
        assert_eq!(1, controller.read());
        assert_eq!(0, controller.read());
        assert_eq!(0, controller.read());
        assert_eq!(1, controller.read());
        assert_eq!(0, controller.read());
        assert_eq!(0, controller.read());
        // Always 1
        for _ in 0..10 {
            assert_eq!(1, controller.read());
        }
        controller.write(1);
        for _ in 0..10 {
            assert_eq!(0, controller.read());
        }
    }

    #[test]
    pub fn test_read_reset_early() {
        // not a real test
        let mut controller = Controller::new();
        controller.set_controller_state(ControllerState::from_bits_retain(0b0010_0100));
        // Buttons
        assert_eq!(0, controller.read());
        assert_eq!(0, controller.read());
        assert_eq!(1, controller.read());
        assert_eq!(0, controller.read());
        assert_eq!(0, controller.read());
        controller.write(1);
        controller.write(0);
        // Always 1
        assert_eq!(0, controller.read());
        assert_eq!(0, controller.read());
        assert_eq!(1, controller.read());
        assert_eq!(0, controller.read());
        assert_eq!(0, controller.read());
    }

    #[test]
    fn test_strobe_mode() {
        let mut controller = Controller::new();
        controller.write(1);
        controller.controller_state.insert(ControllerState::A);
        for _x in 0..10 {
            assert_eq!(controller.read(), 1);
        }
    }

    #[test]
    fn test_strobe_mode_on_off() {
        let mut controller = Controller::new();

        controller.write(0);
        controller.controller_state.insert(ControllerState::RIGHT);
        controller.controller_state.insert(ControllerState::LEFT);
        controller.controller_state.insert(ControllerState::SELECT);
        controller.controller_state.insert(ControllerState::B);

        for _ in 0..=1 {
            assert_eq!(controller.read(), 0);
            assert_eq!(controller.read(), 1);
            assert_eq!(controller.read(), 1);
            assert_eq!(controller.read(), 0);
            assert_eq!(controller.read(), 0);
            assert_eq!(controller.read(), 0);
            assert_eq!(controller.read(), 1);
            assert_eq!(controller.read(), 1);

            for _x in 0..10 {
                assert_eq!(controller.read(), 1);
            }
            controller.write(1);
            controller.write(0);
        }
    }
}
