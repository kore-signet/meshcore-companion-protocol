use alloc::{borrow::Cow, string::String};
use meshcore::{
    DecodeError, Path, PathHashMode, PathLen,
    io::{SliceWriter, TinyReadExt},
    payloads::{AppdataFlags, TextType},
};
use modular_bitfield::Specifier;
use strum::{EnumDiscriminants, FromRepr, IntoDiscriminant};

use crate::{
    CompanionSer, NullPaddedString,
    responses::{Contact, ControlData, StatTypes},
};

#[derive(EnumDiscriminants)]
#[strum_discriminants(derive(FromRepr))]
#[strum_discriminants(name(HostCommandType))]
#[repr(u8)]
pub enum HostCommand<'a> {
    AppStart(AppStart<'a>) = 1,
    SendTxtMsg(SendTxtMsg<'a>) = 2,
    SendChannelTxtMsg(SendChannelTxtMsg<'a>) = 3,
    GetContacts(GetContacts) = 4,
    GetDeviceTime = 5,
    SetDeviceTime(SetDeviceTime) = 6,
    SendSelfAdvert(SendSelfAdvert) = 7,
    SetAdvertName(SetAdvertName<'a>) = 8,
    AddUpdateContact(AddUpdateContact) = 9,
    SyncNextMessage = 10,
    SetRadioParams(SetRadioParams) = 11,
    SetTxPower(SetTxPower) = 12,
    ResetPath(ResetPath) = 13,
    SetAdvertLatLon(SetAdvertLatLon) = 14,
    RemoveContact(RemoveContact) = 15,
    ShareContact(ShareContact) = 16,
    ExportContact(ExportContact) = 17,
    ImportContact(ImportContact<'a>) = 18,
    Reboot = 19,
    GetBatteryVoltage = 20,
    SetTuningParams = 21, // TODO
    DeviceQuery(DeviceQuery) = 22,
    ExportPrivateKey = 23,
    ImportPrivateKey(ImportPrivateKey) = 24,
    SendRawData(SendRawData<'a>) = 25,
    SendLogin(SendLogin<'a>) = 26,
    SendStatusReq(SendStatusReq) = 27,
    GetChannel(GetChannel) = 31,
    SetChannel(SetChannel<'a>) = 32,
    SignStart = 33,
    SignData(SignData<'a>) = 34,
    SignFinish = 35,
    SendTracePath(SendTracePath<'a>) = 36,
    SetOtherParams = 38,   // TODO
    SendTelemetryReq = 39, // TODO
    SendBinaryReq(SendBinaryReq<'a>) = 50,
    SetFloodScope(SetFloodScope<'a>) = 54,
    GetCustomVars = 40,
    SetCustomVar(SetCustomVar<'a>) = 41, // TODO
    SendControlData(ControlData<'a>) = 55,
    GetStats(GetStats) = 56,
    SendAnonReq(SendAnonReq<'a>) = 57,
}

impl<'a> CompanionSer for HostCommand<'a> {
    type Decoded<'data> = HostCommand<'data>;

    fn ser_size(&self) -> usize {
        1 + match self {
            HostCommand::AppStart(app_start) => app_start.ser_size(),
            HostCommand::SendTxtMsg(send_txt_msg) => send_txt_msg.ser_size(),
            HostCommand::SendChannelTxtMsg(send_channel_txt_msg) => send_channel_txt_msg.ser_size(),
            HostCommand::GetContacts(get_contacts) => get_contacts.ser_size(),
            HostCommand::GetDeviceTime => 0,
            HostCommand::SetDeviceTime(set_device_time) => set_device_time.ser_size(),
            HostCommand::SendSelfAdvert(send_self_advert) => send_self_advert.ser_size(),
            HostCommand::SetAdvertName(set_advert_name) => set_advert_name.ser_size(),
            HostCommand::AddUpdateContact(v) => v.ser_size(),
            HostCommand::SyncNextMessage => 0,
            HostCommand::SetRadioParams(set_radio_params) => set_radio_params.ser_size(),
            HostCommand::SetTxPower(set_tx_power) => set_tx_power.ser_size(),
            HostCommand::ResetPath(reset_path) => reset_path.ser_size(),
            HostCommand::SetAdvertLatLon(set_advert_lat_lon) => set_advert_lat_lon.ser_size(),
            HostCommand::RemoveContact(remove_contact) => remove_contact.ser_size(),
            HostCommand::ShareContact(share_contact) => share_contact.ser_size(),
            HostCommand::ExportContact(export_contact) => export_contact.ser_size(),
            HostCommand::ImportContact(import_contact) => import_contact.ser_size(),
            HostCommand::Reboot => 0,
            HostCommand::GetBatteryVoltage => 0,
            HostCommand::SetTuningParams => todo!(),
            HostCommand::DeviceQuery(device_query) => device_query.ser_size(),
            HostCommand::ExportPrivateKey => 0,
            HostCommand::ImportPrivateKey(import_private_key) => import_private_key.ser_size(),
            HostCommand::SendRawData(send_raw_data) => send_raw_data.ser_size(),
            HostCommand::SendLogin(send_login) => send_login.ser_size(),
            HostCommand::SendStatusReq(send_status_req) => send_status_req.ser_size(),
            HostCommand::GetChannel(get_channel) => get_channel.ser_size(),
            HostCommand::SetChannel(set_channel) => set_channel.ser_size(),
            HostCommand::SignStart => 0,
            HostCommand::SignData(sign_data) => sign_data.ser_size(),
            HostCommand::SignFinish => 0,
            HostCommand::SendTracePath(send_trace_path) => send_trace_path.ser_size(),
            HostCommand::SetOtherParams => todo!(),
            HostCommand::SendTelemetryReq => todo!(),
            HostCommand::SendBinaryReq(send_binary_req) => send_binary_req.ser_size(),
            HostCommand::SetFloodScope(set_flood_scope) => set_flood_scope.ser_size(),
            HostCommand::GetCustomVars => 0,
            HostCommand::SetCustomVar(a) => a.ser_size(),
            HostCommand::SendControlData(control_data) => control_data.ser_size(),
            HostCommand::GetStats(a) => a.ser_size(),
            HostCommand::SendAnonReq(a) => a.ser_size(),
        }
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        out[0] = self.discriminant() as u8;
        let s = match self {
            HostCommand::AppStart(a) => a.companion_serialize(&mut out[1..]),
            HostCommand::SendTxtMsg(a) => a.companion_serialize(&mut out[1..]),
            HostCommand::SendChannelTxtMsg(a) => a.companion_serialize(&mut out[1..]),
            HostCommand::GetContacts(a) => a.companion_serialize(&mut out[1..]),
            HostCommand::SetDeviceTime(a) => a.companion_serialize(&mut out[1..]),
            HostCommand::SendSelfAdvert(a) => a.companion_serialize(&mut out[1..]),
            HostCommand::SetAdvertName(a) => a.companion_serialize(&mut out[1..]),
            HostCommand::SetRadioParams(a) => a.companion_serialize(&mut out[1..]),
            HostCommand::SetTxPower(a) => a.companion_serialize(&mut out[1..]),
            HostCommand::ResetPath(a) => a.companion_serialize(&mut out[1..]),
            HostCommand::SetAdvertLatLon(a) => a.companion_serialize(&mut out[1..]),
            HostCommand::RemoveContact(a) => a.companion_serialize(&mut out[1..]),
            HostCommand::ShareContact(a) => a.companion_serialize(&mut out[1..]),
            HostCommand::ExportContact(a) => a.companion_serialize(&mut out[1..]),
            HostCommand::ImportContact(a) => a.companion_serialize(&mut out[1..]),
            HostCommand::DeviceQuery(a) => a.companion_serialize(&mut out[1..]),
            HostCommand::ImportPrivateKey(a) => a.companion_serialize(&mut out[1..]),
            HostCommand::SendRawData(a) => a.companion_serialize(&mut out[1..]),
            HostCommand::SendLogin(a) => a.companion_serialize(&mut out[1..]),
            HostCommand::SendStatusReq(a) => a.companion_serialize(&mut out[1..]),
            HostCommand::GetChannel(a) => a.companion_serialize(&mut out[1..]),
            HostCommand::SetChannel(a) => a.companion_serialize(&mut out[1..]),
            HostCommand::SignData(a) => a.companion_serialize(&mut out[1..]),
            HostCommand::SendTracePath(a) => a.companion_serialize(&mut out[1..]),
            HostCommand::SendBinaryReq(a) => a.companion_serialize(&mut out[1..]),
            HostCommand::SetFloodScope(a) => a.companion_serialize(&mut out[1..]),
            HostCommand::SendControlData(a) => a.companion_serialize(&mut out[1..]),
            HostCommand::SendAnonReq(a) => a.companion_serialize(&mut out[1..]),
            HostCommand::AddUpdateContact(a) => a.companion_serialize(&mut out[1..]),
            HostCommand::GetStats(a) => a.companion_serialize(&mut out[1..]),
            HostCommand::SetCustomVar(a) => a.companion_serialize(&mut out[1..]),
            _ => &[],
        };
        let s_len = s.len();
        drop(s);
        &out[0..1 + s_len]
    }

    fn companion_deserialize<'d>(
        mut input: &'d [u8],
    ) -> Result<Self::Decoded<'d>, meshcore::DecodeError> {
        Ok(
            match HostCommandType::from_repr(input.read_u8()?)
                .ok_or(DecodeError::InvalidBitPattern)?
            {
                HostCommandType::AppStart => {
                    HostCommand::AppStart(AppStart::companion_deserialize(input)?)
                }
                HostCommandType::SendTxtMsg => {
                    HostCommand::SendTxtMsg(SendTxtMsg::companion_deserialize(input)?)
                }
                HostCommandType::SendChannelTxtMsg => {
                    HostCommand::SendChannelTxtMsg(SendChannelTxtMsg::companion_deserialize(input)?)
                }
                HostCommandType::GetContacts => {
                    HostCommand::GetContacts(GetContacts::companion_deserialize(input)?)
                }
                HostCommandType::GetDeviceTime => HostCommand::GetDeviceTime,
                HostCommandType::SetDeviceTime => {
                    HostCommand::SetDeviceTime(SetDeviceTime::companion_deserialize(input)?)
                }
                HostCommandType::SendSelfAdvert => {
                    HostCommand::SendSelfAdvert(SendSelfAdvert::companion_deserialize(input)?)
                }
                HostCommandType::SetAdvertName => {
                    HostCommand::SetAdvertName(SetAdvertName::companion_deserialize(input)?)
                }
                HostCommandType::AddUpdateContact => {
                    HostCommand::AddUpdateContact(AddUpdateContact::companion_deserialize(input)?)
                }
                HostCommandType::SyncNextMessage => HostCommand::SyncNextMessage,
                HostCommandType::SetRadioParams => {
                    HostCommand::SetRadioParams(SetRadioParams::companion_deserialize(input)?)
                }
                HostCommandType::SetTxPower => {
                    HostCommand::SetTxPower(SetTxPower::companion_deserialize(input)?)
                }
                HostCommandType::ResetPath => {
                    HostCommand::ResetPath(ResetPath::companion_deserialize(input)?)
                }
                HostCommandType::SetAdvertLatLon => {
                    HostCommand::SetAdvertLatLon(SetAdvertLatLon::companion_deserialize(input)?)
                }
                HostCommandType::RemoveContact => {
                    HostCommand::RemoveContact(RemoveContact::companion_deserialize(input)?)
                }
                HostCommandType::ShareContact => {
                    HostCommand::ShareContact(ShareContact::companion_deserialize(input)?)
                }
                HostCommandType::ExportContact => {
                    HostCommand::ExportContact(ExportContact::companion_deserialize(input)?)
                }
                HostCommandType::ImportContact => {
                    HostCommand::ImportContact(ImportContact::companion_deserialize(input)?)
                }
                HostCommandType::Reboot => HostCommand::Reboot,
                HostCommandType::GetBatteryVoltage => HostCommand::GetBatteryVoltage,
                HostCommandType::SetTuningParams => todo!(),
                HostCommandType::DeviceQuery => {
                    HostCommand::DeviceQuery(DeviceQuery::companion_deserialize(input)?)
                }
                HostCommandType::ExportPrivateKey => HostCommand::ExportPrivateKey,
                HostCommandType::ImportPrivateKey => {
                    HostCommand::ImportPrivateKey(ImportPrivateKey::companion_deserialize(input)?)
                }
                HostCommandType::SendRawData => {
                    HostCommand::SendRawData(SendRawData::companion_deserialize(input)?)
                }
                HostCommandType::SendLogin => {
                    HostCommand::SendLogin(SendLogin::companion_deserialize(input)?)
                }
                HostCommandType::SendStatusReq => {
                    HostCommand::SendStatusReq(SendStatusReq::companion_deserialize(input)?)
                }
                HostCommandType::GetChannel => {
                    HostCommand::GetChannel(GetChannel::companion_deserialize(input)?)
                }
                HostCommandType::SetChannel => {
                    HostCommand::SetChannel(SetChannel::companion_deserialize(input)?)
                }
                HostCommandType::SignStart => HostCommand::SignStart,
                HostCommandType::SignData => {
                    HostCommand::SignData(SignData::companion_deserialize(input)?)
                }
                HostCommandType::SignFinish => HostCommand::SignFinish,
                HostCommandType::SendTracePath => {
                    HostCommand::SendTracePath(SendTracePath::companion_deserialize(input)?)
                }
                HostCommandType::SetOtherParams => todo!(),
                HostCommandType::SendTelemetryReq => todo!(),
                HostCommandType::SendBinaryReq => {
                    HostCommand::SendBinaryReq(SendBinaryReq::companion_deserialize(input)?)
                }
                HostCommandType::SetFloodScope => {
                    HostCommand::SetFloodScope(SetFloodScope::companion_deserialize(input)?)
                }
                HostCommandType::GetCustomVars => HostCommand::GetCustomVars,
                HostCommandType::SetCustomVar => {
                    HostCommand::SetCustomVar(SetCustomVar::companion_deserialize(input)?)
                }
                HostCommandType::SendControlData => {
                    HostCommand::SendControlData(ControlData::companion_deserialize(input)?)
                }
                HostCommandType::GetStats => {
                    HostCommand::GetStats(GetStats::companion_deserialize(input)?)
                }
                HostCommandType::SendAnonReq => {
                    HostCommand::SendAnonReq(SendAnonReq::companion_deserialize(input)?)
                }
            },
        )
    }
}

pub struct AppStart<'a> {
    pub app_ver: u8,
    pub reserved: [u8; 6],
    pub app_name: Cow<'a, str>,
}

impl<'a> CompanionSer for AppStart<'a> {
    type Decoded<'data> = AppStart<'data>;

    fn ser_size(&self) -> usize {
        1 + 6 + self.app_name.len()
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);

        out.write_u8(self.app_ver);
        out.write_slice(&self.reserved);
        out.write_slice(self.app_name.as_bytes());

        out.finish()
    }

    fn companion_deserialize<'d>(
        mut input: &'d [u8],
    ) -> Result<Self::Decoded<'d>, meshcore::DecodeError> {
        Ok(AppStart {
            app_ver: input.read_u8()?,
            reserved: *input.read_chunk()?,
            app_name: Cow::Borrowed(core::str::from_utf8(input)?),
        })
    }
}

pub struct SendTxtMsg<'a> {
    pub txt_type: TextType,
    pub attempt: u8,
    pub timestamp: u32,
    pub pubkey: [u8; 6],
    pub text: Cow<'a, str>,
}

impl<'a> CompanionSer for SendTxtMsg<'a> {
    type Decoded<'data> = SendTxtMsg<'data>;

    fn ser_size(&self) -> usize {
        1 + 1 + 4 + 6 + self.text.len()
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);

        out.write_u8(self.txt_type as u8);
        out.write_u8(self.attempt);
        out.write_u32_le(self.timestamp);
        out.write_slice(&self.pubkey);
        out.write_slice(self.text.as_bytes());

        out.finish()
    }

    fn companion_deserialize<'d>(
        mut input: &'d [u8],
    ) -> Result<Self::Decoded<'d>, meshcore::DecodeError> {
        Ok(SendTxtMsg {
            txt_type: match input.read_u8()? {
                0 => TextType::PlainText,
                1 => TextType::CliCommand,
                2 => TextType::SignedPlainText,
                _ => return Err(meshcore::DecodeError::InvalidBitPattern),
            },
            attempt: input.read_u8()?,
            timestamp: input.read_u32_le()?,
            pubkey: *input.read_chunk()?,
            text: Cow::Borrowed(core::str::from_utf8(input)?),
        })
    }
}

pub struct SendChannelTxtMsg<'a> {
    pub txt_type: TextType,
    pub channel_idx: u8,
    pub timestamp: u32,
    pub text: Cow<'a, str>,
}

impl<'a> CompanionSer for SendChannelTxtMsg<'a> {
    type Decoded<'data> = SendChannelTxtMsg<'data>;

    fn ser_size(&self) -> usize {
        1 + 1 + 4 + self.text.len()
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);

        out.write_u8(self.txt_type as u8);
        out.write_u8(self.channel_idx);
        out.write_u32_le(self.timestamp);
        out.write_slice(self.text.as_bytes());

        out.finish()
    }

    fn companion_deserialize<'d>(
        mut input: &'d [u8],
    ) -> Result<Self::Decoded<'d>, meshcore::DecodeError> {
        Ok(SendChannelTxtMsg {
            txt_type: match input.read_u8()? {
                0 => TextType::PlainText,
                1 => TextType::CliCommand,
                2 => TextType::SignedPlainText,
                _ => return Err(meshcore::DecodeError::InvalidBitPattern),
            },
            channel_idx: input.read_u8()?,
            timestamp: input.read_u32_le()?,
            text: Cow::Borrowed(core::str::from_utf8(input)?),
        })
    }
}

