//! User-facing data types relevant to SDO Information communications

/// The subset of indices of the object dictionary which
/// [`crate::SubDeviceRef::sdo_info_object_description_list`] makes a request for.
///
/// Defined in ETG.1000.6 §5.6.3.3.1.
///
/// Note that object quantities (value 0 in the standard) can be queried with
/// [`crate::SubDeviceRef::sdo_info_object_quantities`].
#[derive(Debug, Copy, Clone, ethercrab_wire::EtherCrabWireReadWrite)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum ObjectDescriptionListQuery {
    // ObjectQuantities is invoked through a different API
    /// All objects of the object dictionary.
    All = 0x01,
    /// Objects which are mappable in an RxPDO.
    RxPdoMappable = 0x02,
    /// Objects which are mappable in a TxPDO.
    TxPdoMappable = 0x03,
    /// Objects which have to be stored for a device replacement.
    StoredForDeviceReplacement = 0x04,
    /// Objects which can be used as startup parameter.
    StartupParameters = 0x05,
}

/// A list of object indices responsive to a [Get Object Description List](crate::subdevice::get_object_description_list) request.
pub type ObjectDescriptionList = heapless::Vec<u16, { u16::MAX as usize + 1 }>;

/// How many CoE objects on a subdevice are of each [`ObjectDescriptionListQuery`].
#[derive(Debug, Copy, Clone, PartialEq, ethercrab_wire::EtherCrabWireRead)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[wire(bytes = 10)]
pub struct ObjectDescriptionListQueryCounts {
    /// How many are of type [`ObjectDescriptionListQuery::All`].
    #[wire(bytes = 2)]
    pub all: u16,
    /// How many are of type [`ObjectDescriptionListQuery::RxPdoMappable`].
    #[wire(bytes = 2)]
    pub rx_pdo_mappable: u16,
    /// How many are of type [`ObjectDescriptionListQuery::TxPdoMappable`].
    #[wire(bytes = 2)]
    pub tx_pdo_mappable: u16,
    /// How many are of type [`ObjectDescriptionListQuery::StoredForDeviceReplacement`].
    #[wire(bytes = 2)]
    pub stored_for_device_replacement: u16,
    /// How many are of type [`ObjectDescriptionListQuery::StartupParameters`].
    #[wire(bytes = 2)]
    pub startup_parameters: u16,
}

/// Defined in ETG.1000.6 §5.6.3.5.2 Table 46.
#[derive(Debug, Clone, PartialEq, ethercrab_wire::EtherCrabWireRead)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[wire(bytes = 68)]
pub struct ObjectDescription {
    /// The type of the object
    #[wire(bytes = 2)]
    pub data_type: DataType,
    /// The number of subindices the object has minus one.
    #[wire(bytes = 1)]
    pub max_sub_index: u8,
    /// A secondary type for the object
    #[wire(bytes = 1)]
    pub object_code: ObjectCode,
    /// The human-readable name of the object
    #[wire(bytes = 64)]
    pub name: heapless::String<64>,
}

/// The object code shall denote what kind of object is at that
/// particular index within the object dictionary.
///
/// Defined in ETG.1020 §26 and CiA 301 §7.4.3 Table 42.
#[derive(Debug, Copy, Clone, PartialEq, ethercrab_wire::EtherCrabWireRead)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum ObjectCode {
    // /// An object with no data fields
    // Null = 0,
    // /// Large variable amount of data e.g. executable program code
    // Domain = 2,
    // /// Denotes a type definition such as a BOOLEAN, UNSIGNED16, FLOAT
    // /// and so on
    // DefType = 5,
    // /// Defines a new record type e.g. the PDO mapping structure at 0x21
    // DefStruct = 6,
    /// A single value such as an UNSIGNED8, BOOLEAN, FLOAT, INTEGER16,
    /// VISIBLE STRING etc.
    Variable = 7,
    /// A multiple data field object where each data field is a simple
    /// variable of the SAME basic data type e.g. array of UNSIGNED16
    /// etc. Sub-index 0 is of UNSIGNED8 and therefore not part of the
    /// ARRAY data
    Array = 8,
    /// A multiple data field object where the data fields may be any
    /// combination of simple variables. Sub-index 0 is of UNSIGNED8 and
    /// sub-index 255 is of UNSIGNED32 and therefore not part of the
    /// RECORD data
    Record = 9,
}

