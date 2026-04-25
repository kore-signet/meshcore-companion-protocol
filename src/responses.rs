use alloc::{
    borrow::{Cow, ToOwned},
    string::{String, ToString},
    vec::Vec,
};
use meshcore::{
    DecodeError, Path, PathLen,
    io::{SliceWriter, TinyReadExt},
    payloads::{AppdataFlags, TextType},
    repeater_protocol::Permissions,
};
use strum::FromRepr;

use crate::{CompanionSer, NullPaddedSlice, NullPaddedString};

#[derive(FromRepr)]
#[repr(u8)]
pub enum ResponseCodes {
    Ok = 0x00,
    Err = 0x01,
    ContactsStart = 0x02,
    Contact = 0x03,
    EndOfContacts = 0x04,
    SelfInfo = 0x05,
    MsgSent = 0x06,
    ContactMsgRecv = 0x07,
    ChannelMsgRecv = 0x08,
    ContactMsgRecvV3 = 0x10,
    ChannelMsgRecvV3 = 0x11,
    CurrTime = 0x09,
    NoMoreMessages = 0x0A,
    Battery = 0x0C,
    DeviceInfo = 0x0D,
    ChannelInfo = 0x12,
    Advertisement = 0x80,
    Ack = 0x82, // ?
    MessagesWaiting = 0x83,
    LogData = 0x88,
    LoginSuccess = 0x85,
    SignStart = 0x13,
    Signature = 0x14,
    ExportPrivateKey = 0xe,
    CustomVars = 0x15,
    TraceData = 0x89,
    ControlData = 0x8E,
    BinaryResponse = 0x8C,
}

#[derive(FromRepr, Copy, Clone, Debug)]
#[repr(u8)]
pub enum StatTypes {
    Core = 0,
    Radio = 1,
    Packets = 2,
}

#[derive(Debug, Clone)]
pub struct ChannelMsgRecv<'a> {
    pub snr: i8,
    pub reserved: [u8; 2],
    pub idx: u8,
    pub path_len: u8,
    pub text_ty: TextType,
    pub timestamp: u32,
    pub data: Cow<'a, [u8]>,
}

#[derive(Debug, Clone)]
pub struct ContactMsgRecv<'a> {
    pub snr: i8,
    pub reserved: [u8; 2],
    pub pk_prefix: [u8; 6],
    pub path_len: u8,
    pub text_ty: TextType,
    pub timestamp: u32,
    pub signature: Option<[u8; 4]>,
    pub data: Cow<'a, [u8]>,
}

#[derive(Clone)]
pub enum GetMessageRes<'a> {
    Contact(ContactMsgRecv<'a>),
    Channel(ChannelMsgRecv<'a>),
    NoMoreMessages,
}

impl<'a> CompanionSer for GetMessageRes<'a> {
    type Decoded<'data> = GetMessageRes<'data>;

    fn ser_size(&self) -> usize {
        match self {
            GetMessageRes::Contact(contact_msg_recv) => contact_msg_recv.ser_size(),
            GetMessageRes::Channel(channel_msg_recv) => channel_msg_recv.ser_size(),
            GetMessageRes::NoMoreMessages => 1,
        }
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        match self {
            GetMessageRes::Contact(contact_msg_recv) => contact_msg_recv.companion_serialize(out),
            GetMessageRes::Channel(channel_msg_recv) => channel_msg_recv.companion_serialize(out),
            GetMessageRes::NoMoreMessages => {
                let mut out = SliceWriter::new(out);
                out.write_u8(ResponseCodes::NoMoreMessages as u8);
                out.finish()
            }
        }
    }

    fn companion_deserialize<'d>(input: &'d [u8]) -> Result<Self::Decoded<'d>, DecodeError> {
        match ResponseCodes::from_repr(*input.first().ok_or(DecodeError::UnexpectedEof)?)
            .ok_or(DecodeError::InvalidBitPattern)?
        {
            ResponseCodes::ContactMsgRecvV3 => {
                ContactMsgRecv::companion_deserialize(input).map(GetMessageRes::Contact)
            }
            ResponseCodes::ChannelMsgRecvV3 => {
                ChannelMsgRecv::companion_deserialize(input).map(GetMessageRes::Channel)
            }
            ResponseCodes::NoMoreMessages => Ok(GetMessageRes::NoMoreMessages),
            _ => Err(DecodeError::InvalidBitPattern),
        }
    }
}

pub type CompanionProtoResult<T> = Result<T, Err>;
// Res(T),
// Err(Err),
// }

impl<T: CompanionSer> CompanionSer for CompanionProtoResult<T> {
    type Decoded<'data> = Result<T::Decoded<'data>, Err>;