pub struct GetContacts {
    pub since: Option<u32>,
}

impl CompanionSer for GetContacts {
    type Decoded<'data> = GetContacts;

    fn ser_size(&self) -> usize {
        if self.since.is_some() { 4 } else { 0 }
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);
        if let Some(since) = self.since {
            out.write_u32_le(since);
        }
        out.finish()
    }

    fn companion_deserialize<'d>(
        mut input: &'d [u8],
    ) -> Result<Self::Decoded<'d>, meshcore::DecodeError> {
        Ok(GetContacts {
            since: input.read_u32_le().ok(),
        })
    }
}

// pub struct GetDeviceTime;

pub struct SetDeviceTime {
    pub timestamp: u32,
}

impl CompanionSer for SetDeviceTime {
    type Decoded<'data> = SetDeviceTime;

    fn ser_size(&self) -> usize {
        4
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);
        out.write_u32_le(self.timestamp);
        out.finish()
    }

    fn companion_deserialize<'d>(
        mut input: &'d [u8],
    ) -> Result<Self::Decoded<'d>, meshcore::DecodeError> {
        Ok(SetDeviceTime {
            timestamp: input.read_u32_le()?,
        })
    }
}

pub struct SendSelfAdvert {
    pub flood: bool,
}

impl CompanionSer for SendSelfAdvert {
    type Decoded<'data> = SendSelfAdvert;

