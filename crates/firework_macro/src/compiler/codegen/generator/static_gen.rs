// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

/// Хелпер для декларации статического экземпляра структуры экрана (слайда), заполняет
/// все поля как None, то есть требует чтобы все поля структуры были строго Option. Это
/// не проблема так как компилятор раста не скомпилирует код который использует переменные
/// до инициализации
pub(crate) fn static_declaration(instance_name: &str, struct_name: &str, fields: &[(String, String)]) -> String {
    let mut output = String::new();
   
    // Нужно генерировать код только если у структуры есть поля, их отсуствие невозможно,
    // но для надёжности это имеет смысл
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

/// Хелпер для создания переменной которая хранит указатель на текущую
/// функцию чтобы сравнивать его и установить в диспетчере
pub(crate) fn is_first_call(function_name: &str) -> String {
    format!(
        "\tlet _fwc_id: fn() = {};\n",
        function_name,
    )
}

/// Хелпер для инлайна инициализации поля _fwc_screen_id через firework::register, так как
/// новая архитектура хранит только указатель на функцию, а не контейнер и индексы, то нужно
/// использовать заглушку (Some(1)) чтобы не переписывать много кода
pub(crate) fn init_instance(instance_name: &str, screen_name: &str) -> String {
    format!(
        "\tif _fwc_id == firework::get_focus() {{\n\t\t_fwc_build = true;\n\t\tunsafe {{\n\t\t\t{}_INSTANCE._fwc_screen_id = Some(1);\n\t\t}}\n\t}}\n\n",
        instance_name,
    )
}

/// Хелпер для инлайна получения ссылки на экземпляр, позволяет читать значения через эту ссылку
pub(crate) fn block_ref(instance_name: &str) -> String {
    format!("\tlet _fwc_block = unsafe {{ &{}_INSTANCE }};\n\n", instance_name)
}

/// Хелпер который позволяет установить значение поля экземпляра экрана (слайда). Важно, метод
/// считает что все поля в экземпляре это Option поэтому автоматически задае́т
/// им значение как Some( ... ) где "..." это ввод
pub(crate) fn set_field(instance_name: &str, field_name: &str, value: &str) -> String {
    format!(
        "\tunsafe {{ {}_INSTANCE.{} = Some({}) }};\n",
        instance_name, field_name, value,
    )
}
