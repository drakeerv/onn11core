//! Device, image and unlock constants.

pub const MODEL: &str = "onn11Core";
pub const BOARD: &str = "T405A";
pub const STOCK_BUILD_ID: &str = "BP2A.250605.031.A3";

/// LK builds the unlock payload was verified against.
pub const SUPPORTED_LK: &[(&str, &str)] = &[
    ("T405A_a16db94_202604232118", "April 2026 stock"),
    ("T405A_a16db94_202608121002", "August 2026 stock"),
];

/// "oem erase " + 24 filler + low 4 bytes of 0xffff000050f883f8.
pub const PAYLOAD: [u8; 38] = make_payload();

const fn make_payload() -> [u8; 38] {
    let mut p = [0u8; 38];
    let prefix = b"oem erase ";
    let mut i = 0;
    while i < prefix.len() {
        p[i] = prefix[i];
        i += 1;
    }
    while i < 34 {
        p[i] = b'A';
        i += 1;
    }
    p[34] = 0xf8;
    p[35] = 0x83;
    p[36] = 0xf8;
    p[37] = 0x50;
    p
}

pub struct Asset {
    pub url: &'static str,
    pub file_name: &'static str,
    pub sha256: Option<&'static str>,
    pub sha1: Option<&'static str>,
    pub md5: Option<&'static str>,
    pub size: u64,
}

pub struct TrebleDroid {
    pub asset: Asset,
    pub xz_name: &'static str,
    pub img_name: &'static str,
    pub raw_size: u64,
}

pub const TREBLEDROID: TrebleDroid = TrebleDroid {
    asset: Asset {
        url: "https://github.com/TrebleDroid/treble_experimentations/releases/download/ci-20250617/system-td-arm64-vanilla.img.xz",
        file_name: "system-td-arm64-vanilla.img.xz",
        sha256: Some("d06c66e45c6d1e8d61b45d1261a65cb7ccf2e7946739602011e500a12ae55ad6"),
        sha1: None,
        md5: None,
        size: 774_420_196,
    },
    xz_name: "system-td-arm64-vanilla.img.xz",
    img_name: "system-td-arm64-vanilla.img",
    raw_size: 2_211_401_728,
};

pub struct Lineage {
    pub asset: Asset,
    pub archive_name: &'static str,
    pub img_name: &'static str,
    pub img_size: u64,
}

pub const LINEAGE: Lineage = Lineage {
    asset: Asset {
        url: "https://github.com/drakeerv/onn11core/releases/download/v0.1.0/LineageOS-23.2-20260524-VANILLA-EXT4-GSI.7z",
        file_name: "LineageOS-23.2-20260524-VANILLA-EXT4-GSI.7z",
        sha256: Some("4eba20225472917923c4ba6f2367ba99babcd4c847b03ddd97c315745c54c2da"),
        sha1: None,
        md5: None,
        size: 922_022_276,
    },
    archive_name: "LineageOS-23.2-20260524-VANILLA-EXT4-GSI.7z",
    img_name: "LineageOS-23.2-20260524-VANILLA-EXT4-GSI.img",
    img_size: 2_733_260_800,
};

/// LineageOS for microG, with the Bluetooth seapp/build.prop fix already
/// applied. The image is hosted with the release so Windows users do not need
/// debugfs.
pub const LINEAGE_MICROG: Lineage = Lineage {
    asset: Asset {
        url: "https://github.com/drakeerv/onn11core/releases/download/v0.1.0/LineageOS-23.2-2026.11.18-microG-gsi-btfixed.img.xz",
        file_name: "LineageOS-23.2-2026.11.18-microG-gsi-btfixed.img.xz",
        sha256: Some("0051ee71d9e8d614e38bc57f464803d188c38108233d4434ab7367194c1ae892"),
        sha1: None,
        md5: None,
        size: 1_016_598_808,
    },
    archive_name: "LineageOS-23.2-2026.11.18-microG-gsi-btfixed.img.xz",
    img_name: "LineageOS-23.2-2026.11.18-microG-gsi-btfixed.img",
    img_size: 3_071_406_080,
};