    fn ser_size(&self) -> usize {
        1
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        out[0] = self.flood as u8;
        &out[..1]
    }

    fn companion_deserialize<'d>(
        mut input: &'d [u8],
    ) -> Result<Self::Decoded<'d>, meshcore::DecodeError> {
        Ok(SendSelfAdvert {
            flood: input.read_u8()? > 0,
        })
    }
}

pub struct SetAdvertName<'a> {
    pub name: Cow<'a, str>,
}

impl<'a> CompanionSer for SetAdvertName<'a> {
    type Decoded<'data> = SetAdvertName<'data>;

    fn ser_size(&self) -> usize {
        self.name.len()
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);
        out.write_slice(self.name.as_bytes());
        out.finish()
    }

    fn companion_deserialize<'d>(
        input: &'d [u8],
    ) -> Result<Self::Decoded<'d>, meshcore::DecodeError> {
        Ok(SetAdvertName {
            name: Cow::Borrowed(core::str::from_utf8(input)?),
        })
    }
}

// pub struct SyncNextMessage;

pub struct SetRadioParams {
    pub freq: u32,
    pub bw: u32,
    pub sf: u8,
    pub cr: u8,
}

impl CompanionSer for SetRadioParams {
    type Decoded<'data> = SetRadioParams;

    fn ser_size(&self) -> usize {
        4 + 4 + 1 + 1
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);
        out.write_u32_le(self.freq);
        out.write_u32_le(self.bw);
        out.write_u8(self.sf);
        out.write_u8(self.cr);
        out.finish()
    }

    fn companion_deserialize<'d>(
        mut input: &'d [u8],
    ) -> Result<Self::Decoded<'d>, meshcore::DecodeError> {
        Ok(SetRadioParams {
            freq: input.read_u32_le()?,
            bw: input.read_u32_le()?,
            sf: input.read_u8()?,
            cr: input.read_u8()?,
        })
    }
}

