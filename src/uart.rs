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
    const HEADER_LEN: usize = 4;

    pub fn new() -> Self{
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
                self.rx_offset += 1;

                //We got 4 bytes that should be the command header UMX
                if self.rx_offset >= Self::HEADER_LEN {
                    //Check the magic numbers to make sure we're receiving valid packet
                    if self.rx_buf[0..3] == [85, 77, 88] {
                        self.reset();
                        self.state = UartState::ReceivingCommand;
                        //Update bytes to read with packet length
                        self.bytes_to_read = self.rx_buf[3] as usize;
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

    pub fn reset(&mut self){
        self.rx_buf.fill(0);
        self.rx_offset = 0;
        self.bytes_to_read = 0;
        self.state = UartState::AwaitingHeader;
    }

    pub fn get_command(&mut self) -> Option<[u8; RX_BUFFER_SIZE]> {
        if let UartState::CommandReceived = self.state{
            let copy = self.rx_buf.clone();
            self.reset();
            return Some(copy);
        }

        return None;
    }
}