    fn ser_size(&self) -> usize {
        match self {
            CompanionProtoResult::Ok(r) => r.ser_size(),
            CompanionProtoResult::Err(e) => e.ser_size(),
        }
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        match self {
            CompanionProtoResult::Ok(r) => r.companion_serialize(out),
            CompanionProtoResult::Err(e) => e.companion_serialize(out),
        }
    }

    fn companion_deserialize<'d>(input: &'d [u8]) -> Result<Self::Decoded<'d>, DecodeError> {
        if *input.first().ok_or(DecodeError::UnexpectedEof)? == ResponseCodes::Err as u8 {
            Ok(Err(Err::companion_deserialize(input)?))
        } else {
            Ok(Ok(T::companion_deserialize(input)?))
        }
    }
}

pub struct Ok {
    pub code: Option<u32>,
}

impl CompanionSer for Ok {
    type Decoded<'a> = Ok;

    fn ser_size(&self) -> usize {
        1 + if self.code.is_some() { 4 } else { 0 }
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);
        out.write_u8(ResponseCodes::Ok as u8);
        if let Some(code) = self.code {
            out.write_u32_le(code);
        }
        out.finish()
    }

    fn companion_deserialize<'d>(mut input: &'d [u8]) -> Result<Self::Decoded<'d>, DecodeError> {
        if input.read_u8()? != ResponseCodes::Ok as u8 {
            return Err(DecodeError::InvalidBitPattern);
        }

        if let Ok(rem) = input.read_chunk::<4>() {
            Ok(Ok {
                code: Some(u32::from_le_bytes(*rem)),
            })
        } else {
            Ok(Ok { code: None })
        }
    }
}

pub struct Err {
    pub code: Option<u8>,
}

impl CompanionSer for Err {
    type Decoded<'a> = Err;

    fn ser_size(&self) -> usize {
        1 + if self.code.is_some() { 1 } else { 0 }
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);
        out.write_u8(ResponseCodes::Ok as u8);
        if let Some(code) = self.code {
            out.write_u8(code);
        }
        out.finish()
    }

    fn companion_deserialize<'d>(mut input: &'d [u8]) -> Result<Self::Decoded<'d>, DecodeError> {
        if input.read_u8()? != ResponseCodes::Err as u8 {
            return Err(DecodeError::InvalidBitPattern);
        }

        if let Ok(rem) = input.read_chunk::<1>() {
            Ok(Err { code: Some(rem[0]) })
        } else {
            Ok(Err { code: None })
        }
    }
}

pub struct SelfInfo<'a> {
    pub advertisement_type: u8,
    pub tx_power: u8,
    pub max_tx_power: u8,
    pub public_key: [u8; 32],
    pub lat: u32,
    pub long: u32,
    pub multi_acks: u8,
    pub adv_loc_policy: u8,
    pub telemetry_mode: u8,
    pub manual_add_contacts: bool,
    pub radio_freq: u32,
    pub radio_bandwidth: u32,
    pub radio_sf: u8,
    pub radio_cr: u8,
    pub device_name: &'a str,
}

impl<'a> CompanionSer for SelfInfo<'a> {
    type Decoded<'data> = SelfInfo<'data>;

    fn ser_size(&self) -> usize {
        4 // packet ty + adv ty + tx power + max tx power
            + 32 // pk
            + 4 * 2 // lat,long
            + 4 // multi acks + adv loc policy + telemetry mode + manual add contacts
            + 4 * 2 // radio freq + band
            + 2 // spreading factor coding rate
            + self.device_name.len()
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);

        out.write_slice(&[
            ResponseCodes::SelfInfo as u8,
            self.advertisement_type,
            self.tx_power,
            self.max_tx_power,
        ]);

        out.write_slice(&self.public_key);
        out.write_u32_le(self.lat);
        out.write_u32_le(self.long);

        out.write_slice(&[
            self.multi_acks,
            self.adv_loc_policy,
            self.telemetry_mode,
            self.manual_add_contacts as u8,
        ]);

        out.write_u32_le(self.radio_freq);
        out.write_u32_le(self.radio_bandwidth);

        out.write_slice(&[self.radio_sf, self.radio_cr]);

        out.write_slice(self.device_name.as_bytes());

        out.finish()
    }

    fn companion_deserialize<'d>(mut input: &'d [u8]) -> Result<Self::Decoded<'d>, DecodeError> {
        if input.read_u8()? != ResponseCodes::SelfInfo as u8 {
            return Err(DecodeError::InvalidBitPattern);
        }

        Ok(SelfInfo {
            advertisement_type: input.read_u8()?,
            tx_power: input.read_u8()?,
            max_tx_power: input.read_u8()?,
            public_key: *input.read_chunk::<32>()?,
            lat: input.read_u32_le()?,
            long: input.read_u32_le()?,
            multi_acks: input.read_u8()?,
            adv_loc_policy: input.read_u8()?,
            telemetry_mode: input.read_u8()?,
            manual_add_contacts: input.read_u8()? > 0,
            radio_freq: input.read_u32_le()?,
            radio_bandwidth: input.read_u32_le()?,
            radio_sf: input.read_u8()?,
            radio_cr: input.read_u8()?,
            device_name: core::str::from_utf8(input)?,
        })
    }
}

