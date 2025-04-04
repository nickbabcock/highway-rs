use highway::{HighwayHash, HighwayHasher, Key, PortableHash};

#[test]
fn hash_zeroes() {
    let key = Key([0, 0, 0, 0]);
    let hash = PortableHash::new(key).hash64(&[]);
    assert_eq!(0x7035_DA75_B9D5_4469, hash);
}

#[test]
fn portable_hash_simple() {
    let key = Key([1, 2, 3, 4]);
    let b: Vec<u8> = (0..33).map(|x| 128 + x as u8).collect();
    let hash = PortableHash::new(key).hash64(&b[..]);
    assert_eq!(0x53c5_16cc_e478_cad7, hash);
}

#[test]
fn portable_hash_append() {
    let key = Key([1, 2, 3, 4]);
    let b: Vec<u8> = (0..33).map(|x| 128 + x as u8).collect();
    let mut hasher = PortableHash::new(key);
    hasher.append(&b[..]);
    let hash = hasher.finalize64();
    assert_eq!(0x53c5_16cc_e478_cad7, hash);
}

#[test]
fn portable_hash_simple2() {
    let key = Key([1, 2, 3, 4]);
    let hash = PortableHash::new(key).hash64(&[-1_i8 as u8]);
    assert_eq!(0x7858_f24d_2d79_b2b2, hash);
}

#[test]
fn portable_hash_append2() {
    let key = Key([1, 2, 3, 4]);
    let mut hasher = PortableHash::new(key);
    hasher.append(&[-1_i8 as u8]);
    let hash = hasher.finalize64();
    assert_eq!(0x7858_f24d_2d79_b2b2, hash);
}

