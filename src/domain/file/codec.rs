pub enum VideoCodec {
    H264,
    Hevc,
    Av1,
    Unknown(String),
}

impl VideoCodec {
    pub fn needs_transcoding(&self) -> bool {
        match self {
            Self::H264 => true,
            Self::Hevc => false,
            Self::Av1 => false,
            Self::Unknown(_) => true,
        }
    }
    pub fn is_known(&self) -> bool {
        !matches!(self, Self::Unknown(_))
    }
}

impl std::str::FromStr for VideoCodec {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "h264" => Ok(Self::H264),
            "hevc" => Ok(Self::Hevc),
            "av1" => Ok(Self::Av1),
            other => Ok(Self::Unknown(other.to_string())),
        }
    }
}

impl std::fmt::Display for VideoCodec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::H264 => write!(f, "h264"),
            Self::Hevc => write!(f, "hevc"),
            Self::Av1 => write!(f, "av1"),
            Self::Unknown(other) => write!(f, "{other}"),
        }
    }
}