/// The data types which are
///
/// Defined in ETG.1020 §26 Table 119, which is a superset of ETG.1000.6
/// §5.6.7.3 Tables 64 and 65.
#[derive(Debug, Copy, Clone, PartialEq, ethercrab_wire::EtherCrabWireRead)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u16)]
#[allow(missing_docs)] // there are so many and users will pick up some docs from context
pub enum DataType {
    /// [`bool`], 0 is [`true`], 1 is [`false`].
    Boolean = 0x0001,
    /// [`u8`]
    Byte = 0x001e,
    /// [`u16`]
    Word = 0x001f,
    /// [`u32`]
    Dword = 0x0020,
    /// CiA 301 §7.1.6.5
    TimeOfDay = 0x000c,
    /// CiA 301 §7.1.6.6
    TimeDifference = 0x000d,
    /// CiA 301 §7.1.6.7
    Domain = 0x000f,

    /// # Bit strings
    Bit1 = 0x0030,
    Bit2 = 0x0031,
    Bit3 = 0x0032,
    Bit4 = 0x0033,
    Bit5 = 0x0034,
    Bit6 = 0x0035,
    Bit7 = 0x0036,
    Bit8 = 0x0037,
    Bit9 = 0x0038,
    Bit10 = 0x0039,
    Bit11 = 0x003a,
    Bit12 = 0x003b,
    Bit13 = 0x003c,
    Bit14 = 0x003d,
    Bit15 = 0x003e,
    Bit16 = 0x003f,

    /// # Bit arrays
    Bitarr8 = 0x002d,
    Bitarr16 = 0x002e,
    Bitarr32 = 0x002f,

    /// # Signed integers
    ///
    /// [`i8`]
    Integer8 = 0x0002,
    /// [`i16`]
    Integer16 = 0x0003,
    Integer24 = 0x0010,
    /// [`i32`]
    Integer32 = 0x0004,
    Integer40 = 0x0012,
    Integer48 = 0x0013,
    Integer56 = 0x0014,
    /// [`i64`]
    Integer64 = 0x0015,

    /// # Unsigned integers
    ///
    /// [`u8`]
    Unsigned8 = 0x0005,
    /// [`u16`]
    Unsigned16 = 0x0006,
    Unsigned24 = 0x0016,
    /// [`u32`]
    Unsigned32 = 0x0007,
    Unsigned40 = 0x0018,
    Unsigned48 = 0x0019,
    Unsigned56 = 0x001a,
    /// [`u64`]
    Unsigned64 = 0x001b,

    /// # Floats
    ///
    /// [`f32`]
    Real32 = 0x0008,
    /// [`f64`]
    Real64 = 0x0011,

    /// GUID
    Guid = 0x001d,

    /// Table 120: Base Data Types with variable length
    ///
    /// # Strings
    ///
    /// Structured Text `STRING(n)` (1-byte encoding)
    VisibleString = 0x0009,
    /// Structured Text `WSTRING(n)` (2-byte encoding)
    UnicodeString = 0x0268,

    /// # Octet fields, a.k.a. arrays
    ///
    /// `[u8; N]`
    OctetString = 0x000a,
    /// `[u16; N]`
    ArrayOfUint = 0x000b,
    /// `[i16; N]`
    ArrayOfInt = 0x0260,
    /// `[i8; N]`
    ArrayOfSint = 0x0261,
    /// `[i32; N]`
    ArrayOfDint = 0x0262,
    /// `[u32; N]`
    ArrayOfUdint = 0x0263,
    ArrayOfBitarr8 = 0x0264,
    ArrayOfBitarr16 = 0x0265,
    ArrayOfBitarr32 = 0x0266,
    /// `[u8; N]`
    ArrayOfUsint = 0x0267,
    /// `[f32; N]`
    ArrayOfReal = 0x0269,
    /// `[f64; N]`
    ArrayOfLreal = 0x026a,