pub fn hash_all() {
    let expected64 = [
        0x907A_56DE_22C2_6E53,
        0x7EAB_43AA_C7CD_DD78,
        0xB8D0_569A_B0B5_3D62,
        0x5C6B_EFAB_8A46_3D80,
        0xF205_A468_9300_7EDA,
        0x2B8A_1668_E4A9_4541,
        0xBD4C_CC32_5BEF_CA6F,
        0x4D02_AE17_38F5_9482,
        0xE120_5108_E55F_3171,
        0x32D2_644E_C77A_1584,
        0xF6E1_0ACD_B103_A90B,
        0xC3BB_F461_5B41_5C15,
        0x243C_C204_0063_FA9C,
        0xA89A_58CE_65E6_41FF,
        0x24B0_31A3_4845_5A23,
        0x4079_3F86_A449_F33B,
        0xCFAB_3489_F97E_B832,
        0x19FE_67D2_C8C5_C0E2,
        0x04DD_90A6_9C56_5CC2,
        0x75D9_518E_2371_C504,
        0x38AD_9B11_41D3_DD16,
        0x0264_432C_CD8A_70E0,
        0xA9DB_5A62_8868_3390,
        0xD7B0_5492_003F_028C,
        0x205F_615A_EA59_E51E,
        0xEEE0_C896_2105_2884,
        0x1BFC_1A93_A728_4F4F,
        0x5121_75B5_B70D_A91D,
        0xF71F_8976_A0A2_C639,
        0xAE09_3FEF_1F84_E3E7,
        0x22CA_92B0_1161_860F,
        0x9FC7_007C_CF03_5A68,
        0xA0C9_64D9_ECD5_80FC,
        0x2C90_F73C_A031_81FC,
        0x185C_F84E_5691_EB9E,
        0x4FC1_F5EF_2752_AA9B,
        0xF5B7_391A_5E0A_33EB,
        0xB9B8_4B83_B4E9_6C9C,
        0x5E42_FE71_2A5C_D9B4,
        0xA150_F2F9_0C3F_97DC,
        0x7FA5_22D7_5E2D_637D,
        0x181A_D0CC_0DFF_D32B,
        0x3889_ED98_1E85_4028,
        0xFB42_97E8_C586_EE2D,
        0x6D06_4A45_BB28_059C,
        0x9056_3609_B3EC_860C,
        0x7AA4_FCE9_4097_C666,
        0x1326_BAC0_6B91_1E08,
        0xB926_168D_2B15_4F34,
        0x9919_8489_45B1_948D,
        0xA2A9_8FC5_3482_5EBE,
        0xE980_9095_213E_F0B6,
        0x582E_5483_707B_C0E9,
        0x086E_9414_A88A_6AF5,
        0xEE86_B98D_20F6_743D,
        0xF89B_7FF6_09B1_C0A7,
        0x4C7D_9CC1_9E22_C3E8,
        0x9A97_0050_2456_2A6F,
        0x5DD4_1CF4_23E6_EBEF,
        0xDF13_609C_0468_E227,
        0x6E0D_A4F6_4188_155A,
        0xB755_BA4B_50D7_D4A1,
        0x887A_3484_6474_79BD,
        0xAB8E_EBE9_BF21_39A0,
        0x7554_2C5D_4CD2_A6FF,
    ];

    let expected128 = [
        0x33565E767F093E6F_0FED268F9D8FFEC7,
        0xDC291DF9EB9CDCB4_D6B0A8893681E7A8,
        0x78085638DC32E868_3D15AD265A16DA04,
        0xBFE69A0FD9CEDD79_0607621B295F0BEB,
        0x2E922AD039319208_26399EB46DACE49E,
        0x193810906C63C23A_3250BDC386D12ED8,
        0x7CDE576F37ED1019_6F476AB3CB896547,
        0xBE1F03FF9F02796C_2A401FCA697171B4,
        0x695CF1C63BEC0AC2_A1E96D84280552E8,
        0x1A85B98C5B5000CC_142A2102F31E63B2,
        0x929E1F3B2DA45559_51A1B70E26B6BC5B,
        0xBED21F22C47B7D13_88990362059A415B,
        0xA818BA8CE0F9C8D4_CD1F1F5F1CAF9566,
        0xB2E94C78B8DDB848_A225564112FE6157,
        0xCECD1DBC025641A2_BD492FEBD1CC0919,
        0xE0796C0B6E26BCD7_142237A52BC4AF54,
        0x029EA3D5019F18C8_414460FFD5A401AD,
        0xECB878B1169B5EA0_C52A4B96C51C9962,
        0xF93A46D616F8D531_D940CA8F11FBEACE,
        0x3FFDBF8DF51D7C93_8AC49D0AE5C0CBF5,
        0x7DCD3A6BA5EBAA46_AC6D279B852D00A8,
        0x3173C398163DD9D5_F11621BD93F08A56,
        0xB3123CDA411898ED_0C4CE250F68CF89F,
        0x7CE274479169080E_15AB97ED3D9A51CE,
        0xD0D9D98BD8AA2D77_CD001E198D4845B8,
        0x7DD304F6397F7E16_34F3D617A0493D79,
        0x130829166567304F_5CB56890A9F4C6B6,
        0x6F828B7E3FD9748C_30DA6F8B245BD1C0,
        0x93F6DA0CAC5F441C_E0580349204C12C0,
        0x5FB897114FB65976_F648731BA5073045,
        0x509A4918EB7E0991_024F8354738A5206,
        0x52415E3A07F5D446_06E7B465E8A57C29,
        0x16FC1958F9B3E4B9_1984DF66C1434AAA,
        0xF958B59DE5A2849D_111678AFE0C6C36C,
        0xC96ED5D243658536_773FBC8440FB0490,
        0xEA336A0BC1EEACE9_91E3DC710BB6C941,
        0xF2E94F8C828FC59E_25CFE3815D7AD9D4,
        0x7479C4C8F850EC04_B9FB38B83CC288F2,
        0x6E26B1C16F48DBF4_1D85D5C525982B8C,
        0x2134D599058B3FD0_8A4E55BD6060BDE7,
        0xE8052D1AE61D6423_2A958FF994778F36,
        0x3ACF9C87D7E8C0B9_89233AE6BE453233,
        0x418FB49BCA2A5140_4458F5E27EA9C8D5,
        0x1017F69633C861E6_090301837ED12A68,
        0x339DF1AD3A4BA6E4_330DD84704D49590,
        0x363B3D95E3C95EF6_569363A663F2C576,
        0x2BA0E8087D4E28E9_ACC8D08586B90737,
        0x8DB620A45160932E_39C27A27C86D9520,
        0x6ED3561A10E47EE6_8E6A4AEB671A072D,
        0xD80E6E656EDE842E_0011D765B1BEC74A,
        0xCE088794D7088A7D_2515D62B936AC64C,
        0x264F0094EB23CCEF_91621552C16E23AF,
        0xD8654807D3A31086_1E21880D97263480,
        0xA517E1E09D074739_39D76AAF097F432D,
        0x2F51215F69F976D4_0F17A4F337C65A14,
        0x568C3DC4D1F13CD1_A0FB5CDA12895E44,
        0xBAD5DA947E330E69_93C8FC00D89C46CE,
        0x584D6EE72CBFAC2B_817C07501D1A5694,
        0xF98E647683C1E0ED_91D668AF73F053BF,
        0xBC4CC3DF166083D8_5281E1EF6B3CCF8B,
        0xFF969D000C16787B_AAD61B6DBEAAEEB9,
        0x14B919BD905F1C2D_4325D84FC0475879,
        0xF1F720C5A53A2B86_79A176D1AA6BA6D1,
        0x3AEA94A8AD5F4BCB_74BD7018022F3EF0,
        0xE0BC0571DE918FC8_98BB1F7198D4C4F2,
    ];

    let expected256 = [
        (
            0xD946017313C7351F_DD44482AC2C874F5,
            0x41DA233145751DF4_B3AEBECCB98714FF,
        ),
        (
            0xE20D44EF3DCAC60F_EDB941BCE45F8254,
            0x2073624CB275E484_72651B9BCB324A47,
        ),
        (
            0x11C4BF1A1B0AE873_3FDFF9DF24AFE454,
            0x1208F6590D33B42C_115169CC6922597A,
        ),
        (
            0x89225E7C6911D1D0_480AA0D70DD1D95C,
            0xE23DFBC390E1C722_8EA8426B8BBB865A,
        ),
        (
            0xA85F9DF6AFD2929B_C9CFC497212BE4DC,
            0x07E4277A374D4F9B_1FDA9F211DF4109E,
        ),
        (
            0xBF4B63BA5E460142_B4B4F566A4DC85B3,
            0x0F74587D388085C6_15F48E68CDDC1DE3,
        ),
        (
            0xA99CFB2784B4CEB6_6445C70A86ADB9B4,
            0xB6526DF29A9D1170_DAE29D40A0B2DB13,
        ),
        (
            0xA4F1F838EB8C6D37_D666B1A00987AD81,
            0x5754D67D062C526C_E9226E07D463E030,
        ),
        (
            0xE6976FF3FCFF3A45_F1B905B0ED768BC0,
            0xD9A0AFEB371E0D33_4FBE518DD9D09778,
        ),
        (
            0xF10FBBD16424F1A1_80D8E4D70D3C2981,
            0xC0BFE8F701B673F2_CF5C2DBE9D3F0CD1,
        ),
        (
            0x8E9492B1FDFE38E0_ADE48C50E5A262BE,
            0x0E41D574DB656DCD_0784B74B2FE9B838,
        ),
        (
            0xBA97A7DE6A1A9738_A1BE77B9531807CF,
            0x3E39B935C74CE8E8_AF274CEF9C8E261F,
        ),
        (
            0x9D11CBDC39E853A0_15AD3802E3405857,
            0x6CD9E9E3CAF4212E_23EA3E993C31B225,
        ),
        (
            0xA367F9C1531F95A6_01C96F5EB1D77C36,
            0x97F1000ABF3BD5D3_1F94A3427CDADCB8,
        ),
        (
            0x0E0C28FA6E21DF5D_0815E91EEEFF8E41,
            0x3FFD01DA1C9D73E6_4EAD8E62ED095374,
        ),
        (
            0x62C3DB018501B146_C11905707842602E,
            0xC884F87BD4FEC347_85F5AD17FA3406C1,
        ),
        (
            0xF7F075D62A627BD9_F51AD989A1B6CD1F,
            0x1AD415C16A174D9F_7E01D5F579F28A06,
        ),
        (
            0x3B9D4ABD3A9275B9_19F4CFA82CA4068E,
            0x8884D50949215613_8000B0DDE9C010C6,
        ),
        (
            0x4EDAA3C5097716EE_126D6C7F81AB9F5D,
            0x9001AC85AA80C32D_AF121573A7DD3E49,
        ),
        (
            0xDF864F4144E71C3D_06AABEF9149155FA,
            0xDE2BA54792491CB6_FDBABCE860BC64DA,
        ),
        (
            0xA087B7328E486E65_ADFC6B4035079FDB,
            0xE3895C440D3CEE44_46D1A9935A4623EA,
        ),
        (
            0x8F3024E20A06E133_B5F9D31DEEA3B3DF,
            0x703F1DCF9BD69749_F24C38C8288FE120,
        ),
        (
            0x1C5D3F969BDACEA0_2B3C0B854794EFE3,
            0x23441C5A79D03075_81F16AAFA563AC2E,
        ),
        (
            0xBC6B8E9461D7F924_418AF8C793FD3762,
            0x3AA0B7BFD417CA6E_776FF26A2A1A9E78,
        ),
        (
            0x0185FEE5B59C1B2A_CD03EA2AD255A3C1,
            0xBE69DD67F83B76E4_D1F438D44F9773E4,
        ),
        (
            0x2C7B31D2A548E0AE_F951A8873887A0FB,
            0xA3C78EC7BE219F72_44803838B6186EFA,
        ),
        (
            0x4B7E8997B4F63488_958FF151EA0D8C08,
            0xD95577556F20EEFA_C78E074351C5386D,
        ),
        (
            0x3318F884351F578C_29A917807FB05406,
            0xE74393465E97AEFF_DD24EA6EF6F6A7FA,
        ),
        (
            0x1FD0D271B09F97DA_98240880935E6CCB,
            0x291649F99F747817_56E786472700B183,
        ),
        (
            0xFFDB2EFF7C596CEB_1BD4954F7054C556,
            0x0F037670537FC153_7C6AC69A1BAB6B5B,
        ),
        (
            0x647CF6EBAF6332C1_8825E38897597498,
            0x72D7632C00BFC5AB_552BD903DC28C917,
        ),
        (
            0xB3728B20B10FB7DA_6880E276601A644D,
            0x8AEF14EF33452EF2_D0BD12060610D16E,
        ),
        (
            0x42D56326A3C11289_BCE38C9039A1C3FE,
            0xC9B03C6BC9475A99_E35595F764FCAEA9,
        ),
        (
            0x6C36EA75BFCE46D0_F60115CBF034A6E5,
            0x7EDAA2ED11007A35_3B17C8D382725990,
        ),
        (
            0xC4776801739F720C_1326E959EDF9DEA2,
            0x8A0DD0D90A2529AB_5169500FD762F62F,
        ),
        (
            0xFF6BB41302DAD144_935149D503D442D4,
            0xE61D53619ECC2230_339CB012CD9D36EC,
        ),
        (
            0xB8AEECA36084E1FC_528BC888AA50B696,
            0x02C14AAD097CEC44_A158151EC0243476,
        ),
        (
            0x1EE65114F760873F_BED688A72217C327,
            0xDDF2E895631597B9_3F5C26B37D3002A6,
        ),
        (
            0xFAFC6324F4B0AB6C_E7DB21CF2B0B51AD,
            0xF0AD888D1E05849C_B0857244C22D9C5B,
        ),
        (
            0x3C594A3163067DEB_05519793CD4DCB00,
            0x5AC86297805CB094_AC75081ACF119E34,
        ),
        (
            0x19644DB2516B7E84_09228D8C22B5779E,
            0x7F785AD725E19391_2B92C8ABF83141A0,
        ),
        (
            0x5EA53C65CA036064_59C42E5D46D0A74B,
            0xBAE6DF143F54E9D4_48A9916BB635AEB4,
        ),
        (
            0xD53D78BCB41DA092_5EB623696D03D0E3,
            0x64802457632C8C11_FE2348DC52F6B10D,
        ),
        (
            0xC6318C25717E80A1_43B61BB2C4B85481,
            0xBD0217E035401D7C_8C4A7F4D6F9C687D,
        ),
        (
            0xB04C4D5EB11D703A_7F51CA5743824C37,
            0xD66775EA215456E2_4D511E1ECBF6F369,
        ),
        (
            0x52B8E8C459FC79B3_39B409EEF87E45CC,
            0x80F07B645EEE0149_44920918D1858C24,
        ),
        (
            0xBFA19026526836E7_CE8694D1BE9AD514,
            0x380C4458D696E1FE_1EA4FDF6E4902A7D,
        ),
        (
            0x1F3B353BE501A7D7_D189E18BF823A0A4,
            0x7E94646F74F9180C_A24F77B4E02E2884,
        ),
        (
            0x2C2E0AA414038D0B_AFF8C635D325EC48,
            0x39EC38E33B501489_4ED37F611A447467,
        ),
        (
            0x013D3E6EBEF274CC_2A2BFDAD5F83F197,
            0xF15A8A5DE932037E_E1563C0477726155,
        ),
        (
            0x10110B9BF9B1FF11_D5D1F91EC8126332,
            0x87BADC5728701552_A175AB26541C6032,
        ),
        (
            0xDDA62AB61B2EEEFB_C7B5A92CD8082884,
            0x6B38BD5CC01F4FFB_8F9882ECFEAE732F,
        ),
        (
            0xA3F0822DA2BF7D8B_CF6EF275733D32F0,
            0x0B28E3EFEBB3172D_304E7435F512406A,
        ),
        (
            0x66AE2A819A8A8828_E698F80701B2E9DB,
            0xA7416170523EB5A4_14EA9024C9B8F2C9,
        ),
        (
            0x17B4DEDAE34452C1_3A917E87E307EDB7,
            0x29CE6BFE789CDD0E_F689F162E711CC70,
        ),
        (
            0x47CD9EAD4C0844A2_0EFF3AD8CB155D8E,
            0xDEF3C25DF0340A51_46C8E40EE6FE21EB,
        ),
        (
            0x32AB0D600717136D_03FD86E62B82D04D,
            0x138CE3F1443739B1_682B0E832B857A89,
        ),
        (
            0x1053E0A9D9ADBFEA_2F77C754C4D7F902,
            0xC48A829C72DD83CA_58E66368544AE70A,
        ),
        (
            0x31BE9E01A8C7D314_F900EB19E466A09F,
            0xB8C0EB0F87FFE7FB_3AFEC6B8CA08F471,
        ),
        (
            0x53CE6877E11AA57B_DB277D8FBE3C8EFB,
            0xB345B56392453CC9_719C94D20D9A7E7D,
        ),
        (
            0x6095E7B336466DC8_37639C3BDBA4F2C9,
            0x82C988CDE5927CD5_3A8049791E65B88A,
        ),
        (
            0x20562E255BA6467E_6B1FB1A714234AE4,
            0xF40CE3FBE41ED768_3E2B892D40F3D675,
        ),
        (
            0x8FC2AAEFF63D266D_8EE11CB1B287C92A,
            0x578AA91DE8D56873_66643487E6EB9F03,
        ),
        (
            0x83B040BE4DEC1ADD_F5B1F8266A3AEB67,
            0xF4A3A447DEFED79F_7FE1C8635B26FBAE,
        ),
        (
            0x1A422A196EDAC1F2_90D8E6FF6AC12475,
            0xC1BDD7C4C351CFBE_9E3765FE1F8EB002,
        ),
    ];
    let data: Vec<u8> = (0..65).map(|x| x as u8).collect();
    let key = Key([
        0x0706_0504_0302_0100,
        0x0F0E_0D0C_0B0A_0908,
        0x1716_1514_1312_1110,
        0x1F1E_1D1C_1B1A_1918,
    ]);

    for i in 0..64 {
        println!("{}", i);
        let res_128 = u64_to_u128(&HighwayHasher::new(key).hash128(&data[..i])[..]);
        let res_256 = u64_to_u256(&HighwayHasher::new(key).hash256(&data[..i])[..]);
        assert_eq!(expected64[i], HighwayHasher::new(key).hash64(&data[..i]));
        assert_eq!(expected128[i], res_128);
        assert_eq!(expected256[i], res_256);

        assert_eq!(expected64[i], {
            let mut hasher = HighwayHasher::new(key);
            hasher.append(&data[..i]);
            hasher.finalize64()
        });
        assert_eq!(expected128[i], {
            let mut hasher = HighwayHasher::new(key);
            hasher.append(&data[..i]);
            u64_to_u128(&hasher.finalize128()[..])
        });
        assert_eq!(expected256[i], {
            let mut hasher = HighwayHasher::new(key);
            hasher.append(&data[..i]);
            u64_to_u256(&hasher.finalize256()[..])
        });
    }
}

