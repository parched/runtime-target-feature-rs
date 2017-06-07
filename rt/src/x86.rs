pub fn have_avx() -> bool {
    test_bit(1, 28)
}

pub fn have_avx2() -> bool {
    test_bit(2, 5)
}

pub fn have_bmi() -> bool {
    test_bit(2, 3)
}

pub fn have_bmi2() -> bool {
    test_bit(2, 8)
}

pub fn have_sse() -> bool {
    test_bit(0, 25)
}

pub fn have_sse2() -> bool {
    test_bit(0, 26)
}

pub fn have_sse3() -> bool {
    test_bit(1, 0)
}

pub fn have_sse4_1() -> bool {
    test_bit(1, 19)
}

pub fn have_sse4_2() -> bool {
    test_bit(1, 20)
}

pub fn have_ssse3() -> bool {
    test_bit(1, 9)
}

pub fn have_tbm() -> bool {
    test_bit(6, 21)
}

pub fn have_lzcnt() -> bool {
    test_bit(6, 55)
}

pub fn have_popcnt() -> bool {
    test_bit(1, 23)
}

pub fn have_sse4a() -> bool {
    test_bit(6, 6)
}

pub fn have_rdrnd() -> bool {
    test_bit(1, 30)
}

pub fn have_rdseed() -> bool {
    test_bit(2, 18)
}

pub fn have_fma() -> bool {
    test_bit(1, 12)
}

lazy_static! { static ref FEATURES: [u32; 7] = unsafe {
    let highest_cpuid: u32;
    let features_0: u32;
    let features_1: u32;
    let features_2: u32;
    let features_3: u32;
    let features_4: u32;

    let extended_highest_cpuid: u32;
    let features_5: u32;
    let features_6: u32;

    asm!("cpuid" : "={eax}"(highest_cpuid) : "{eax}"(0) : "ebx", "ecx", "edx");

    if highest_cpuid >= 1 {
        asm!("cpuid" : "={ecx}"(features_1), "={edx}"(features_0) : "{eax}"(1) : "eax", "ebx");
        if highest_cpuid >= 7 {
            asm!("cpuid":
                 "={ebx}"(features_2), "={ecx}"(features_3), "={edx}"(features_4):
                 "{eax}"(7), "{ecx}"(0):
                 "eax");
        } else {
            features_2 = 0;
            features_3 = 0;
            features_4 = 0;
        }

    } else {
        features_0 = 0;
        features_1 = 0;
        features_2 = 0;
        features_3 = 0;
        features_4 = 0;
    }

    asm!("cpuid" : "={eax}"(extended_highest_cpuid) : "{eax}"(0) : "ebx", "ecx", "edx");

    if extended_highest_cpuid >= 0x80000001u32 {
        asm!("cpuid":
             "={ecx}"(features_6), "={edx}"(features_5):
             "{eax}"(0x80000001u32):
             "eax", "ebx");
    } else {
        features_5 = 0;
        features_6 = 0;
    }

    [features_0, features_1, features_2, features_3, features_4, features_5, features_6]
};}

fn test_bit(word: usize, bit: usize) -> bool {
    FEATURES[word] & (1u32 << bit) != 0u32
}