    /// Table 121: Pre-defined records
    ///
    /// ETG.1000.6 §5.6.7.4.7 Table 74
    PdoMapping = 0x0021,
    /// ETG.1000.6 §5.6.7.4.6 Table 73
    Identity = 0x0023,
    CommandPar = 0x0025,
    /// ETG.1020 §17.2 and §17.3, Tables 57 and 59
    PdoParameter = 0x0027,
    /// ETG.1020 §15.1, Table 47
    Enum = 0x0028,
    /// ETG.1000.6 §5.5, Table 27, Object 0x10f4
    SmSynchronisation = 0x0029,
    /// No pre-defined Record structure
    Record = 0x002a,
    BackupParameter = 0x002b,
    /// ETG.5001.3 §A.2.2.1, Table 7, Object 0xf000
    ModularDeviceProfile = 0x002c,
    /// ETG.1020 §22.2.2, Table 71, Object 0x10f1
    ErrorSetting = 0x0281,
    /// ETG.1020 §16.2, Table 48, Object 0x0x10f3
    DiagnosisHistory = 0x0282,
    /// ETG.1020 §24.2.1, Table 112, Object 0x10f4
    ExternalSyncStatus = 0x0283,
    /// ETG.1020 §24.2.2, Table 114, Object 0x10f5
    ExternalSyncSettings = 0x0284,
    /// ETG.5120 §5.2.2, Table 6
    DeftypeFsoeFrame = 0x0285,
    /// ETG.5120 §5.2.3, Table 8
    DefTypeFsoeCommPar = 0x0286,

    /// Determine what kind of value this is with [`GenericDataTypeKind::from_u16`].
    #[wire(catch_all)]
    Other(u16),
}

/// Data types not specified by [`DataType`].
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum GenericDataTypeKind {
    /// Manufacturer Specific Complex Data Types
    ManufacturerComplex,
    /// Device Profile 0 Specific Standard Data Types
    DeviceProfile0Standard,
    /// Device Profile 0 Specific Complex Data Types
    DeviceProfile0Complex,
    /// Device Profile 1 Specific Standard Data Types
    DeviceProfile1Standard,
    /// Device Profile 1 Specific Complex Data Types    
    DeviceProfile1Complex,
    /// Device Profile 2 Specific Standard Data Types
    DeviceProfile2Standard,
    /// Device Profile 2 Specific Complex Data Types    
    DeviceProfile2Complex,
    /// Device Profile 3 Specific Standard Data Types
    DeviceProfile3Standard,
    /// Device Profile 3 Specific Complex Data Types    
    DeviceProfile3Complex,
    /// Device Profile 4 Specific Standard Data Types
    DeviceProfile4Standard,
    /// Device Profile 4 Specific Complex Data Types    
    DeviceProfile4Complex,
    /// Device Profile 5 Specific Standard Data Types
    DeviceProfile5Standard,
    /// Device Profile 5 Specific Complex Data Types    
    DeviceProfile5Complex,
    /// Device Profile 6 Specific Standard Data Types
    DeviceProfile6Standard,
    /// Device Profile 6 Specific Complex Data Types    
    DeviceProfile6Complex,
    /// Device Profile 7 Specific Standard Data Types
    DeviceProfile7Standard,
    /// Device Profile 7 Specific Complex Data Types    
    DeviceProfile7Complex,
    /// The data type is using a value explicitly disallowed by the standard
    ///
    /// Good luck!
    Reserved,
    /// The data type is using a value unspecified by the standard.
    Unknown,
}