pub struct DeviceInfo<'a> {
    pub fw_version: u8,
    pub max_contacts: u8,
    pub max_channels: u8,
    pub ble_pin: u32,
    pub firmware_build: NullPaddedSlice<'a, 12>,
    pub model: NullPaddedSlice<'a, 40>,
    pub version: NullPaddedSlice<'a, 20>,
    pub client_repeat_enabled: bool,
    pub path_hash_mode: u8,
}

impl<'a> CompanionSer for DeviceInfo<'a> {
    type Decoded<'data> = DeviceInfo<'data>;

    fn ser_size(&self) -> usize {
        4 // packet ty, firmware ver, max contacts, max channels
            + 4 // ble pin
            + 12 // firmware build
            + 40 // model
            + 20 // version
            + 1 // client repeat enabled
            + 1 // path hash mode
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);

        out.write_slice(&[
            ResponseCodes::DeviceInfo as u8,
            self.fw_version,
            self.max_contacts,
            self.max_channels,
        ]);

        out.write_u32_le(self.ble_pin);
        self.firmware_build.encode_to(&mut out);
        self.model.encode_to(&mut out);
        self.version.encode_to(&mut out);

        out.write_slice(&[self.client_repeat_enabled as u8, self.path_hash_mode]);

        out.finish()
    }

    fn companion_deserialize<'d>(mut input: &'d [u8]) -> Result<Self::Decoded<'d>, DecodeError> {
        if input.read_u8()? != ResponseCodes::DeviceInfo as u8 {
            return Err(DecodeError::InvalidBitPattern);
        }

        Ok(DeviceInfo {
            fw_version: input.read_u8()?,
            max_contacts: input.read_u8()?,
            max_channels: input.read_u8()?,
            ble_pin: input.read_u32_le()?,
            firmware_build: NullPaddedSlice::read(&mut input)?,
            model: NullPaddedSlice::read(&mut input)?,
            version: NullPaddedSlice::read(&mut input)?,
            client_repeat_enabled: input.read_u8()? > 0,
            path_hash_mode: input.read_u8()?,
        })
    }
}

pub struct ChannelInfo<'a> {
    pub idx: u8,
    pub name: NullPaddedString<'a, 32>,
    pub secret: [u8; 16],
}

impl<'a> CompanionSer for ChannelInfo<'a> {
    type Decoded<'data> = ChannelInfo<'data>;

    fn ser_size(&self) -> usize {
        2 // packet ty, channel idx
            + 32 // channel name
            + 16 // secret
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);
        out.write_slice(&[ResponseCodes::ChannelInfo as u8, self.idx]);
        self.name.encode_to(&mut out);
        out.write_slice(&self.secret);

        out.finish()
    }

    fn companion_deserialize<'d>(mut input: &'d [u8]) -> Result<Self::Decoded<'d>, DecodeError> {
        if input.read_u8()? != ResponseCodes::ChannelInfo as u8 {
            return Err(DecodeError::InvalidBitPattern);
        }

        Ok(ChannelInfo {
            idx: input.read_u8()?,
            name: NullPaddedString::read(&mut input)?,
            secret: *input.read_chunk::<16>()?,
        })
    }
}

pub struct Battery {
    pub battery_voltage: u16,
    pub used_storage: u32,
    pub total_storage: u32,
}

impl CompanionSer for Battery {
    type Decoded<'data> = Battery;

    fn ser_size(&self) -> usize {
        1 // packet ty
            + 2 // battery voltage
            + 4 // used storage
            + 4 // total storage
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);

        out.write_u8(ResponseCodes::Battery as u8);
        out.write_u16_le(self.battery_voltage);
        out.write_u32_le(self.used_storage);
        out.write_u32_le(self.total_storage);

        out.finish()
    }

    fn companion_deserialize<'d>(mut input: &'d [u8]) -> Result<Self::Decoded<'d>, DecodeError> {
        if input.read_u8()? != ResponseCodes::Battery as u8 {
            return Err(DecodeError::InvalidBitPattern);
        }

        Ok(Battery {
            battery_voltage: input.read_u16_le()?,
            used_storage: input.read_u32_le()?,
            total_storage: input.read_u32_le()?,
        })
    }
}

pub struct MsgSent {
    pub is_flood: bool,
    pub expected_ack: [u8; 4],
    pub suggested_timeout: u32,
}

impl CompanionSer for MsgSent {
    type Decoded<'data> = MsgSent;

    fn ser_size(&self) -> usize {
        1 // packet ty
            + 1 // routing type
            + 4 // expected ack
            + 4 // suggested timeout
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);

