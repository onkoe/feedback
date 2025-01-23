//! Stuff related to sending things to the Rover.

// we need a type to store data about *where* to send info.

use std::net::{AddrParseError, IpAddr};

use pyo3::{exceptions::PyException, prelude::*};
use tokio::net::UdpSocket;

use crate::{
    error::{SendError, SendException},
    Arm, Led, Wheels,
};

/// An indicator of whether the request succeeded.
///
/// For more info on the error, see the wrapped [`reqwest::Error`].
///
/// WARNING: the microcontrollers will immediately change the existing values
/// to match what's sent over the network. In practice, this can result in some
/// weird behavior with the Rover! Be careful when sending these values.
type SendResult = Result<(), crate::error::SendError>;

/// Controls the Rover.
#[pyclass]
pub struct RoverController {
    /// A socket to speak with the microcontroller that moves the Rover.
    socket: UdpSocket,
}

impl RoverController {
    /// Creates a new [`SendToRover`] with the given IP address and port.
    ///
    /// ## Example
    ///
    /// ```
    /// use feedback::prelude::*;
    /// use core::net::{IpAddr, Ipv4Addr};
    ///
    /// // replace these with the actual ip/port of the microcontroller!
    /// let (ip, port): (Ipv4Addr, u16) = (Ipv4Addr::new(192, 168, 1, 101), 1001);
    ///
    /// // this creates the controller. use the various `ctrl.send` methods to
    /// // use it properly!
    /// let ctrl: RoverController = RoverController::new(IpAddr::V4(ip), port);
    /// ```
    #[tracing::instrument]
    pub async fn new(ip: IpAddr, port: u16) -> Result<Self, std::io::Error> {
        let socket = UdpSocket::bind(format!("{ip}:{port}"))
            .await
            .inspect_err(|e| tracing::warn!("Failed to connect to the given socket! err: {e}"))?;

        // socket was created successfully if we're still running!
        //
        // so... return a `Self`!
        Ok(Self { socket })
    }

    /// Attempts to send the given wheel speeds.
    #[tracing::instrument(skip(self))]
    pub async fn send_wheels(&self, wheels: &Wheels) -> SendResult {
        let mut message: [u8; 9] = [0x0; 9];

        // start with subsystem byte.
        // wheels is `0x01`, so...
        message[0] = Wheels::SUBSYSTEM_BYTE;

        // add the wheels part byte
        message[1] = Wheels::PART_BYTE;

        // each speed can be added directly...
        message[2] = wheels.wheel0;
        message[3] = wheels.wheel1;
        message[4] = wheels.wheel2;
        message[5] = wheels.wheel3;
        message[6] = wheels.wheel4;
        message[7] = wheels.wheel5;

        // add the checksum. no clue if electrical actually uses it lol
        message[8] = wheels.checksum;

        // check the message validity
        crate::parse::parse(message.as_slice()).inspect_err(|e| {
            tracing::error!("Constructed message for the wheels was invalid! err: {e}")
        })?;
        tracing::debug!("Sending wheels message over UDP... {message:?}");

        // finally, we can send the message over UDP!
        //
        // FIXME: we should NOT be using UDP, as this can cause major errors in
        //        pathing!
        self.socket
            .send(&message)
            .await
            .inspect_err(|e| tracing::error!("Failed to send wheel speeds! err: {e}"))
            .map(|_bytes_sent| ())
            .map_err(SendError::SocketError)
    }

    /// Attempts to send the given light color.
    #[tracing::instrument(skip(self))]
    pub async fn send_led(&self, lights: &Led) -> SendResult {
        let mut message: [u8; 5] = [0x0; 5];

        // first, the subsystem byte
        message[0] = Led::SUBSYSTEM_BYTE;

        // then the part byte (lights)
        message[1] = Led::PART_BYTE;

        // now the three values - r, g, b!
        message[2] = lights.red;
        message[3] = lights.green;
        message[4] = lights.blue;

        // check the message validity
        crate::parse::parse(message.as_slice()).inspect_err(|e| {
            tracing::error!("Constructed message for the lights was invalid! err: {e}")
        })?;
        tracing::debug!("Sending lights message over UDP... {message:?}");

        // send the message
        self.socket
            .send(&message)
            .await
            .inspect_err(|e| tracing::error!("Failed to send light color! err: {e}"))
            .map(|_bytes_sent| ())
            .map_err(SendError::SocketError)
    }

    /// Attempts to send... all that arm stuff.
    #[tracing::instrument(skip(self))]
    pub async fn send_arm(&self, arm: &Arm) -> SendResult {
        let mut message: [u8; 8] = [0x0; 8];

        // subsystem
        // NOTE: there is no part byte for the arm!
        message[0] = Arm::SUBSYSTEM_BYTE;

        // now... add each field in order.
        message[1] = arm.bicep;
        message[2] = arm.forearm;
        message[3] = arm.base;
        message[4] = arm.wrist_pitch;
        message[5] = arm.wrist_roll;
        message[6] = arm.claw;

        // and the checksum! again, no clue if electrical uses this.
        //
        // just assuming it's correct.
        message[7] = arm.checksum;

        // check the message validity
        crate::parse::parse(message.as_slice()).inspect_err(|e| {
            tracing::error!("Constructed message for the arm was invalid! err: {e}")
        })?;
        tracing::debug!("Sending arm message over UDP... {message:?}");

        // send the message
        self.socket
            .send(&message)
            .await
            .inspect_err(|e| tracing::error!("Failed to send arm controls! err: {e}"))
            .map(|_bytes_sent| ())
            .map_err(SendError::SocketError)
    }

    // TODO: some helper fns for moving the rover (i.e. turning/etc.) might be
    // helpful in the future.
}

pyo3::create_exception!(error, IpParseException, PyException);

pyo3::create_exception!(error, SocketConnectionException, PyException);

#[pymethods]
impl RoverController {
    /// Creates a new [`RoverController`].
    #[new]
    pub fn py_new(ip: String, port: u16) -> PyResult<Self> {
        let addr = ip
            .parse()
            .inspect_err(|e| {
                tracing::warn!("Failed to parse the given `ip` as an IP address! err: {e}")
            })
            .map_err(|e: AddrParseError| IpParseException::new_err(e.to_string()))?;

        futures_lite::future::block_on(Self::new(addr, port))
            .map_err(|e| SocketConnectionException::new_err(e.to_string()))
    }

    /// Attempts to send the given wheel speeds.
    pub fn py_send_wheels(&self, wheels: Wheels) -> PyResult<()> {
        futures_lite::future::block_on(self.send_wheels(&wheels))
            .map_err(|e| SendException::new_err(e.to_string()))
    }

    /// Attempts to send the given light color.
    pub fn py_send_led(&self, wheels: Wheels) -> PyResult<()> {
        futures_lite::future::block_on(self.send_wheels(&wheels))
            .map_err(|e| SendException::new_err(e.to_string()))
    }

    /// Attempts to send... all that arm stuff.
    pub fn py_send_arm(&self, wheels: Wheels) -> PyResult<()> {
        futures_lite::future::block_on(self.send_wheels(&wheels))
            .map_err(|e| SendException::new_err(e.to_string()))
    }
}
