// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

pub(crate) fn set_flag(mask_name: &str, flag: u8) -> String {
    assert!(flag < 128, "Flag bit index must be < 128");
    return format!("{} |= 1 << {}", mask_name, flag);
}

pub(crate) fn check_flag(mask_name: &str, flag: u8) -> String {
    assert!(flag < 128, "Flag bit index must be < 128");
    return format!("({} & (1 << {})) != 0", mask_name, flag);
}

pub(crate) fn clear_mask(mask_name: &str) -> String {
    return format!("{} = 0", mask_name);
}