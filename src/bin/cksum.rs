use std::env;
use std::fs::File;
use std::io::Read;

fn main() -> Result<(), ()> {
    let mut args = env::args();
    if args.len() > 1 {
        checksum_files(&mut args)
    } else {
        let mut buffer: Vec<u8> = Vec::new();
        match std::io::stdin().lock().read_to_end(&mut buffer) {
            Ok(length) => println!("{} {}", compute_crc32(&buffer), length),
            Err(error) => {
                eprintln!("cksum: stdin: {}", error);
                return Err(());
            }
        }
    }
}

fn checksum_files(file_names: &mut _) -> Result<()> {
    let mut has_bad_file = false;
    let mut next_arg = file_names.nth(1);
    while let Some(file_name) = next_arg {
        let buffer_result = read_file(&file_name);
        match buffer_result {
            Ok(buffer) => println!(
                "{} {} {}",
                compute_crc32(&buffer),
                buffer.len(),
                file_name
            ),
            Err(error) => {
                eprintln!("cksum: {}: {}", file_name, error);
                has_bad_file = true;
            }
        }
        next_arg = file_names.next();
    }

    if !has_bad_file {
        Ok(())
    } else {
        Err(())
    }
}

fn read_file(path: &str) -> std::io::Result<Vec<u8>> {
    let mut file = File::open(path)?;

    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    Ok(data)
}