pub struct SetTxPower {
    pub tx_power: u8,
}

impl CompanionSer for SetTxPower {
    type Decoded<'data> = SetTxPower;

    fn ser_size(&self) -> usize {
        1
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);

        out.write_u8(self.tx_power);

        out.finish()
    }

    fn companion_deserialize<'d>(
        mut input: &'d [u8],
    ) -> Result<Self::Decoded<'d>, meshcore::DecodeError> {
        Ok(SetTxPower {
            tx_power: input.read_u8()?,
        })
    }
}

pub struct ResetPath {
    pub pubkey: [u8; 32],
}

impl CompanionSer for ResetPath {
    type Decoded<'data> = ResetPath;

    fn ser_size(&self) -> usize {
        32
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);
        out.write_slice(&self.pubkey);
        out.finish()
    }

    fn companion_deserialize<'d>(
        mut input: &'d [u8],
    ) -> Result<Self::Decoded<'d>, meshcore::DecodeError> {
        Ok(ResetPath {
            pubkey: *input.read_chunk()?,
        })
    }
}

pub struct SetAdvertLatLon {
    pub lat: i32,
    pub lon: i32,
}

impl CompanionSer for SetAdvertLatLon {
    type Decoded<'data> = SetAdvertLatLon;

    fn ser_size(&self) -> usize {
        4 + 4
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);

