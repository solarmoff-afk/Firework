// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

#![allow(dead_code)]

/// Хелпер для декларации статического экземпляра структуры экрана (слайда), заполняет
/// все поля как None, то есть требует чтобы все поля структуры были строго Option. Это
/// не проблема так как компилятор раста не скомпилирует код который использует переменные
/// до инициализации
#[cfg(not(feature = "safety-multithread"))]
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

#[cfg(feature = "safety-multithread")]
pub(crate) fn static_declaration(instance_name: &str, struct_name: &str, fields: &[(String, String)]) -> String {
    let mut output = String::new();
    
    if fields.len() > 0 {
        output.push_str(format!(
            "static {}_INSTANCE: std::sync::OnceLock<std::sync::Mutex<{}>> = std::sync::OnceLock::new();\n\n",
            instance_name, struct_name,
        ).as_str());
    }
    
    output
}

/// Хелпер для создания переменной которая хранит указатель на текущую
/// функцию чтобы сравнивать его и установить в диспетчере
pub(crate) fn is_first_call(id: u128) -> String {
    format!("\tlet _fwc_id: u128 = {};\n", id)
}

/// Хелпер для инлайна инициализации поля _fwc_screen_id через firework::register, так как
/// новая архитектура хранит только указатель на функцию, а не контейнер и индексы, то нужно
/// использовать заглушку (Some(1)) чтобы не переписывать много кода
#[cfg(not(feature = "safety-multithread"))]
pub(crate) fn init_instance(instance_name: &str, _screen_name: &str, _fields: &[(String, String)]) -> String {
    format!(
        "\tif unsafe {{ {}_INSTANCE._fwc_screen_id.is_none() }} {{\n\t\t_fwc_build = true;\n\t\tunsafe {{\n\t\t\t{}_INSTANCE._fwc_screen_id = Some(1);\n\t\t}}\n\t}}\n\n",
        instance_name, instance_name,
    )
}

#[cfg(feature = "safety-multithread")]
pub(crate) fn init_instance(instance_name: &str, struct_name: &str, fields: &[(String, String)]) -> String {
    let mut output = String::new();
    
    output.push_str(format!(
        "\tlet mut instance = {}_INSTANCE.get_or_init(|| std::sync::Mutex::new({} {{\n",
        instance_name, struct_name,
    ).as_str());
    
    for (field_name, _) in fields {
        output.push_str(format!("\t\t{}: None,\n", field_name).as_str());
    }
    
    output.push_str("\t\t_fwc_screen_id: None,\n");
    output.push_str("\t})).lock().unwrap();\n");
    
    output.push_str("\tif instance._fwc_screen_id.is_none() {\n");
    output.push_str("\t\t_fwc_build = true;\n");
    output.push_str("\t\tinstance._fwc_screen_id = Some(1);\n");
    output.push_str("\t}\n");
    output.push_str("\tdrop(instance);\n\n");
    
    output
}

/// Хелпер который позволяет установить значение поля экземпляра экрана (слайда). Важно, метод
/// считает что все поля в экземпляре это Option поэтому автоматически задае́т
/// им значение как Some( ... ) где "..." это ввод
#[cfg(not(feature = "safety-multithread"))]
pub(crate) fn set_field(instance_name: &str, field_name: &str, value: &str) -> String {
    // Статический экземпляр имеет имя в верхнем регистре поэтому для правильной генерации
    // нужно возвести имя структуры в верхний регистр
    let instance_name_upper = instance_name.to_uppercase();

    format!(
        "unsafe {{ (*::core::ptr::addr_of_mut!({}_INSTANCE)).{} = Some({}) }};\n",
        instance_name_upper, field_name, value,
    )
}

#[cfg(feature = "safety-multithread")]
pub(crate) fn set_field(instance_name: &str, field_name: &str, value: &str) -> String {
    let instance_name_upper = instance_name.to_uppercase();
    
    format!(
        "\t{}_INSTANCE.get().unwrap().lock().unwrap().{} = Some({});\n",
        instance_name_upper, field_name, value,
    )
}

/// Хелпер для получения значения поля с забиранием владения (take) для Option полей
/// структуры экрана/компонента
#[cfg(not(feature = "safety-multithread"))]
pub(crate) fn take_field(instance_name: &str, field_name: &str) -> String {
    let instance_name_upper = instance_name.to_uppercase();
    
    format!(
        "unsafe {{ (*::core::ptr::addr_of_mut!({}_INSTANCE)).{}.take().unwrap() }}",
        instance_name_upper, field_name,
    )
}

#[cfg(feature = "safety-multithread")]
pub(crate) fn take_field(instance_name: &str, field_name: &str) -> String {
    let instance_name_upper = instance_name.to_uppercase();
    
    format!(
        "{}_INSTANCE.get().unwrap().lock().unwrap().{}.take().unwrap()",
        instance_name_upper, field_name,
    )
}

#[cfg(not(feature = "safety-multithread"))]
pub(crate) fn block_ref(instance_name: &str) -> String {
    format!("\tlet _fwc_block = unsafe {{ &{}_INSTANCE }};\n\n", instance_name)
}

#[cfg(feature = "safety-multithread")]
pub(crate) fn block_ref(instance_name: &str) -> String {
    format!(
        "\tlet _fwc_block = {}_INSTANCE.get().unwrap().lock().unwrap();\n\n",
        instance_name,
    )
}
