// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

pub(crate) fn set_flag(mask_name: &str, flag: u8) -> String {
    assert!(flag < 64, "Flag bit index must be < 64");
    return format!("{} |= 1 << {}", mask_name, flag);
}

pub(crate) fn check_flag(mask_name: &str, flag: u8) -> String {
    assert!(flag < 64, "Flag bit index must be < 64");
    return format!("({} & (1 << {})) != 0", mask_name, flag);
}

pub(crate) fn clear_mask(mask_name: &str) -> String {
    return format!("{} = 0", mask_name);
}

pub(crate) fn normalize_bit_index(id: usize) -> u8 {
    // SAFETY: Формула написана так что в любом случае будет значение от 0 до 63, а
    // ограчение u8 это 256, то есть даже при максимальном значении переполнение
    // математически невозможно
    (id % 64).try_into().unwrap()
}

pub(crate) fn get_spark_mask(id: usize) -> u8 {
    // 1 -> 1, 19 -> 1, 64 -> 1, 67 -> 2, 98 -> 2, 128 -> 2, 136 -> 3
    ((id + 63) / 64) as u8
}
