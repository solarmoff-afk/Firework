// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

#![allow(dead_code)]

use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;

/// Хелпер для декларации статического экземпляра структуры экрана (слайда), заполняет
/// все поля как None, то есть требует чтобы все поля структуры были строго Option. Это
/// не проблема так как компилятор раста не скомпилирует код который использует переменные
/// до инициализации
#[cfg(not(feature = "safety"))]
pub(crate) fn static_declaration(
    instance_name: &str,
    struct_name: &str,
    fields: &[(String, String)],
) -> String {
    let mut output = String::new();
    // Нужно генерировать код только если у структуры есть поля, их отсуствие невозможно,
    // но для надёжности это имеет смысл
    if !fields.is_empty() {
        output.push_str(
            format!(
                "static mut {}_INSTANCE: {} = {} {{\n",
                instance_name, struct_name, struct_name,
            )
            .as_str(),
        );
        for (field_name, _) in fields {
            output.push_str(format!("\t{}: None,\n", field_name).as_str());
        }
        output.push_str("};\n");
    }
    output
}

/// Для безопасного режима (однопоточный, thread_local)
#[cfg(feature = "safety")]
pub(crate) fn static_declaration(
    instance_name: &str,
    struct_name: &str,
    fields: &[(String, String)],
) -> String {
    let mut output = String::new();
    if fields.len() > 0 {
        output.push_str(format!(
            "std::thread_local! {{\n\tstatic {}_INSTANCE: std::cell::RefCell<{}> = std::cell::RefCell::new({} {{\n",
            instance_name, struct_name, struct_name,
        ).as_str());
        for (field_name, _) in fields {
            output.push_str(format!("\t\t{}: None,\n", field_name).as_str());
        }
        output.push_str("\t});\n}\n");
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
#[cfg(not(feature = "safety"))]
pub(crate) fn init_instance(
    instance_name: &str,
    _screen_name: &str,
    _fields: &[(String, String)],
) -> String {
    format!(
        "\tif unsafe {{ {}_INSTANCE._fwc__fwc_screen_id.is_none() }} {{\n\t\t_fwc_build = true;\n\t\tunsafe {{\n\t\t\t{}_INSTANCE._fwc__fwc_screen_id = Some(1);\n\t\t}}\n\t}}\n",
        instance_name, instance_name,
    )
}

#[cfg(feature = "safety")]
pub(crate) fn init_instance(
    instance_name: &str,
    _struct_name: &str,
    _fields: &[(String, String)],
) -> String {
    let mut output = String::new();
    output.push_str(
        format!(
            "\t{}_INSTANCE.with(|inst| {{\n\t\tlet mut instance = inst.borrow_mut();\n",
            instance_name,
        )
        .as_str(),
    );
    output.push_str("\t\tif instance._fwc__fwc_screen_id.is_none() {\n");
    output.push_str("\t\t\t_fwc_build = true;\n");
    output.push_str("\t\t\tinstance._fwc__fwc_screen_id = Some(1);\n");
    output.push_str("\t\t}\n");
    output.push_str("\t});\n");
    output
}

#[cfg(not(feature = "safety"))]
pub(crate) fn init_instance_tokens(
    instance_name: &str,
    _struct_name: &str,
    _fields: &[(String, String)],
) -> TokenStream {
    let instance_ident = format_ident!("{}_INSTANCE", instance_name);
    quote! {
        if unsafe { #instance_ident._fwc__fwc_screen_id.is_none() } {
            _fwc_build = true;
            unsafe {
                #instance_ident._fwc__fwc_screen_id = Some(1);
            }
        }
    }
}

#[cfg(feature = "safety")]
pub(crate) fn init_instance_tokens(
    instance_name: &str,
    _struct_name: &str,
    _fields: &[(String, String)],
) -> TokenStream {
    let instance_ident = format_ident!("{}_INSTANCE", instance_name);
    quote! {
        #instance_ident.with(|inst| {
            let mut instance = inst.borrow_mut();
            if instance._fwc__fwc_screen_id.is_none() {
                _fwc_build = true;
                instance._fwc__fwc_screen_id = Some(1);
            }
        });
    }
}

/// Хелпер который позволяет установить значение поля экземпляра экрана (слайда). Важно, метод
/// считает что все поля в экземпляре это Option поэтому автоматически задае́т
/// им значение как Some( ... ) где "..." это ввод
#[cfg(not(feature = "safety"))]
pub(crate) fn set_field(instance_name: &str, field_name: &str, value: &str) -> String {
    // Статический экземпляр имеет имя в верхнем регистре поэтому для правильной генерации
    // нужно возвести имя структуры в верхний регистр
    let instance_name_upper = instance_name.to_uppercase();
    format!(
        "unsafe {{ (*::core::ptr::addr_of_mut!({}_INSTANCE)).{} = Some({}) }};\n",
        instance_name_upper, field_name, value,
    )
}

#[cfg(feature = "safety")]
pub(crate) fn set_field(instance_name: &str, field_name: &str, value: &str) -> String {
    let instance_name_upper = instance_name.to_uppercase();
    format!(
        "\t{}_INSTANCE.with(|inst| {{ inst.borrow_mut().{} = Some({}); }});\n",
        instance_name_upper, field_name, value,
    )
}