        out.write_u8(ResponseCodes::MsgSent as u8);
        out.write_u8(self.is_flood as u8);
        out.write_slice(&self.expected_ack);
        out.write_u32_le(self.suggested_timeout);

        out.finish()
    }

    fn companion_deserialize<'d>(mut input: &'d [u8]) -> Result<Self::Decoded<'d>, DecodeError> {
        if input.read_u8()? != ResponseCodes::MsgSent as u8 {
            return Err(DecodeError::InvalidBitPattern);
        }

        Ok(MsgSent {
            is_flood: input.read_u8()? > 0,
            expected_ack: *input.read_chunk()?,
            suggested_timeout: input.read_u32_le()?,
        })
    }
}

impl<'a> CompanionSer for ContactMsgRecv<'a> {
    type Decoded<'data> = ContactMsgRecv<'data>;

    fn ser_size(&self) -> usize {
        1 // packet ty
            + 1 // snr
            + 2 // reserved
            + 6 // pk prefix
            + 1 // path len
            + 1 // text type
            + 4 // timestamp
            + if self.signature.is_some() { 4 } else { 0 }
            + self.data.len()
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);

        out.write_u8(ResponseCodes::ContactMsgRecvV3 as u8);
        out.write_i8(self.snr);
        out.write_slice(&self.reserved);
        out.write_slice(&self.pk_prefix);
        out.write_u8(self.path_len);
        out.write_u8(self.text_ty as u8);
        out.write_u32_le(self.timestamp);

        if let Some(signature) = self.signature {
            out.write_slice(&signature);
        }

        out.write_slice(&self.data);

        out.finish()
    }

    fn companion_deserialize<'d>(mut input: &'d [u8]) -> Result<Self::Decoded<'d>, DecodeError> {
        if input.read_u8()? != ResponseCodes::ContactMsgRecvV3 as u8 {
            return Err(DecodeError::InvalidBitPattern);
        }
        let snr = input.read_i8()?;
        let reserved = *input.read_chunk()?;
        let pk_prefix = *input.read_chunk()?;
        let path_len = input.read_u8()?;
        let text_ty = match input.read_u8()? {
            0 => TextType::PlainText,
            1 => TextType::CliCommand,
            2 => TextType::SignedPlainText,
            _ => return Err(DecodeError::InvalidBitPattern),
        };

        Ok(ContactMsgRecv {
            snr,
            reserved,
            pk_prefix,
            path_len,
            text_ty,
            timestamp: input.read_u32_le()?,
            signature: if matches!(text_ty, TextType::SignedPlainText) {
                Some(*input.read_chunk()?)
            } else {
                None
            },
            data: Cow::Borrowed(input),
        })
    }
}

impl<'a> CompanionSer for ChannelMsgRecv<'a> {
    type Decoded<'data> = ChannelMsgRecv<'data>;

    fn ser_size(&self) -> usize {
        1 // packet ty
            + 1 // snr
            + 2 // reserved
            + 1 // channel idx
            + 1 // path len
            + 1 // text ty
            + 4 // timestamp
            + self.data.len()
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);

        out.write_u8(ResponseCodes::ChannelMsgRecvV3 as u8);
        out.write_i8(self.snr);
        out.write_slice(&self.reserved);
        out.write_slice(&[self.idx, self.path_len, self.text_ty as u8]);
        out.write_u32_le(self.timestamp);
        out.write_slice(&self.data);

        out.finish()
    }

    fn companion_deserialize<'d>(mut input: &'d [u8]) -> Result<Self::Decoded<'d>, DecodeError> {
        if input.read_u8()? != ResponseCodes::ChannelMsgRecvV3 as u8 {
            return Err(DecodeError::InvalidBitPattern);
        }

        Ok(ChannelMsgRecv {
            snr: input.read_i8()?,
            reserved: *input.read_chunk()?,
            idx: input.read_u8()?,
            path_len: input.read_u8()?,
            text_ty: match input.read_u8()? {
                0 => TextType::PlainText,
                1 => TextType::CliCommand,
                2 => TextType::SignedPlainText,
                _ => return Err(DecodeError::InvalidBitPattern),
            },
            timestamp: input.read_u32_le()?,
            data: Cow::Borrowed(input),
        })
    }
}

pub struct ContactStart {
    pub contacts: u32,
}

impl CompanionSer for ContactStart {
    type Decoded<'data> = ContactStart;

    fn ser_size(&self) -> usize {
        1 + 4 // packet_ty, len
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);
        out.write_u8(ResponseCodes::ContactsStart as u8);
        out.write_u32_le(self.contacts);
        out.finish()
    }

    fn companion_deserialize<'d>(mut input: &'d [u8]) -> Result<Self::Decoded<'d>, DecodeError> {
        if input.read_u8()? != ResponseCodes::ContactsStart as u8 {
            return Err(DecodeError::InvalidBitPattern);
        }

        Ok(ContactStart {
            contacts: input.read_u32_le()?,
        })
    }
}

