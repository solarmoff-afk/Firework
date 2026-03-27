// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

/// Хелпер для декларации статического экземпляра структуры экрана (слайда), заполняет
/// все поля как None, то есть требует чтобы все поля структуры были строго Option. Это
/// не проблема так как компилятор раста не скомпилирует код который использует переменные
/// до инициализации
pub fn static_declaration(instance_name: &str, struct_name: &str, fields: &[(String, String)]) -> String {
    let mut output = String::new();
    
    if fields.len() > 0 {
        output.push_str(format!(
            "static mut {}_INSTANCE: {} = {} {{\n",
            instance_name, struct_name, struct_name,
        ).as_str());
        
        for (field_name, _) in fields {
            output.push_str(format!("\t{}: None,\n", field_name).as_str());
        }
        
        output.push_str("\t_fwc_screen_id: None,\n");
        output.push_str("};\n\n");
    }
    
    output
}

/// Хелпер для инлайна проверки на первый вызов. Проверяет маркерное поле _fwc_screen_id
/// на то, было ли оно инициализировано (some) в статичном экземпляре
pub fn is_first_call(instance_name: &str) -> String {
    format!(
        "\tlet _fwc_is_first_call = unsafe {{ {}_INSTANCE._fwc_screen_id.is_none() }};\n",
        instance_name
    )
}

/// Хелпер для инлайна инициализации поля _fwc_screen_id через firework::register, так как
/// функция экрана регистрирует сама себя в системе навигации диспетчера
pub fn init_instance(instance_name: &str, screen_name: &str) -> String {
    format!(
        "\tif _fwc_is_first_call {{\n\t\tunsafe {{\n\t\t\t{}_INSTANCE._fwc_screen_id = Some(firework::register({}));\n\t\t}}\n\t}}\n\n",
        instance_name, screen_name
    )
}

/// Хелпер для инлайна получения ссылки на экземпляр, позволяет читать значения через эту ссылку
pub fn block_ref(instance_name: &str) -> String {
    format!("\tlet _fwc_block = unsafe {{ &{}_INSTANCE }};\n\n", instance_name)
}

/// Хелпер который позволяет установить значение поля экземпляра экрана (слайда)
pub fn set_field(instance_name: &str, field_name: &str, value: &str) -> String {
    format!(
        "\tunsafe {{ {}_INSTANCE.{} = {} }};\n",
        instance_name, field_name, value
    )
}