/// Хелпер для получения значения поля с забиранием владения (take) для Option полей
/// структуры экрана/компонента
#[cfg(not(feature = "safety"))]
pub(crate) fn take_field(instance_name: &str, field_name: &str) -> String {
    let instance_name_upper = instance_name.to_uppercase();
    format!(
        "unsafe {{ (*::core::ptr::addr_of_mut!({}_INSTANCE)).{}.take().unwrap() }}",
        instance_name_upper, field_name,
    )
}

#[cfg(feature = "safety")]
pub(crate) fn take_field(instance_name: &str, field_name: &str) -> String {
    let instance_name_upper = instance_name.to_uppercase();
    format!(
        "{}_INSTANCE.with(|inst| inst.borrow_mut().{}.take().unwrap())",
        instance_name_upper, field_name,
    )
}

#[cfg(not(feature = "safety"))]
pub(crate) fn block_ref(instance_name: &str) -> String {
    format!(
        "\tlet _fwc_block = unsafe {{ &{}_INSTANCE }};\n",
        instance_name
    )
}

#[cfg(feature = "safety")]
pub(crate) fn block_ref(instance_name: &str) -> String {
    format!(
        "\tlet _fwc_block = {}_INSTANCE.with(|inst| inst.borrow());\n",
        instance_name,
    )
}

/// Хелпер для получения неизменяемой ссылки на значение поля структуры шейреда
#[cfg(not(feature = "safety"))]
pub(crate) fn get_field_ref(instance_name: &str, field_name: &str, var_name: &str) -> String {
    let instance_name_upper = instance_name.to_uppercase();
    format!(
        "let {} = unsafe {{ (*::core::ptr::addr_of!({}_INSTANCE)).{}.as_ref().unwrap() }};",
        var_name, instance_name_upper, field_name,
    )
}

#[cfg(feature = "safety")]
pub(crate) fn get_field_ref(instance_name: &str, field_name: &str, var_name: &str) -> String {
    let instance_name_upper = instance_name.to_uppercase();
    format!(
        "let {} = {}_INSTANCE.with(|inst| inst.borrow().{}.as_ref().unwrap().clone());",
        var_name, instance_name_upper, field_name,
    )
}

/// Хелпер для получения изменяемой ссылки на значение поля структуры шейдера
#[cfg(not(feature = "safety"))]
pub(crate) fn get_field_ref_mut(instance_name: &str, field_name: &str, var_name: &str) -> String {
    let instance_name_upper = instance_name.to_uppercase();
    format!(
        "let mut {} = unsafe {{ (*::core::ptr::addr_of_mut!({}_INSTANCE)).{}.as_mut().unwrap() }};",
        var_name, instance_name_upper, field_name,
    )
}

#[cfg(feature = "safety")]
pub(crate) fn get_field_ref_mut(instance_name: &str, field_name: &str, var_name: &str) -> String {
    let instance_name_upper = instance_name.to_uppercase();
    format!(
        "let mut {} = {}_INSTANCE.with(|inst| inst.borrow_mut().{}.as_mut().unwrap());",
        var_name, instance_name_upper, field_name,
    )
}

/// Принимает имя структуры, поле и имя переменной после чего генерирует код который
/// записывает в эту переменную копию, работает только для типов которые реализуют Copy
#[cfg(not(feature = "safety"))]
pub(crate) fn copy_field(instance_name: &str, field_name: &str, var_name: &str) -> String {
    let instance_name_upper = instance_name.to_uppercase();
    format!(
        "if let Some(val) = unsafe {{ (*::core::ptr::addr_of!({}_INSTANCE)).{}.as_ref() }} {{ {} = *val; }}",
        instance_name_upper, field_name, var_name,
    )
}

#[cfg(feature = "safety")]
pub(crate) fn copy_field(instance_name: &str, field_name: &str, var_name: &str) -> String {
    let instance_name_upper = instance_name.to_uppercase();
    format!(
        "if let Some(val) = {}_INSTANCE.with(|inst| inst.borrow().{}.as_ref()) {{ {} = *val; }}",
        instance_name_upper, field_name, var_name,
    )
}

#[cfg(not(feature = "safety"))]
pub(crate) fn copy_cell_field(instance_name: &str, field_name: &str, var_name: &str) -> String {
    let instance_name_upper = instance_name.to_uppercase();
    format!(
        "if let Some(val) = unsafe {{ (*::core::ptr::addr_of!({}_INSTANCE)).{}.as_ref() }} {{ {}.set(*val); }}",
        instance_name_upper, field_name, var_name,
    )
}

#[cfg(feature = "safety")]
pub(crate) fn copy_cell_field(instance_name: &str, field_name: &str, var_name: &str) -> String {
    let instance_name_upper = instance_name.to_uppercase();
    format!(
        "if let Some(val) = {}_INSTANCE.with(|inst| inst.borrow().{}.as_ref()) {{ {}.set(*val); }}",
        instance_name_upper, field_name, var_name,
    )
}