#[test]
fn test_hash_all() {
    hash_all();
}

fn u64_to_u128(data: &[u64]) -> u128 {
    u128::from(data[0]) + (u128::from(data[1]) << 64)
}

fn u64_to_u256(data: &[u64]) -> (u128, u128) {
    (u64_to_u128(data), u64_to_u128(&data[2..]))
}

#[cfg(target_arch = "x86_64")]
#[test]
fn sse_hash_zeroes() {
    use highway::SseHash;

    if !is_x86_feature_detected!("sse4.1") {
        return;
    }

    let key = Key([0, 0, 0, 0]);
    let hash = unsafe { SseHash::force_new(key).hash64(&[]) };
    assert_eq!(0x7035_DA75_B9D5_4469, hash);
}

#[cfg(target_arch = "x86_64")]
#[test]
fn sse_hash_eq_portable() {
    use highway::SseHash;

    if !is_x86_feature_detected!("sse4.1") {
        return;
    }

    let data: Vec<u8> = (0..100).map(|x| x as u8).collect();
    let key = Key([
        0x0706_0504_0302_0100,
        0x0F0E_0D0C_0B0A_0908,
        0x1716_1514_1312_1110,
        0x1F1E_1D1C_1B1A_1918,
    ]);

    for i in 0..data.len() {
        println!("{}", i);
        let hash64 = PortableHash::new(key).hash64(&data[..i]);
        assert_eq!(
            unsafe { SseHash::force_new(key) }.hash64(&data[..i]),
            hash64
        );

        let (head, tail) = &data[..i].split_at(i / 2);
        let mut hasher = unsafe { SseHash::force_new(key) };
        hasher.append(head);
        let mut snd = unsafe { SseHash::force_from_checkpoint(hasher.checkpoint()) };
        snd.append(tail);
        assert_eq!(hash64, snd.finalize64());

        assert_eq!(
            unsafe { SseHash::force_new(key) }.hash128(&data[..i]),
            PortableHash::new(key).hash128(&data[..i])
        );

        assert_eq!(
            unsafe { SseHash::force_new(key) }.hash256(&data[..i]),
            PortableHash::new(key).hash256(&data[..i])
        );
    }
}