pub struct ContactEnd {
    pub last_mod: u32,
}

impl CompanionSer for ContactEnd {
    type Decoded<'data> = ContactEnd;

    fn ser_size(&self) -> usize {
        1 + 4 // packet_ty, last_mod
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);

        out.write_u8(ResponseCodes::EndOfContacts as u8);
        out.write_u32_le(self.last_mod);

        out.finish()
    }

    fn companion_deserialize<'d>(mut input: &'d [u8]) -> Result<Self::Decoded<'d>, DecodeError> {
        if input.read_u8()? != ResponseCodes::EndOfContacts as u8 {
            return Err(DecodeError::InvalidBitPattern);
        }

        Ok(ContactEnd {
            last_mod: input.read_u32_le()?,
        })
    }
}

pub struct Ack {
    pub code: [u8; 4],
}

impl CompanionSer for Ack {
    type Decoded<'data> = Ack;

    fn ser_size(&self) -> usize {
        1 + 4 // packet_ty, last_mod
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);

        out.write_u8(ResponseCodes::Ack as u8);
        out.write_slice(&self.code);

        out.finish()
    }

    fn companion_deserialize<'d>(mut input: &'d [u8]) -> Result<Self::Decoded<'d>, DecodeError> {
        if input.read_u8()? != ResponseCodes::Ack as u8 {
            return Err(DecodeError::InvalidBitPattern);
        }

        Ok(Ack {
            code: *input.read_chunk()?,
        })
    }
}

pub struct RfLogData<'a> {
    pub snr: i8,
    pub rssi: i8,
    pub data: &'a [u8],
}

impl<'a> CompanionSer for RfLogData<'a> {
    type Decoded<'data> = RfLogData<'data>;

    fn ser_size(&self) -> usize {
        3 + self.data.len()
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);
        out.write_u8(ResponseCodes::LogData as u8);
        out.write_i8(self.snr);
        out.write_i8(self.rssi);
        out.write_slice(self.data);
        out.finish()
    }

    fn companion_deserialize<'d>(mut input: &'d [u8]) -> Result<Self::Decoded<'d>, DecodeError> {
        if input.read_u8()? != ResponseCodes::LogData as u8 {
            return Err(DecodeError::InvalidBitPattern);
        }

        Ok(RfLogData {
            snr: input.read_i8()?,
            rssi: input.read_i8()?,
            data: input,
        })
    }
}

pub struct CurrentTime {
    pub time: u32,
}

impl CompanionSer for CurrentTime {
    type Decoded<'data> = CurrentTime;

    fn ser_size(&self) -> usize {
        5
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);
        out.write_u8(ResponseCodes::CurrTime as u8);
        out.write_u32_le(self.time);
        out.finish()
    }

    fn companion_deserialize<'d>(mut input: &'d [u8]) -> Result<Self::Decoded<'d>, DecodeError> {
        if input.read_u8()? != ResponseCodes::CurrTime as u8 {
            return Err(DecodeError::InvalidBitPattern);
        }
        Ok(CurrentTime {
            time: input.read_u32_le()?,
        })
    }
}

pub struct LoginSuccess {
    pub permissions: Permissions,
    pub prefix: [u8; 6],
}

impl CompanionSer for LoginSuccess {
    type Decoded<'data> = LoginSuccess;

    fn ser_size(&self) -> usize {
        8
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);
        out.write_u8(ResponseCodes::LoginSuccess as u8);
        out.write_u8(self.permissions.into_bytes()[0]);
        out.write_slice(&self.prefix);
        out.finish()
    }

    fn companion_deserialize<'d>(mut input: &'d [u8]) -> Result<Self::Decoded<'d>, DecodeError> {
        if input.read_u8()? != ResponseCodes::LoginSuccess as u8 {
            return Err(DecodeError::InvalidBitPattern);
        }

        Ok(LoginSuccess {
            permissions: Permissions::from_bytes([input.read_u8()?]),
            prefix: *input.read_chunk()?,
        })
    }
}

pub struct SignStart {
    pub reserved: u8,
    pub max_len: u32,
}

impl CompanionSer for SignStart {
    type Decoded<'data> = SignStart;

    fn ser_size(&self) -> usize {
        6
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);
        out.write_u8(ResponseCodes::SignStart as u8);
        out.write_u8(self.reserved);
        out.write_u32_le(self.max_len);
        out.finish()
    }

    fn companion_deserialize<'d>(mut input: &'d [u8]) -> Result<Self::Decoded<'d>, DecodeError> {
        if input.read_u8()? != ResponseCodes::SignStart as u8 {
            return Err(DecodeError::InvalidBitPattern);
        }

        Ok(SignStart {
            reserved: input.read_u8()?,
            max_len: input.read_u32_le()?,
        })
    }
}