fn compute_crc32(buffer: &[u8]) -> u32 {
    /* A mostly direct translation from the algorithm written in C found
     * here: https://github.com/wertarbyte/coreutils/blob/master/src/cksum.c */
    let crc_table: [u32; 256] = [
        0x0000_0000,
        0x04c1_1db7, 0x0982_3b6e, 0x0d43_26d9, 0x1304_76dc, 0x17c5_6b6b,
        0x1a86_4db2, 0x1e47_5005, 0x2608_edb8, 0x22c9_f00f, 0x2f8a_d6d6,
        0x2b4b_cb61, 0x350c_9b64, 0x31cd_86d3, 0x3c8e_a00a, 0x384f_bdbd,
        0x4c11_db70, 0x48d0_c6c7, 0x4593_e01e, 0x4152_fda9, 0x5f15_adac,
        0x5bd4_b01b, 0x5697_96c2, 0x5256_8b75, 0x6a19_36c8, 0x6ed8_2b7f,
        0x639b_0da6, 0x675a_1011, 0x791d_4014, 0x7ddc_5da3, 0x709f_7b7a,
        0x745e_66cd, 0x9823_b6e0, 0x9ce2_ab57, 0x91a1_8d8e, 0x9560_9039,
        0x8b27_c03c, 0x8fe6_dd8b, 0x82a5_fb52, 0x8664_e6e5, 0xbe2b_5b58,
        0xbaea_46ef, 0xb7a9_6036, 0xb368_7d81, 0xad2f_2d84, 0xa9ee_3033,
        0xa4ad_16ea, 0xa06c_0b5d, 0xd432_6d90, 0xd0f3_7027, 0xddb0_56fe,
        0xd971_4b49, 0xc736_1b4c, 0xc3f7_06fb, 0xceb4_2022, 0xca75_3d95,
        0xf23a_8028, 0xf6fb_9d9f, 0xfbb8_bb46, 0xff79_a6f1, 0xe13e_f6f4,
        0xe5ff_eb43, 0xe8bc_cd9a, 0xec7d_d02d, 0x3486_7077, 0x3047_6dc0,
        0x3d04_4b19, 0x39c5_56ae, 0x2782_06ab, 0x2343_1b1c, 0x2e00_3dc5,
        0x2ac1_2072, 0x128e_9dcf, 0x164f_8078, 0x1b0c_a6a1, 0x1fcd_bb16,
        0x018a_eb13, 0x054b_f6a4, 0x0808_d07d, 0x0cc9_cdca, 0x7897_ab07,
        0x7c56_b6b0, 0x7115_9069, 0x75d4_8dde, 0x6b93_dddb, 0x6f52_c06c,
        0x6211_e6b5, 0x66d0_fb02, 0x5e9f_46bf, 0x5a5e_5b08, 0x571d_7dd1,
        0x53dc_6066, 0x4d9b_3063, 0x495a_2dd4, 0x4419_0b0d, 0x40d8_16ba,
        0xaca5_c697, 0xa864_db20, 0xa527_fdf9, 0xa1e6_e04e, 0xbfa1_b04b,
        0xbb60_adfc, 0xb623_8b25, 0xb2e2_9692, 0x8aad_2b2f, 0x8e6c_3698,
        0x832f_1041, 0x87ee_0df6, 0x99a9_5df3, 0x9d68_4044, 0x902b_669d,
        0x94ea_7b2a, 0xe0b4_1de7, 0xe475_0050, 0xe936_2689, 0xedf7_3b3e,
        0xf3b0_6b3b, 0xf771_768c, 0xfa32_5055, 0xfef3_4de2, 0xc6bc_f05f,
        0xc27d_ede8, 0xcf3e_cb31, 0xcbff_d686, 0xd5b8_8683, 0xd179_9b34,
        0xdc3a_bded, 0xd8fb_a05a, 0x690c_e0ee, 0x6dcd_fd59, 0x608e_db80,
        0x644f_c637, 0x7a08_9632, 0x7ec9_8b85, 0x738a_ad5c, 0x774b_b0eb,
        0x4f04_0d56, 0x4bc5_10e1, 0x4686_3638, 0x4247_2b8f, 0x5c00_7b8a,
        0x58c1_663d, 0x5582_40e4, 0x5143_5d53, 0x251d_3b9e, 0x21dc_2629,
        0x2c9f_00f0, 0x285e_1d47, 0x3619_4d42, 0x32d8_50f5, 0x3f9b_762c,
        0x3b5a_6b9b, 0x0315_d626, 0x07d4_cb91, 0x0a97_ed48, 0x0e56_f0ff,
        0x1011_a0fa, 0x14d0_bd4d, 0x1993_9b94, 0x1d52_8623, 0xf12f_560e,
        0xf5ee_4bb9, 0xf8ad_6d60, 0xfc6c_70d7, 0xe22b_20d2, 0xe6ea_3d65,
        0xeba9_1bbc, 0xef68_060b, 0xd727_bbb6, 0xd3e6_a601, 0xdea5_80d8,
        0xda64_9d6f, 0xc423_cd6a, 0xc0e2_d0dd, 0xcda1_f604, 0xc960_ebb3,
        0xbd3e_8d7e, 0xb9ff_90c9, 0xb4bc_b610, 0xb07d_aba7, 0xae3a_fba2,
        0xaafb_e615, 0xa7b8_c0cc, 0xa379_dd7b, 0x9b36_60c6, 0x9ff7_7d71,
        0x92b4_5ba8, 0x9675_461f, 0x8832_161a, 0x8cf3_0bad, 0x81b0_2d74,
        0x8571_30c3, 0x5d8a_9099, 0x594b_8d2e, 0x5408_abf7, 0x50c9_b640,
        0x4e8e_e645, 0x4a4f_fbf2, 0x470c_dd2b, 0x43cd_c09c, 0x7b82_7d21,
        0x7f43_6096, 0x7200_464f, 0x76c1_5bf8, 0x6886_0bfd, 0x6c47_164a,
        0x6104_3093, 0x65c5_2d24, 0x119b_4be9, 0x155a_565e, 0x1819_7087,
        0x1cd8_6d30, 0x029f_3d35, 0x065e_2082, 0x0b1d_065b, 0x0fdc_1bec,
        0x3793_a651, 0x3352_bbe6, 0x3e11_9d3f, 0x3ad0_8088, 0x2497_d08d,
        0x2056_cd3a, 0x2d15_ebe3, 0x29d4_f654, 0xc5a9_2679, 0xc168_3bce,
        0xcc2b_1d17, 0xc8ea_00a0, 0xd6ad_50a5, 0xd26c_4d12, 0xdf2f_6bcb,
        0xdbee_767c, 0xe3a1_cbc1, 0xe760_d676, 0xea23_f0af, 0xeee2_ed18,
        0xf0a5_bd1d, 0xf464_a0aa, 0xf927_8673, 0xfde6_9bc4, 0x89b8_fd09,
        0x8d79_e0be, 0x803a_c667, 0x84fb_dbd0, 0x9abc_8bd5, 0x9e7d_9662,
        0x933e_b0bb, 0x97ff_ad0c, 0xafb0_10b1, 0xab71_0d06, 0xa632_2bdf,
        0xa2f3_3668, 0xbcb4_666d, 0xb875_7bda, 0xb536_5d03, 0xb1f7_40b4
    ];


    let mut sum: u32 = 0;
    for octet in buffer {
        sum = (sum << 8) ^ crc_table[(((sum >> 24) ^ *octet as u32) & 0xFF) as usize];
    }

    let mut buffer_length: u32 = buffer.len() as u32;
    while buffer_length != 0 {
        sum = (sum << 8) ^ crc_table[(((sum >> 24) ^ buffer_length) & 0xFF) as usize];
        buffer_length >>= 8;
    }

    !sum
}