#[test]
#[cfg(target_arch = "x86_64")]
fn avx_hash_eq_portable() {
    use highway::AvxHash;
    if !is_x86_feature_detected!("avx2") {
        return;
    }

    let data: Vec<u8> = (0..100).map(|x| x as u8).collect();
    let key = Key([
        0x0706_0504_0302_0100,
        0x0F0E_0D0C_0B0A_0908,
        0x1716_1514_1312_1110,
        0x1F1E_1D1C_1B1A_1918,
    ]);

    for i in 0..100 {
        println!("{}", i);
        unsafe {
            assert_eq!(
                PortableHash::new(key).hash64(&data[..i]),
                AvxHash::force_new(key).hash64(&data[..i])
            );

            assert_eq!(
                PortableHash::new(key).hash128(&data[..i]),
                AvxHash::force_new(key).hash128(&data[..i])
            );

            assert_eq!(
                PortableHash::new(key).hash256(&data[..i]),
                AvxHash::force_new(key).hash256(&data[..i])
            );
        }
    }
}

#[test]
fn portable_survive_crash() {
    let data = include_bytes!("../assets/portable-crash-1");
    let hash = PortableHash::new(Key([1, 2, 3, 4])).hash64(&data[..]);
    assert!(hash != 0);
}

#[test]
#[cfg(target_arch = "x86_64")]
fn avx_survive_crash() {
    use highway::AvxHash;
    if !is_x86_feature_detected!("avx2") {
        return;
    }

    let data = include_bytes!("../assets/avx-crash-1");
    let hash = unsafe { AvxHash::force_new(Key([1, 2, 3, 4])) }.hash64(&data[..]);
    assert!(hash != 0);
}

#[test]
fn builder_hash_eq_portable() {
    use highway::HighwayHasher;

    let data: Vec<u8> = (0..100).map(|x| x as u8).collect();
    let key = Key([
        0x0706_0504_0302_0100,
        0x0F0E_0D0C_0B0A_0908,
        0x1716_1514_1312_1110,
        0x1F1E_1D1C_1B1A_1918,
    ]);

    for i in 0..100 {
        println!("{}", i);
        assert_eq!(
            PortableHash::new(key).hash64(&data[..i]),
            HighwayHasher::new(key).hash64(&data[..i])
        );

        assert_eq!(
            PortableHash::new(key).hash128(&data[..i]),
            HighwayHasher::new(key).hash128(&data[..i])
        );

        assert_eq!(
            PortableHash::new(key).hash256(&data[..i]),
            HighwayHasher::new(key).hash256(&data[..i])
        );
    }
}