pub struct SignatureResponse {
    pub signature: [u8; 64],
}

impl CompanionSer for SignatureResponse {
    type Decoded<'data> = SignatureResponse;

    fn ser_size(&self) -> usize {
        65
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);
        out.write_u8(ResponseCodes::Signature as u8);
        out.write_slice(&self.signature);
        out.finish()
    }

    fn companion_deserialize<'d>(mut input: &'d [u8]) -> Result<Self::Decoded<'d>, DecodeError> {
        if input.read_u8()? != ResponseCodes::Signature as u8 {
            return Err(DecodeError::InvalidBitPattern);
        }

        Ok(SignatureResponse {
            signature: *input.read_chunk()?,
        })
    }
}

pub struct PrivateKeyResponse {
    pub key: [u8; 64],
}

impl CompanionSer for PrivateKeyResponse {
    type Decoded<'data> = PrivateKeyResponse;

    fn ser_size(&self) -> usize {
        65
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);
        out.write_u8(ResponseCodes::ExportPrivateKey as u8);
        out.write_slice(&self.key);
        out.finish()
    }

    fn companion_deserialize<'d>(mut input: &'d [u8]) -> Result<Self::Decoded<'d>, DecodeError> {
        if input.read_u8()? != ResponseCodes::ExportPrivateKey as u8 {
            return Err(DecodeError::InvalidBitPattern);
        }

        Ok(PrivateKeyResponse {
            key: *input.read_chunk()?,
        })
    }
}

pub struct CustomVars(pub Vec<(String, String)>);

impl<'a> CompanionSer for CustomVars {
    type Decoded<'data> = CustomVars;

    fn ser_size(&self) -> usize {
        1 + self
            .0
            .iter()
            .map(|(k, v)| k.len() + v.len() + 1 /* for the ':' */)
            .sum::<usize>()
            + (self.0.len().saturating_sub(1)/* for the ','s */)
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);
        out.write_u8(ResponseCodes::CustomVars as u8);
        let mut iter = self.0.iter().peekable();
        while let Some((k, v)) = iter.next() {
            out.write_slice(k.as_bytes());
            out.write_u8(b':');
            out.write_slice(v.as_bytes());
            if iter.peek().is_some() {
                out.write_u8(b',');
            }
        }

        out.finish()
    }

    fn companion_deserialize<'d>(mut input: &'d [u8]) -> Result<Self::Decoded<'d>, DecodeError> {
        if input.read_u8()? != ResponseCodes::CustomVars as u8 {
            return Err(DecodeError::InvalidBitPattern);
        }

        let data = core::str::from_utf8(input)?;
        Ok(CustomVars(
            data.split_terminator(',')
                .filter_map(|s| s.split_once(':'))
                .map(|(k, v)| (k.to_owned(), v.to_owned()))
                .collect(),
        ))
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct CoreStats {
    pub battery_mv: u16,
    pub uptime_secs: u32,
    pub errors: u16,
    pub queue_len: u8,
}

impl CompanionSer for CoreStats {
    type Decoded<'data> = CoreStats;

    fn ser_size(&self) -> usize {
        2 + 2 + 4 + 2 + 1
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);

        out.write_u8(0); // response code ??
        out.write_u8(StatTypes::Core as u8);

        out.write_u16_le(self.battery_mv);
        out.write_u32_le(self.uptime_secs);
        out.write_u16_le(self.errors);
        out.write_u8(self.queue_len);

        out.finish()
    }

    fn companion_deserialize<'d>(mut input: &'d [u8]) -> Result<Self::Decoded<'d>, DecodeError> {
        input.read_u8()?; // ?????????
        if input.read_u8()? != StatTypes::Core as u8 {
            return Err(DecodeError::InvalidBitPattern);
        }

        Ok(CoreStats {
            battery_mv: input.read_u16_le()?,
            uptime_secs: input.read_u32_le()?,
            errors: input.read_u16_le()?,
            queue_len: input.read_u8()?,
        })
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct RadioStats {
    pub noise_floor: i16,
    pub last_rssi: i8,
    pub last_snr: i8,
    pub tx_air_secs: u32,
    pub rx_air_secs: u32,
}

impl CompanionSer for RadioStats {
    type Decoded<'data> = RadioStats;

    fn ser_size(&self) -> usize {
        2 + 2 + 1 + 1 + 4 + 4
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);

        out.write_u8(0); // response code ??
        out.write_u8(StatTypes::Radio as u8);

        out.write_i16_le(self.noise_floor);
        out.write_i8(self.last_rssi);
        out.write_i8(self.last_snr * 4);
        out.write_u32_le(self.tx_air_secs);
        out.write_u32_le(self.rx_air_secs);