        out.write_i32_le(self.lat);
        out.write_i32_le(self.lon);

        out.finish()
    }

    fn companion_deserialize<'d>(
        mut input: &'d [u8],
    ) -> Result<Self::Decoded<'d>, meshcore::DecodeError> {
        Ok(SetAdvertLatLon {
            lat: i32::from_le_bytes(*input.read_chunk()?),
            lon: i32::from_le_bytes(*input.read_chunk()?),
        })
    }
}

pub struct RemoveContact {
    pub pubkey: [u8; 32],
}

impl CompanionSer for RemoveContact {
    type Decoded<'data> = RemoveContact;

    fn ser_size(&self) -> usize {
        32
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);
        out.write_slice(&self.pubkey);
        out.finish()
    }

    fn companion_deserialize<'d>(
        mut input: &'d [u8],
    ) -> Result<Self::Decoded<'d>, meshcore::DecodeError> {
        Ok(RemoveContact {
            pubkey: *input.read_chunk()?,
        })
    }
}

pub struct ShareContact {
    pub pubkey: [u8; 32],
}

impl CompanionSer for ShareContact {
    type Decoded<'data> = ShareContact;

    fn ser_size(&self) -> usize {
        32
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);
        out.write_slice(&self.pubkey);
        out.finish()
    }

    fn companion_deserialize<'d>(
        mut input: &'d [u8],
    ) -> Result<Self::Decoded<'d>, meshcore::DecodeError> {
        Ok(ShareContact {
            pubkey: *input.read_chunk()?,
        })
    }
}

pub struct ExportContact {
    pub pubkey: Option<[u8; 32]>,
}

impl CompanionSer for ExportContact {
    type Decoded<'data> = ExportContact;

    fn ser_size(&self) -> usize {
        if self.pubkey.is_some() { 32 } else { 0 }
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);
        if let Some(pubkey) = self.pubkey {
            out.write_slice(&pubkey);
        }
        out.finish()
    }

    fn companion_deserialize<'d>(
        mut input: &'d [u8],
    ) -> Result<Self::Decoded<'d>, meshcore::DecodeError> {
        Ok(ExportContact {
            pubkey: input.read_chunk().ok().copied(),
        })
    }
}

pub struct ImportContact<'a> {
    pub bytes: Cow<'a, [u8]>,
}

