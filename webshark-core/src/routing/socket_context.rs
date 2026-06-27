//! Модуль, отвечающий за предоставление методов удобной работы с вебсокетом для конечного пользователя.
//!
use bytes::Bytes;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tracing::{error, info};
use tokio_tungstenite::tungstenite::{Error as WsError, Message};
use tokio_tungstenite::tungstenite::error::ProtocolError;

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
    fragment_buffer: Vec<u8>,
    fragment_opcode: u8,
}

impl WebSocketContext {
    pub fn new<T>(socket: T) -> Self
    where
        T: AsyncRead + AsyncWrite + Unpin + Send + 'static + Sync,
    {
        Self {
            socket: Box::new(socket),
            fragment_buffer: Vec::new(),
            fragment_opcode: 0
        }
    }

    pub async fn send(&mut self, data: impl Into<Bytes>) -> Result<(), WsError> {
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

    pub async fn recv(&mut self) -> Result<Message, WsError> {

        loop {
            let mut header = [0u8; 2];
            self.socket.read_exact(&mut header).await?;

            let is_fin = (header[0] & 0x80) != 0;
            let opcode = header[0] & 0x0F;

            let is_masked = (header[1] & 0x80) != 0;
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
            if is_masked {
                self.socket.read_exact(&mut musk).await?;
            } else {
                error!("Получен не замаскированный фрейм от клиента.");
                let _ = self.socket.shutdown().await;
                return Err(WsError::Protocol(ProtocolError::UnmaskedFrameFromClient))
            }

            let mut payload = vec![0u8; payload_len];
            self.socket.read_exact(&mut payload).await?;

            for i in 0..payload_len {
                payload[i] = payload[i] ^ musk[i % 4];
            }

            match opcode {
                OPCODE_TEXT | OPCODE_BINARY => {
                    info!("Получен фрейм текстовый от клиента.");

                    if !is_fin {
                        self.fragment_opcode = opcode;
                        self.fragment_buffer.extend_from_slice(&payload);
                        continue;
                    }

                    return if opcode == OPCODE_TEXT {
                        let text = String::from_utf8(payload).map_err(|err| WsError::Utf8(err.to_string()))?;
                        Ok(Message::Text(text.into()))
                    } else {
                        Ok(Message::Binary(payload.into()))
                    }
                }
                OPCODE_CLOSE => {
                    info!("Получен фрейм закрытия от клиента.");

                    let reply_len = payload_len.min(125);

                    let mut reply = Vec::with_capacity(2 + reply_len);
                    reply.push(0x88); // FIN = 1, Opcode = 8 (Close)

                    reply.push(reply_len as u8);

                    if payload_len >= 2 {
                        reply.extend_from_slice(&payload[0..2]);
                    }

                    if let Err(e) = self.socket.write_all(&reply).await {
                        error!("Не удалось отправить ответный фрейм закрытия: {}", e);
                    }

                    let _ = self.socket.shutdown().await;

                    return Err(WsError::ConnectionClosed)
                }
                OPCODE_PING => {
                    info!("Получен фрейм пинга от клиента.");

                    let reply_len = payload_len.min(125);

                    let mut reply = Vec::with_capacity(2 + reply_len);
                    reply.push(0x8A); // FIN = 1, Opcode = 10 (Pong)
                    reply.push(reply_len as u8);
                    reply.extend_from_slice(&payload[0..reply_len]);

                    if let Err(e) = self.socket.write_all(&reply).await {
                        error!("Не удалось отправить Pong фрейм: {}", e);
                    }
                }
                OPCODE_PONG => {
                    info!("Получен фрейм понга от клиента.");
                    continue;
                }
                0 => {
                    if self.fragment_buffer.is_empty() {
                        error!("Протокол нарушен: получен добавочный фрейм, но буфер пуст.");
                        let _ = self.socket.shutdown().await;
                        return Err(WsError::Protocol(ProtocolError::UnexpectedContinueFrame));
                    }

                    self.fragment_buffer.extend_from_slice(&payload);

                    if is_fin {
                        let final_payload = std::mem::take(&mut self.fragment_buffer);
                        let final_opcode = self.fragment_opcode;
                        self.fragment_opcode = 0;

                        return if final_opcode == OPCODE_TEXT {
                            let text = String::from_utf8(final_payload).map_err(|err| WsError::Utf8(err.to_string()))?;
                            Ok(Message::Text(text.into()))
                        } else {
                            Ok(Message::Binary(final_payload.into()))
                        }
                    }

                    continue;
                }
                _ => {
                    error!("Получен неизвестный или зарезервированный opcode: {}", opcode);
                    let _ = self.socket.shutdown().await;
                }
            }
        }
    }
}
