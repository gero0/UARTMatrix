const HEADER: [u8; 3] = [85, 77, 88];

pub enum UartState {
    AwaitingHeader,
    ReceivingCommand,
    CommandReceived,
}

pub struct UartController<const RX_BUFFER_SIZE: usize> {
    rx_buf: [u8; RX_BUFFER_SIZE],
    rx_offset: usize,
    bytes_to_read: usize,
    state: UartState,
}

impl<const RX_BUFFER_SIZE: usize> UartController<RX_BUFFER_SIZE> {
    const HEADER_LEN: usize = 5;

    pub fn new() -> Self {
        UartController {
            rx_buf: [0; RX_BUFFER_SIZE],
            rx_offset: 0,
            bytes_to_read: 0,
            state: UartState::AwaitingHeader,
        }
    }

    pub fn read_byte(&mut self, byte: u8) {
        match self.state {
            UartState::AwaitingHeader => {
                self.rx_buf[self.rx_offset] = byte;

                if self.rx_offset < 3 && byte != HEADER[self.rx_offset] {
                    self.reset();
                    return;
                }

                self.rx_offset += 1;

                //We got 5 bytes that should be the command header UMX
                if self.rx_offset >= Self::HEADER_LEN {
                    //Check the magic numbers to make sure we're receiving valid packet
                    if self.rx_buf[0..3] == [85, 77, 88] {
                        //Update bytes_to_read with packet length
                        let bytes_to_read = ( (self.rx_buf[3] as u16) << 8) | self.rx_buf[4] as u16;
                        self.reset();
                        self.bytes_to_read = bytes_to_read as usize;
                        self.state = UartState::ReceivingCommand;
                    } else {
                        self.reset();
                    }
                }
            }

            UartState::ReceivingCommand => {
                self.rx_buf[self.rx_offset] = byte;
                self.rx_offset += 1;

                if self.rx_offset >= self.bytes_to_read {
                    self.state = UartState::CommandReceived;
                }
            }

            _ => {}
        }
    }

    pub fn reset(&mut self) {
        self.rx_buf.fill(0);
        self.rx_offset = 0;
        self.bytes_to_read = 0;
        self.state = UartState::AwaitingHeader;
    }

    pub fn get_command(&mut self) -> Option<[u8; RX_BUFFER_SIZE]> {
        if let UartState::CommandReceived = self.state {
            let copy = self.rx_buf.clone();
            self.reset();
            return Some(copy);
        }

        return None;
    }
}