impl<'a> CompanionSer for ImportContact<'a> {
    type Decoded<'data> = ImportContact<'data>;

    fn ser_size(&self) -> usize {
        self.bytes.len()
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);
        out.write_slice(&self.bytes);
        out.finish()
    }

    fn companion_deserialize<'d>(
        input: &'d [u8],
    ) -> Result<Self::Decoded<'d>, meshcore::DecodeError> {
        Ok(ImportContact {
            bytes: Cow::Borrowed(input),
        })
    }
}

// pub struct Reboot;

// pub struct GetBatteryVoltage;

pub struct DeviceQuery {
    pub app_target_ver: u8,
}

impl CompanionSer for DeviceQuery {
    type Decoded<'data> = DeviceQuery;

    fn ser_size(&self) -> usize {
        1
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        out[0] = self.app_target_ver;
        &out[..1]
    }

    fn companion_deserialize<'d>(
        mut input: &'d [u8],
    ) -> Result<Self::Decoded<'d>, meshcore::DecodeError> {
        Ok(DeviceQuery {
            app_target_ver: input.read_u8()?,
        })
    }
}

// pub struct ExportPrivateKey;

pub struct ImportPrivateKey {
    pub private_key: [u8; 32],
}

impl CompanionSer for ImportPrivateKey {
    type Decoded<'data> = ImportPrivateKey;

    fn ser_size(&self) -> usize {
        32
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);
        out.write_slice(&self.private_key);
        out.finish()
    }

    fn companion_deserialize<'d>(
        mut input: &'d [u8],
    ) -> Result<Self::Decoded<'d>, meshcore::DecodeError> {
        Ok(ImportPrivateKey {
            private_key: *input.read_chunk()?,
        })
    }
}

pub struct SendRawData<'a> {
    pub path: Path<'a>,
    pub data: Cow<'a, [u8]>,
}

impl<'a> CompanionSer for SendRawData<'a> {
    type Decoded<'data> = SendRawData<'data>;

    fn ser_size(&self) -> usize {
        1 + self.path.byte_size() + self.data.len()
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);

        out.write_u8(self.path.path_len_header().into_bytes()[0]);
        out.write_slice(self.path.raw_bytes());
        out.write_slice(&self.data);

        out.finish()
    }

    fn companion_deserialize<'d>(
        mut input: &'d [u8],
    ) -> Result<Self::Decoded<'d>, meshcore::DecodeError> {
        let path = PathLen::from_bytes([input.read_u8()?]);
        Ok(SendRawData {
            path: Path::from_bytes(path.mode(), input.read_slice(path.byte_size())?),
            data: Cow::Borrowed(input),
        })
    }
}

pub struct SendLogin<'a> {
    pub pubkey: [u8; 32],
    pub password: Cow<'a, str>,
}

impl<'a> CompanionSer for SendLogin<'a> {
    type Decoded<'data> = SendLogin<'data>;

    fn ser_size(&self) -> usize {
        32 + self.password.len()
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);

        out.write_slice(&self.pubkey);
        out.write_slice(self.password.as_bytes());

        out.finish()
    }

    fn companion_deserialize<'d>(
        mut input: &'d [u8],
    ) -> Result<Self::Decoded<'d>, meshcore::DecodeError> {
        Ok(SendLogin {
            pubkey: *input.read_chunk()?,
            password: Cow::Borrowed(core::str::from_utf8(input)?),
        })
    }
}

pub struct SendStatusReq {
    pub pubkey: [u8; 32],
}

impl CompanionSer for SendStatusReq {
    type Decoded<'data> = SendStatusReq;

    fn ser_size(&self) -> usize {
        32
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);
        out.write_slice(&self.pubkey);
        out.finish()
    }

    fn companion_deserialize<'d>(
        mut input: &'d [u8],
    ) -> Result<Self::Decoded<'d>, meshcore::DecodeError> {
        Ok(SendStatusReq {
            pubkey: *input.read_chunk()?,
        })
    }
}

pub struct SendTelemetryReq {
    pub reserved: [u8; 3],
    pub pubkey: [u8; 32],
}

impl CompanionSer for SendTelemetryReq {
    type Decoded<'data> = SendTelemetryReq;

    fn ser_size(&self) -> usize {
        3 + 32
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);
        out.write_slice(&self.reserved);
        out.write_slice(&self.pubkey);
        out.finish()
    }

    fn companion_deserialize<'d>(
        mut input: &'d [u8],
    ) -> Result<Self::Decoded<'d>, meshcore::DecodeError> {
        Ok(SendTelemetryReq {
            reserved: *input.read_chunk()?,
            pubkey: *input.read_chunk()?,
        })
    }
}

pub struct SendBinaryReq<'a> {
    pub pubkey: [u8; 32],
    pub req_code_params: Cow<'a, [u8]>,
}

impl<'a> CompanionSer for SendBinaryReq<'a> {
    type Decoded<'data> = SendBinaryReq<'data>;

