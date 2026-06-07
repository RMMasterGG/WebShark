use bytes::Bytes;
use std::io::Error;
use sha1::digest::typenum::op;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tracing::{error, info};

const FIN_BIT: u8 = 0x80; // 1000 0000 (Флаг завершения сообщения)

// Коды операций (Opcodes) по спецификации RFC 6455
const OPCODE_TEXT: u8 = 0x01; // Текстовый фрейм
const OPCODE_BINARY: u8 = 0x02; // Бинарный фрейм
const OPCODE_CLOSE: u8 = 0x08; // Фрейм закрытия соединения
const OPCODE_PING: u8 = 0x09; // Пинг
const OPCODE_PONG: u8 = 0x0A; // Понг

pub trait AsyncDuplex: AsyncRead + AsyncWrite + Unpin + Send {}

impl<T> AsyncDuplex for T where T: AsyncRead + AsyncWrite + Unpin + Send {}

pub struct WebSocketContext {
    socket: Box<dyn AsyncDuplex + Send + Sync + 'static>,
}

impl WebSocketContext {
    pub fn new<T>(socket: T) -> Self
    where
        T: AsyncRead + AsyncWrite + Unpin + Send + 'static + Sync,
    {
        Self {
            socket: Box::new(socket),
        }
    }

    pub async fn send(&mut self, data: impl Into<Bytes>) -> Result<(), Error> {
        let payload = data.into();
        let payload_len = payload.len();

        let mut frame = Vec::with_capacity(10 + payload_len);

        frame.push(FIN_BIT | OPCODE_TEXT);

        if payload_len <= 125 {
            frame.push(payload.len() as u8);
        } else if payload_len <= 65535 {
            frame.push(126);
            frame.extend_from_slice(&(payload_len as u16).to_be_bytes())
        } else {
            frame.push(127);
            frame.extend_from_slice(&(payload_len as u64).to_be_bytes());
        }

        frame.extend_from_slice(&payload);

        self.socket.write_all(&frame).await?;
        self.socket.flush().await?;
        Ok(())
    }

    pub async fn recv(&mut self) -> Result<Bytes, Error> {

        let mut header = [0u8; 2];
        self.socket.read_exact(&mut header).await?;

        let opcode = header[0] & 0x0F;

        let base_len = header[1] & 0x7F;

        let payload_len: usize = match base_len {
            0..=125 => base_len as usize,
            126 => {
                let mut extended_len = [0u8; 2];
                self.socket.read_exact(&mut extended_len).await?;
                u16::from_be_bytes(extended_len) as usize
            }
            127 => {
                let mut extended_len = [0u8; 8];
                self.socket.read_exact(&mut extended_len).await?;
                u64::from_be_bytes(extended_len) as usize
            }
            _ => unreachable!(),
        };

        let mut musk = [0u8; 4];
        self.socket.read_exact(&mut musk).await?;

        let mut payload = vec![0u8; payload_len];
        self.socket.read_exact(&mut payload).await?;

        for i in 0..payload_len {
            payload[i] = payload[i] ^ musk[i % 4];
        }

        if opcode == 8 {
            info!("Получен фрейм закрытия от клиента.");

            let mut reply = Vec::with_capacity(2 + payload_len);
            reply.push(0x88); // FIN = 1, Opcode = 8

            let mut reply = Vec::with_capacity(2 + payload_len);
            reply.push(0x88); // FIN = 1, Opcode = 8

            if payload_len <= 125 {
                reply.push(payload_len as u8);
            } else {

                reply.push(126);
                reply.extend_from_slice(&(payload_len as u16).to_be_bytes());
            }

            if payload_len >= 2 {
                reply.extend_from_slice(&payload[0..2]);
            }

            if let Err(e) = self.socket.write_all(&reply).await {
                error!("Не удалось отправить ответный фрейм закрытия: {}", e);
            }

            let _ = self.socket.shutdown().await;
        }

        // match opcode {
        //     OPCODE_TEXT => {
        //
        //     }
        //     OPCODE_BINARY => {
        //
        //     }
        //     OPCODE_CLOSE => {
        //
        //     }
        //     OPCODE_PING => {
        //
        //     }
        //     OPCODE_PONG => {
        //
        //     }
        //     _ => {
        //
        //     }
        // }

        Ok(payload.into())
    }
}