impl GenericDataTypeKind {
    /// What kind of data is held by an object of type [`DataType::Other`]?
    pub fn from_u16(v: u16) -> GenericDataTypeKind {
        match v {
            // from ETG.1000.6 §5.6.7.3 Table 65
            0x0040..0x0060 => Self::ManufacturerComplex,
            0x0060..0x0080 => Self::DeviceProfile0Standard,
            0x0080..0x00a0 => Self::DeviceProfile0Complex,
            0x00a0..0x00c0 => Self::DeviceProfile1Standard,
            0x00c0..0x00e0 => Self::DeviceProfile1Complex,
            0x00e0..0x0100 => Self::DeviceProfile2Standard,
            0x0100..0x0120 => Self::DeviceProfile2Complex,
            0x0120..0x0140 => Self::DeviceProfile3Standard,
            0x0140..0x0160 => Self::DeviceProfile3Complex,
            0x0160..0x0180 => Self::DeviceProfile4Standard,
            0x0180..0x01a0 => Self::DeviceProfile4Complex,
            0x01a0..0x01c0 => Self::DeviceProfile5Standard,
            0x01c0..0x01e0 => Self::DeviceProfile5Complex,
            0x01e0..0x0200 => Self::DeviceProfile6Standard,
            0x0200..0x0220 => Self::DeviceProfile6Complex,
            0x0220..0x0240 => Self::DeviceProfile7Standard,
            0x0240..0x0260 => Self::DeviceProfile7Complex,
            // I take reserved to mean those indices specified as such in
            // ETG.1000.6 §5.6.7.3 less the addresses which get used in
            // ETG.1020 §26.
            0x000e
            | 0x0017
            | 0x001c
            | 0x0022
            | 0x0024
            | 0x0026
            | 0x026b..=0x0280
            | 0x0287..=0x07ff => Self::Reserved,
            _ => Self::Unknown,
        }
    }
}

impl std::fmt::Display for DataType {
    /// This uses the names in ETG.1020 §26
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Boolean => "BOOLEAN".into(),
                Self::Byte => "BYTE".into(),
                Self::Word => "WORD".into(),
                Self::Dword => "DWORD".into(),
                Self::TimeOfDay => "TIME_OF_DAY".into(),
                Self::TimeDifference => "TIME_DIFFERENCE".into(),
                Self::Domain => "DOMAIN".into(),

                Self::Bit1 => "BIT1".into(),
                Self::Bit2 => "BIT2".into(),
                Self::Bit3 => "BIT3".into(),
                Self::Bit4 => "BIT4".into(),
                Self::Bit5 => "BIT5".into(),
                Self::Bit6 => "BIT6".into(),
                Self::Bit7 => "BIT7".into(),
                Self::Bit8 => "BIT8".into(),
                Self::Bit9 => "BIT9".into(),
                Self::Bit10 => "BIT10".into(),
                Self::Bit11 => "BIT11".into(),
                Self::Bit12 => "BIT12".into(),
                Self::Bit13 => "BIT13".into(),
                Self::Bit14 => "BIT14".into(),
                Self::Bit15 => "BIT15".into(),
                Self::Bit16 => "BIT16".into(),

                Self::Bitarr8 => "BITARR8".into(),
                Self::Bitarr16 => "BITARR16".into(),
                Self::Bitarr32 => "BITARR32".into(),

                Self::Integer8 => "INTEGER8".into(),
                Self::Integer16 => "INTEGER16".into(),
                Self::Integer24 => "INTEGER24".into(),
                Self::Integer32 => "INTEGER32".into(),
                Self::Integer40 => "INTEGER40".into(),
                Self::Integer48 => "INTEGER48".into(),
                Self::Integer56 => "INTEGER56".into(),
                Self::Integer64 => "INTEGER64".into(),

                Self::Unsigned8 => "UNSIGNED8".into(),
                Self::Unsigned16 => "UNSIGNED16".into(),
                Self::Unsigned24 => "UNSIGNED24".into(),
                Self::Unsigned32 => "UNSIGNED32".into(),
                Self::Unsigned40 => "UNSIGNED40".into(),
                Self::Unsigned48 => "UNSIGNED48".into(),
                Self::Unsigned56 => "UNSIGNED56".into(),
                Self::Unsigned64 => "UNSIGNED64".into(),

                Self::Real32 => "REAL32".into(),
                Self::Real64 => "REAL64".into(),

                Self::Guid => "GUID".into(),

                Self::VisibleString => "VISIBLE_STRING".into(),
                Self::UnicodeString => "UNICODE_STRING".into(),