        out.finish()
    }

    fn companion_deserialize<'d>(mut input: &'d [u8]) -> Result<Self::Decoded<'d>, DecodeError> {
        input.read_u8()?; // ?????????
        if input.read_u8()? != StatTypes::Radio as u8 {
            return Err(DecodeError::InvalidBitPattern);
        }

        Ok(RadioStats {
            noise_floor: i16::from_le_bytes(*input.read_chunk()?),
            last_rssi: input.read_i8()?,
            last_snr: input.read_i8()? / 4,
            tx_air_secs: input.read_u32_le()?,
            rx_air_secs: input.read_u32_le()?,
        })
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PacketStats {
    pub recv: u32,
    pub sent: u32,
    pub flood_tx: u32,
    pub direct_tx: u32,
    pub flood_rx: u32,
    pub direct_rx: u32,
    pub recv_errors: u32,
}

impl CompanionSer for PacketStats {
    type Decoded<'data> = PacketStats;

    fn ser_size(&self) -> usize {
        2 + 4 * 7
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);

        out.write_u8(0);
        out.write_u8(StatTypes::Packets as u8);

        out.write_u32_le(self.recv);
        out.write_u32_le(self.sent);
        out.write_u32_le(self.flood_tx);
        out.write_u32_le(self.direct_tx);
        out.write_u32_le(self.flood_rx);
        out.write_u32_le(self.direct_rx);
        out.write_u32_le(self.recv_errors);

        out.finish()
    }

    fn companion_deserialize<'d>(mut input: &'d [u8]) -> Result<Self::Decoded<'d>, DecodeError> {
        input.read_u8()?; // ?????????
        if input.read_u8()? != StatTypes::Radio as u8 {
            return Err(DecodeError::InvalidBitPattern);
        }

        Ok(PacketStats {
            recv: input.read_u32_le()?,
            sent: input.read_u32_le()?,
            flood_tx: input.read_u32_le()?,
            direct_tx: input.read_u32_le()?,
            flood_rx: input.read_u32_le()?,
            direct_rx: input.read_u32_le()?,
            recv_errors: input.read_u32_le()?,
        })
    }
}

pub struct TraceData<'a> {
    pub reserved: u8,
    pub flags: u8,
    pub tag: [u8; 4],
    pub auth_code: [u8; 4],
    pub path: Path<'a>,
    pub snrs: Cow<'a, [i8]>,
    pub last_snr: i8,
}

impl<'a> CompanionSer for TraceData<'a> {
    type Decoded<'data> = TraceData<'data>;

    fn ser_size(&self) -> usize {
        1 // code
            + 1 // reserved
            + 1 // path_len
            + 1 // flags
            + 4 // tag
            + 4 // auth_code
            + self.path.raw_bytes().len()
            + self.snrs.len()
            + 1 // last_snr
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);

        out.write_u8(ResponseCodes::TraceData as u8);
        out.write_u8(self.reserved);
        out.write_u8(self.path.len() as u8);
        out.write_u8(self.flags | (self.path.mode as u8 & 0x03));
        out.write_slice(&self.tag);
        out.write_slice(&self.auth_code);
        out.write_slice(self.path.raw_bytes());
        out.write_slice(unsafe { core::mem::transmute::<&[i8], &[u8]>(&self.snrs) });
        out.write_i8(self.last_snr);

        out.finish()
    }

    fn companion_deserialize<'d>(mut input: &'d [u8]) -> Result<Self::Decoded<'d>, DecodeError> {
        if input.read_u8()? != ResponseCodes::TraceData as u8 {
            return Err(DecodeError::InvalidBitPattern);
        }

        let reserved = input.read_u8()?;
        let path_len = input.read_u8()?;
        let flags = input.read_u8()?; // TODO: extract path mode out of this
        let tag = *input.read_chunk()?;
        let auth = *input.read_chunk()?;

        // todo: support 2+ byte modes
        let path = Path::from_bytes(
            meshcore::PathHashMode::OneByte,
            input.read_slice(path_len as usize)?,
        );
        let snrs =
            unsafe { core::mem::transmute::<&[u8], &[i8]>(input.read_slice(path_len as usize)?) }; // todo: should this be -1?
        let last_snr = input.read_i8()?;

        Ok(TraceData {
            reserved,
            flags,
            tag,
            auth_code: auth,
            path,
            snrs: Cow::Borrowed(snrs),
            last_snr,
        })
    }
}

pub struct ControlData<'a> {
    pub snr: i8,
    pub rssi: i8,
    pub path_len: u8,
    pub payload: Cow<'a, [u8]>,
}

impl<'a> CompanionSer for ControlData<'a> {
    type Decoded<'data> = ControlData<'data>;

    fn ser_size(&self) -> usize {
        1 // type
            + 1 // snr
            + 1 // rssi
            + 1 // path_len
            + self.payload.len()
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);