/// Same LineageOS 23.2 GSI but with Google apps.
pub const LINEAGE_GAPPS: Lineage = Lineage {
    asset: Asset {
        url: "https://sourceforge.net/projects/misterztr-gsi/files/LineageOS/Android%2016/LineageOS-23.2-20260524-GAPPS-EXT4-GSI.7z/download",
        file_name: "LineageOS-23.2-20260524-GAPPS-EXT4-GSI.7z",
        sha256: None,
        sha1: Some("11e782a9207bcb9a6e163577eb5ac7ec577cfd99"),
        md5: Some("8d5a503a0793c1498e23fde4528db62f"),
        size: 1_167_859_370,
    },
    archive_name: "LineageOS-23.2-20260524-GAPPS-EXT4-GSI.7z",
    img_name: "LineageOS-23.2-20260524-GAPPS-EXT4-GSI.img",
    img_size: 3_446_353_920,
};

/// Small stock images used by the fast path and the optional root install.
/// The bundle contains only model firmware images, not per-device calibration.
pub const STOCK_SMALL: Asset = Asset {
    url: "https://github.com/drakeerv/onn11core/releases/download/v0.2.0/onn11core-stock-small.tar.gz",
    file_name: "onn11core-stock-small.tar.gz",
    sha256: Some("7ac72f1457a1b1ef84f220438b06d6c944022b15c11900f8f4fd1750ae82f491"),
    sha1: None,
    md5: None,
    size: 37_901_042,
};

/// Google GKI kernel built for the stock vendor module set.
pub const GKI: Asset = Asset {
    url: "https://github.com/drakeerv/onn11core/releases/download/v0.1.0/boot_b_gki.img",
    file_name: "boot_b_gki.img",
    sha256: Some("6e027aba53839030d00fe2cb8d390b32e8aaf1aaa93187c997c0e8b63630e89e"),
    sha1: None,
    md5: None,
    size: 18_960_384,
};

pub const DSU_USERDATA: u64 = 8 * 1024 * 1024 * 1024;

pub const SKIP_PARTITIONS: &[&str] = &["userdata", "cache", "sda", "sdb", "sdc"];

pub const MAGISK: Asset = Asset {
    url: "https://github.com/topjohnwu/Magisk/releases/download/v31.1/Magisk-v31.1.apk",
    file_name: "Magisk-v31.1.apk",
    sha256: Some("89cefac0a318afa39fb3880c45c87b036c05bc1ebf02a7a715b7f8d7c26ac6e7"),
    sha1: None,
    md5: None,
    size: 10_679_283,
};

pub const PLAY_INTEGRITY_FIX: Asset = Asset {
    url: "https://github.com/osm0sis/PlayIntegrityFork/releases/download/v18/PlayIntegrityFork-v18.zip",
    file_name: "PlayIntegrityFork-v18.zip",
    sha256: Some("625ebe65631f761e1521009f10a205d74255adf2e35732923d65106643cad9a6"),
    sha1: None,
    md5: None,
    size: 227_895,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Variant {
    Vanilla,
    MicroG,
    Gapps,
}

impl Variant {
    pub fn spec(self) -> &'static Lineage {
        match self {
            Variant::Vanilla => &LINEAGE,
            Variant::MicroG => &LINEAGE_MICROG,
            Variant::Gapps => &LINEAGE_GAPPS,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Variant::Vanilla => "LineageOS vanilla",
            Variant::MicroG => "LineageOS for microG",
            Variant::Gapps => "LineageOS with Google apps",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "vanilla" => Some(Variant::Vanilla),
            "microg" | "microG" => Some(Variant::MicroG),
            "gapps" => Some(Variant::Gapps),
            _ => None,
        }
    }
}

pub fn restore_physical(slot: &str) -> Vec<String> {
    [
        "boot",
        "init_boot",
        "vendor_boot",
        "dtbo",
        "vbmeta",
        "vbmeta_system",
        "vbmeta_vendor",
    ]
    .iter()
    .map(|p| format!("{p}_{slot}"))
    .collect()
}

pub fn slot_number(slot: &str) -> u32 {
    if slot.trim_start_matches('_') == "a" {
        0
    } else {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payload_is_the_verified_38_bytes() {
        assert_eq!(PAYLOAD.len(), 38);
        assert_eq!(&PAYLOAD[..10], b"oem erase ");
        assert_eq!(&PAYLOAD[10..34], &[b'A'; 24]);
        assert_eq!(&PAYLOAD[34..], &[0xf8, 0x83, 0xf8, 0x50]);
    }

    #[test]
    fn supported_lk_list_is_exact() {
        assert_eq!(SUPPORTED_LK.len(), 2);
    }

    #[test]
    fn variants_resolve() {
        assert_eq!(Variant::parse("microg"), Some(Variant::MicroG));
        assert_eq!(
            Variant::parse("vanilla").unwrap().spec().img_size,
            2_733_260_800
        );
    }
}