    fn ser_size(&self) -> usize {
        32 + self.req_code_params.len()
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);
        out.write_slice(&self.pubkey);
        out.write_slice(&self.req_code_params);
        out.finish()
    }

    fn companion_deserialize<'d>(
        mut input: &'d [u8],
    ) -> Result<Self::Decoded<'d>, meshcore::DecodeError> {
        Ok(SendBinaryReq {
            pubkey: *input.read_chunk()?,
            req_code_params: Cow::Borrowed(input),
        })
    }
}

pub struct SetFloodScope<'a> {
    pub reserved: u8,
    pub transport_key: Cow<'a, [u8]>,
}

impl<'a> CompanionSer for SetFloodScope<'a> {
    type Decoded<'data> = SetFloodScope<'data>;

    fn ser_size(&self) -> usize {
        1 + self.transport_key.len()
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);
        out.write_u8(self.reserved);
        out.write_slice(&self.transport_key);
        out.finish()
    }

    fn companion_deserialize<'d>(
        mut input: &'d [u8],
    ) -> Result<Self::Decoded<'d>, meshcore::DecodeError> {
        Ok(SetFloodScope {
            reserved: input.read_u8()?,
            transport_key: Cow::Borrowed(input),
        })
    }
}

pub struct GetStats {
    pub stats_type: StatTypes,
}

impl CompanionSer for GetStats {
    type Decoded<'data> = GetStats;

    fn ser_size(&self) -> usize {
        1
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        out[0] = self.stats_type as u8;
        &out[..1]
    }

    fn companion_deserialize<'d>(
        mut input: &'d [u8],
    ) -> Result<Self::Decoded<'d>, meshcore::DecodeError> {
        Ok(GetStats {
            stats_type: StatTypes::from_repr(input.read_u8()?)
                .ok_or(DecodeError::InvalidBitPattern)?,
        })
    }
}

pub struct SendChannelData<'a> {
    pub channel_idx: u8,
    pub path: Path<'a>,
    pub data_type: u16,
    pub bytes: Cow<'a, [u8]>,
}

impl<'a> CompanionSer for SendChannelData<'a> {
    type Decoded<'data> = SendChannelData<'data>;

    fn ser_size(&self) -> usize {
        1 + 1 + 2 + self.bytes.len()
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);
        out.write_u8(self.channel_idx);
        out.write_u8(self.path.path_len_header().into_bytes()[0]);
        out.write_slice(self.path.raw_bytes());
        out.write_u16_le(self.data_type);
        out.write_slice(&self.bytes);
        out.finish()
    }

    fn companion_deserialize<'d>(
        mut input: &'d [u8],
    ) -> Result<Self::Decoded<'d>, meshcore::DecodeError> {
        let channel_idx = input.read_u8()?;
        let path_mode = PathLen::from_bytes([input.read_u8()?]);
        let path = Path::from_bytes(path_mode.mode(), input.read_slice(path_mode.byte_size())?);
        Ok(SendChannelData {
            channel_idx,
            path,
            data_type: input.read_u16_le()?,
            bytes: Cow::Borrowed(input),
        })
        // Ok(SendChannelData { channel_idx: input.read_u8()?, path: (), data_type: (), bytes: () })
    }
}

pub struct GetChannel {
    pub idx: u8,
}

impl CompanionSer for GetChannel {
    type Decoded<'data> = GetChannel;

    fn ser_size(&self) -> usize {
        1
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);
        out.write_u8(self.idx);
        out.finish()
    }

    fn companion_deserialize<'d>(
        mut input: &'d [u8],
    ) -> Result<Self::Decoded<'d>, meshcore::DecodeError> {
        Ok(GetChannel {
            idx: input.read_u8()?,
        })
    }
}

pub struct SetChannel<'a> {
    pub idx: u8,
    pub name: Cow<'a, str>,
    pub secret: [u8; 16],
}

impl<'a> CompanionSer for SetChannel<'a> {
    type Decoded<'data> = SetChannel<'data>;

    fn ser_size(&self) -> usize {
        1 + 32 + 16
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);
        out.write_u8(self.idx);
        NullPaddedString::<'_, 32>(Cow::Borrowed(&self.name)).encode_to(&mut out);
        out.write_slice(&self.secret);
        out.finish()
    }

    fn companion_deserialize<'d>(
        mut input: &'d [u8],
    ) -> Result<Self::Decoded<'d>, meshcore::DecodeError> {
        let idx = input.read_u8()?;
        let NullPaddedString(name) = NullPaddedString::<'_, 32>::read(&mut input)?;
        let secret = *input.read_chunk()?;
        Ok(SetChannel { idx, name, secret })
    }
}

pub struct SignData<'a>(pub Cow<'a, [u8]>);

impl<'a> CompanionSer for SignData<'a> {
    type Decoded<'data> = SignData<'data>;

    fn ser_size(&self) -> usize {
        self.0.len()
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);
        out.write_slice(&self.0);
        out.finish()
    }

    fn companion_deserialize<'d>(
        input: &'d [u8],
    ) -> Result<Self::Decoded<'d>, meshcore::DecodeError> {
        Ok(SignData(Cow::Borrowed(input)))
    }
}