                Self::OctetString => "OCTET_STRING".into(),
                Self::ArrayOfUint => "ARRAY_OF_UINT".into(),
                Self::ArrayOfInt => "ARRAY_OF_INT".into(),
                Self::ArrayOfSint => "ARRAY_OF_SINT".into(),
                Self::ArrayOfDint => "ARRAY_OF_DINT".into(),
                Self::ArrayOfUdint => "ARRAY_OF_UDINT".into(),
                Self::ArrayOfBitarr8 => "ARRAY_OF_BITARR8".into(),
                Self::ArrayOfBitarr16 => "ARRAY_OF_BITARR16".into(),
                Self::ArrayOfBitarr32 => "ARRAY_OF_BITARR32".into(),
                Self::ArrayOfUsint => "ARRAY_OF_USINT".into(),
                Self::ArrayOfReal => "ARRAY_OF_REAL".into(),
                Self::ArrayOfLreal => "ARRAY_OF_LREAL".into(),

                Self::PdoMapping => "PDO_MAPPING".into(),
                Self::Identity => "IDENTITY".into(),
                Self::CommandPar => "COMMAND_PAR".into(),
                Self::PdoParameter => "PDO_PARAMETER".into(),
                Self::Enum => "ENUM".into(),
                Self::SmSynchronisation => "SM_SYNCHRONISATION".into(),
                Self::Record => "RECORD".into(),
                Self::BackupParameter => "BACKUP_PARAMETER".into(),
                Self::ModularDeviceProfile => "MODULAR_DEVICE_PROFILE".into(),
                Self::ErrorSetting => "ERROR_SETTING".into(),
                Self::DiagnosisHistory => "DIAGNOSIS_HISTORY".into(),
                Self::ExternalSyncStatus => "EXTERNAL_SYNC_STATUS".into(),
                Self::ExternalSyncSettings => "EXTERNAL_SYNC_SETTINGS".into(),
                Self::DeftypeFsoeFrame => "DEFTYPE_FSOEFRAME".into(),
                Self::DefTypeFsoeCommPar => "DEFTYPE_FSOECOMMPAR".into(),

                Self::Other(other) => GenericDataTypeKind::from_u16(*other).to_string(),
            }
        )
    }
}

impl std::fmt::Display for GenericDataTypeKind {
    /// This uses the names in ETG.1000.6 §5.6.7.3 Table 65
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::ManufacturerComplex => "Manufacturer Specific Complex Data Types",
                Self::DeviceProfile0Standard => "Device Profile 0 Specific Standard Data Types",
                Self::DeviceProfile0Complex => "Device Profile 0 Specific Complex Data Types",
                Self::DeviceProfile1Standard => "Device Profile 1 Specific Standard Data Types",
                Self::DeviceProfile1Complex => "Device Profile 1 Specific Complex Data Types",
                Self::DeviceProfile2Standard => "Device Profile 2 Specific Standard Data Types",
                Self::DeviceProfile2Complex => "Device Profile 2 Specific Complex Data Types",
                Self::DeviceProfile3Standard => "Device Profile 3 Specific Standard Data Types",
                Self::DeviceProfile3Complex => "Device Profile 3 Specific Complex Data Types",
                Self::DeviceProfile4Standard => "Device Profile 4 Specific Standard Data Types",
                Self::DeviceProfile4Complex => "Device Profile 4 Specific Complex Data Types",
                Self::DeviceProfile5Standard => "Device Profile 5 Specific Standard Data Types",
                Self::DeviceProfile5Complex => "Device Profile 5 Specific Complex Data Types",
                Self::DeviceProfile6Standard => "Device Profile 6 Specific Standard Data Types",
                Self::DeviceProfile6Complex => "Device Profile 6 Specific Complex Data Types",
                Self::DeviceProfile7Standard => "Device Profile 7 Specific Standard Data Types",
                Self::DeviceProfile7Complex => "Device Profile 7 Specific Complex Data Types",
                Self::Reserved => "Reserved",
                Self::Unknown => "Unknown",
            }
        )
    }
}

impl std::fmt::Display for ObjectCode {
    /// This uses the names in CiA 402 §7.4.3 Table 42
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Array => "ARRAY",
                Self::Variable => "VAR",
                Self::Record => "RECORD",
            }
        )
    }
}