        out.write_u8(ResponseCodes::ControlData as u8);
        out.write_i8(self.snr);
        out.write_i8(self.rssi);
        out.write_u8(self.path_len);
        out.write_slice(&self.payload);

        out.finish()
    }

    fn companion_deserialize<'d>(mut input: &'d [u8]) -> Result<Self::Decoded<'d>, DecodeError> {
        if input.read_u8()? != ResponseCodes::ControlData as u8 {
            return Err(DecodeError::InvalidBitPattern);
        }

        Ok(ControlData {
            snr: input.read_i8()?,
            rssi: input.read_i8()?,
            path_len: input.read_u8()?,
            payload: Cow::Borrowed(input),
        })
    }
}

pub struct BinaryResponse<'a> {
    pub data: Cow<'a, [u8]>,
}

impl<'a> CompanionSer for BinaryResponse<'a> {
    type Decoded<'data> = BinaryResponse<'data>;

    fn ser_size(&self) -> usize {
        1 // type
            + 1 // reserved
            + self.data.len()
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);
        out.write_u8(ResponseCodes::BinaryResponse as u8);
        out.write_u8(0); // reserved
        out.write_slice(&self.data);
        out.finish()
    }

    fn companion_deserialize<'d>(mut input: &'d [u8]) -> Result<Self::Decoded<'d>, DecodeError> {
        if input.read_u8()? != ResponseCodes::BinaryResponse as u8 {
            return Err(DecodeError::InvalidBitPattern);
        }

        let _reserved = input.read_u8()?;

        Ok(BinaryResponse {
            data: Cow::Borrowed(input),
        })
    }
}

// #[derive(Serialize, Deserialize)]
pub struct Contact {
    pub key: [u8; 32],
    pub name: String,
    pub path_to: Option<Path<'static>>,
    pub flags: u8,
    pub latitude: u32,
    pub longitude: u32,
    pub last_heard: u32,
}

impl CompanionSer for Contact {
    type Decoded<'data> = Contact;

    fn ser_size(&self) -> usize {
        1 // packet_ty
        + 32 // pk
        + 1 // adv_ty
        + 1 // flags
        + 1 // path_to_len 
        + 64 // path_to
        + 32 // name
        + 4 // last_heard
        + 4 // latitude
        + 4 // longitude
        + 4 // last_mod 
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);

        out.write_u8(ResponseCodes::Contact as u8);
        out.write_slice(&self.key);
        let flags = AppdataFlags::from_bits(self.flags).unwrap();
        let adv_ty = if flags.contains(AppdataFlags::IS_CHAT_NODE) {
            1
        } else if flags.contains(AppdataFlags::IS_REPEATER) {
            2
        } else if flags.contains(AppdataFlags::IS_ROOM_SERVER) {
            3
        } else {
            0
        };

        out.write_u8(adv_ty);
        out.write_u8(flags.bits());
        if let Some(path) = self.path_to.as_ref() {
            out.write_u8(path.path_len_header().into_bytes()[0]);
            NullPaddedSlice::<64>::from(path.raw_bytes()).encode_to(&mut out);
        } else {
            // flood
            out.write_u8(0xFF);
            NullPaddedSlice::<64>(&[]).encode_to(&mut out);
        }

        NullPaddedSlice::<32>::from(self.name.as_str()).encode_to(&mut out);
        out.write_u32_le(self.last_heard);
        out.write_u32_le(self.latitude);
        out.write_u32_le(self.longitude);
        out.write_u32_le(0);

        out.finish()
    }

    fn companion_deserialize<'d>(mut input: &'d [u8]) -> Result<Self::Decoded<'d>, DecodeError> {
        if input.read_u8()? != ResponseCodes::Contact as u8 {
            return Err(DecodeError::InvalidBitPattern);
        }

        let key = *input.read_chunk()?;
        let _adv_ty = input.read_u8()?;
        let flags = input.read_u8()?;

        let path_len = input.read_u8()?;

        let path = if path_len == 0xFF {
            let _nothing = input.read_chunk::<64>()?;
            None
        } else {
            let path_len = PathLen::from_bytes([path_len]);
            let NullPaddedSlice(data) = NullPaddedSlice::<'_, 64>::read(&mut input)?;
            Some(Path::from_bytes(path_len.mode(), Cow::Borrowed(data)))
        };

        let name = NullPaddedString::<'_, 32>::read(&mut input)?;
        let last_heard = input.read_u32_le()?;
        let latitude = input.read_u32_le()?;
        let longitude = input.read_u32_le()?;
        let _res = input.read_u8()?;

        Ok(Contact {
            key,
            name: name.0.to_string(),
            path_to: path.map(|v| v.to_owned()),
            flags,
            latitude,
            longitude,
            last_heard,
        })
    }
}