// pub struct SignFinish;

pub struct SendTracePath<'a> {
    pub tag: [u8; 4],
    pub auth: [u8; 4],
    pub flags: u8,
    pub path: Path<'a>,
}

impl<'a> CompanionSer for SendTracePath<'a> {
    type Decoded<'data> = SendTracePath<'data>;

    fn ser_size(&self) -> usize {
        4 + 4 + 1 + self.path.len()
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);
        out.write_slice(&self.tag);
        out.write_slice(&self.auth);
        out.write_u8(self.flags);
        out.write_slice(self.path.raw_bytes());
        out.finish()
    }

    fn companion_deserialize<'d>(
        mut input: &'d [u8],
    ) -> Result<Self::Decoded<'d>, meshcore::DecodeError> {
        let tag = *input.read_chunk()?;
        let auth = *input.read_chunk()?;
        let flags = input.read_u8()?;

        Ok(SendTracePath {
            tag,
            auth,
            flags,
            path: Path::from_bytes(
                PathHashMode::from_bytes(flags & 0x03)
                    .map_err(|_| DecodeError::InvalidBitPattern)?,
                input,
            ),
        })
    }
}

pub struct SendAnonReq<'a> {
    pub pubkey: [u8; 32],
    pub data: Cow<'a, [u8]>,
}

impl<'a> CompanionSer for SendAnonReq<'a> {
    type Decoded<'data> = SendAnonReq<'data>;

    fn ser_size(&self) -> usize {
        32 + self.data.len()
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);
        out.write_slice(&self.pubkey);
        out.write_slice(&self.data);
        out.finish()
    }

    fn companion_deserialize<'d>(mut input: &'d [u8]) -> Result<Self::Decoded<'d>, DecodeError> {
        Ok(SendAnonReq {
            pubkey: *input.read_chunk()?,
            data: Cow::Borrowed(input),
        })
    }
}

pub struct AddUpdateContact(pub Contact);

impl CompanionSer for AddUpdateContact {
    type Decoded<'data> = AddUpdateContact;

    fn ser_size(&self) -> usize {
        32 + 1 + 1 + 1 + self.0.path_to.as_ref().map_or(0, |v| v.byte_size()) + 32 + 4 + 4 + 4
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);

        let Contact {
            key,
            name,
            path_to,
            flags,
            latitude,
            longitude,
            last_heard,
        } = &self.0;

        out.write_slice(key);
        let flags = AppdataFlags::from_bits(*flags).unwrap();
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
        if let Some(path) = path_to {
            out.write_u8(path.path_len_header().into_bytes()[0]);
            out.write_slice(path.raw_bytes());
        } else {
            out.write_u8(0)
        }

        NullPaddedString::<'_, 32>(Cow::Borrowed(name.as_str())).encode_to(&mut out);
        out.write_u32_le(*last_heard);
        out.write_u32_le(*latitude);
        out.write_u32_le(*longitude);

        out.finish()
    }

    fn companion_deserialize<'d>(mut input: &'d [u8]) -> Result<Self::Decoded<'d>, DecodeError> {
        let pk = input.read_chunk::<32>()?;
        let _ty = input.read_u8()?;
        let flags = input.read_u8()?;
        let out_path_len = PathLen::from_bytes([input.read_u8()?]);
        let path = if out_path_len.byte_size() > 0 {
            Some(Path::from_bytes(
                out_path_len.mode(),
                input.read_slice(out_path_len.byte_size())?,
            ))
        } else {
            None
        };
        let name = NullPaddedString::<'_, 32>::read(&mut input)?;
        let last_adv = input.read_u32_le()?;
        let lat = input.read_u32_le()?;
        let long = input.read_u32_le()?;
        Ok(AddUpdateContact(Contact {
            key: *pk,
            name: String::from(name.0),
            path_to: path.map(|v| v.to_owned()),
            flags,
            latitude: lat,
            longitude: long,
            last_heard: last_adv,
        }))
    }
}

pub struct SetCustomVar<'a> {
    pub key: Cow<'a, str>,
    pub value: Cow<'a, str>,
}

impl<'a> CompanionSer for SetCustomVar<'a> {
    type Decoded<'data> = SetCustomVar<'data>;

    fn ser_size(&self) -> usize {
        self.key.len() + 1 + self.value.len()
    }

    fn companion_serialize<'d>(&self, out: &'d mut [u8]) -> &'d [u8] {
        let mut out = SliceWriter::new(out);
        out.write_slice(self.key.as_bytes());
        out.write_u8(b':');
        out.write_slice(self.value.as_bytes());
        out.finish()
    }

    fn companion_deserialize<'d>(input: &'d [u8]) -> Result<Self::Decoded<'d>, DecodeError> {
        let s = core::str::from_utf8(input)?;
        let Some((key, val)) = s.split_once(':') else {
            return Err(DecodeError::InvalidBitPattern);
        };
        Ok(SetCustomVar {
            key: Cow::Borrowed(key),
            value: Cow::Borrowed(val),
        })
    }
}
