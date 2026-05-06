// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use proc_macro2::{Group, Span};
use quote::quote;
use quote::quote_spanned;
use syn::parse_quote;

pub use super::super::*;

use crate::CompileType;
use crate::compiler::codegen::generator::static_gen;
use crate::compiler::codegen::transform::visitors_mut::self_visitor::SelfFieldAdder;

impl CodegenVisitor<'_> {
    /// Обрабатывает верхний уровень в вызове компилятора (item), функции, структуры и так
    /// далее. Генерирует flash pass и реактивный цикл
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub(crate) fn analyze_file_mut(&mut self, i: &mut File) {
        let mut new_items = Vec::new();

        // Забираем элементы, чтобы не клонировать весь вектор сразу
        let items = std::mem::take(&mut i.items);

        for item in items {
            // Любая функция это экран
            match item {
                Item::Fn(mut item_fn) => {
                    self.transform_ui_function(
                        &mut item_fn.sig,
                        &mut item_fn.block,
                        &mut new_items,
                    );

                    new_items.push(Item::Fn(item_fn));
                }

                Item::Struct(mut item_struct) => {
                    self.codegen_item_struct(&mut item_struct);
                    new_items.push(Item::Struct(item_struct));
                }

                Item::Impl(mut item_impl) => {
                    if let Type::Path(type_path) = &*item_impl.self_ty
                        && let Some(segment) = type_path.path.segments.last()
                    {
                        let struct_name = segment.ident.to_string();
                        self.extend_new(&mut item_impl, &struct_name);
                    }

                    for item in &mut item_impl.items {
                        if let ImplItem::Fn(method) = item
                            && method.sig.ident == "flash"
                        {
                            self.transform_ui_function(
                                &mut method.sig,
                                &mut method.block,
                                &mut new_items,
                            );
                        }
                    }
                    new_items.push(Item::Impl(item_impl));
                }

                mut other_item => {
                    self.visit_item_mut(&mut other_item);
                    new_items.push(other_item);
                }
            }
        }

        self.resolve_shared_desugar_attr(&mut new_items);

        i.items = new_items;
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all, fields(sig = ?sig, block = %quote!(#block))))]
    fn transform_ui_function(
        &mut self,
        sig: &mut Signature,
        block: &mut Block,
        new_items: &mut Vec<Item>,
    ) {
        // Возвращает ли что-то функция, это нужно чтобы понять нужно ли сгенерировать
        // панику в конце цикла чтобы избежать ошибки
        let has_return = self.check_function_has_return(sig);

        self.functions_count += 1;
        let function_name = sig.ident.to_string();

        // Поиск имени функции в IR, если оно не найдено от find вернёт None в self.ui_id
        // и код для UI не сгенерируется
        self.ui_id = self
            .ir
            .screens
            .iter()
            .find(|(name, _, _)| name == &function_name)
            .map(|(_, _, id)| *id);

        self.visit_block_mut(block);

        if let Some(id) = self.ui_id {
            let span = sig.span();

            let struct_name_raw = format!("ApplicationUiBlockStruct{}", id);
            let _instance_ident = format_ident!("APPLICATIONUIBLOCKSTRUCT{}_INSTANCE", id);
            let build_name = format_ident!("_fwc_fn_build{}", id);

            let mut fields: Vec<Field> = Vec::new();
            let fields_data = self.generate_fields(id, &mut fields, span);

            // Генерация статического экземпляра. Если используется safety-multitrhead
            // фича то static_gen генерирует OnceLock + Mutex для безопасной работы
            // из нескольких поток, если safety-multitrhead нет то используется
            // static mut и unsafe
            let instance_name = struct_name_raw.to_uppercase();
            let instance = static_declaration(&instance_name, &struct_name_raw, &fields_data);
            let instance_item: Item = parse_str(&instance).expect("IE:8");

            self.generate_build(new_items, instance_item, &fields, id, span);

            // Оригинальное тело функции (уже трансформированное), так как block
            // не реализует Default нужно использовать std::mem::replace, идёт
            // парсинг обычного пустого блока чтобы заменить на него оригинал, а
            // оригинальые данные забрать сюда чтобы избежать клонирования
            let mut original_block = std::mem::replace(block, parse_quote!({}));

            let reactive_output = self.generate_reactive(id);
            let generated_block = self.generate_flash_pass(id, &function_name);

            let bitmask_statements = reactive_output.bitmask_statements;
            let bitmask_clone_statements = reactive_output.bitmask_clone_statements;
            let bitmask_check_expr = reactive_output.bitmask_check_expr;
            let widget_bitmask_statement = self.generate_widgets_mask(id, &struct_name_raw);

            let is_shared = matches!(self.flags.compile_type, CompileType::Shared);
            let is_component = matches!(self.flags.compile_type, CompileType::Component);

            let init_code = if !is_shared && !is_component {
                quote! {
                    let mut _fwc_build = false;
                    #generated_block
                }
            } else if !is_component {
                // Если это shared то для каждой функции нужно сначала (в первой фазе)
                // вызвать build функцию чтобы проверить инициализацию и если спарки
                // ещё не инициализированы на уровне state! {} то нужно их
                // инициализировать
                quote! {
                    #build_name();
                }
            } else {
                // Компонентам не нужна инициализация
                quote! {}
            };

            // При входе в item обход дерева продолжился из-за строки выше, конкретно:
            // visit_item_fn_mut(self, &mut item_fn);
            //
            // Она продолжает обход, VisitorMut входит в блок и начинает вызовы
            // кодбилдера для каждого стейтемента который есть в IR снапшоте. В
            // результате билдер формирует пост токены, после вызова visit_item_fn_mut
            // стэк вызовов возвращается в этот метод и токены уже доступны, их
            // можно клонировать и вставить в код
            let post_tokens = self.builder.tokens.clone();

            // Генерация снапшота битовых масок в текущем кадре чтобы в следующем
            // при Event получить снапшот вместо нуля
            let mut widgets_gen_snapshot = TokenStream::new();

            // SAFETY: Для всех id экранов генерируется количество масок, а так как
            // id взят из IR то такой элемент точно есть в карте
            let widget_mask_count = self.widget_mask_count.get(&id).expect("IE:5");

            for mask_index in 0..*widget_mask_count {
                // Имя локальной маски и имя поле идентично
                let field_name = format!("_fwc_widget_bitmask{}", mask_index + 1);
                let set_field_str =
                    static_gen::set_field(&struct_name_raw, &field_name, &field_name);

                widgets_gen_snapshot.extend(CodeBuilder::convert_string_to_syn(&set_field_str));
            }

            {
                #[cfg(feature = "trace")]
                let _span = tracing::warn_span!("final_block_generation", has_return = has_return)
                    .entered();

                let parse_batch = |token_stream: TokenStream| -> Vec<Stmt> {
                    if token_stream.is_empty() {
                        return Vec::new();
                    }

                    syn::parse2::<Block>(quote!({ #token_stream }))
                        .expect("IE: Final Batch Parse Error")
                        .stmts
                };

                let mut final_stmts = Vec::new();

                // Мёртвый код для shared режима, но так как он весь завёрнут в _{name}
                // (с _) то предупреждений не будет, а компилятор раста просто вырежет
                // этот код в релизной сборке как мёртвый
                final_stmts.extend(parse_batch(quote! {
                    let mut _fwc_event = firework_ui::LifeCycle::Navigate;
                    #init_code
                    let mut _fwc_guard: u8 = 0;
                    #(#bitmask_statements)*
                    #(#widget_bitmask_statement)*
                }));

                if !has_return {
                    let mut loop_stmts = Vec::new();

                    loop_stmts.extend(parse_batch(quote! {
                        #(#bitmask_clone_statements)*
                    }));

                    loop_stmts.append(&mut original_block.stmts);

                    // Если цикл совершил более 64 итераций (хардкод )то происходит выход
                    // из него это делается после добавления единицы к итерациям чтобы не
                    // отнимать единицу
                    // (64 - 1 = 63) от максимального количества итераций, так как:
                    //  - Нулевой шаг, +1, 1 итерация
                    //  - Первый шаг,  +1, 2 итерация
                    //  - 63 шаг, +1,  +1, 64 итерация, условие сработало
                    loop_stmts.extend(parse_batch(quote! {
                        // #dyn_lists_end
                        if #bitmask_check_expr { break; }
                        _fwc_guard += 1;
                        _fwc_event = firework_ui::LifeCycle::Reactive;
                        if _fwc_guard > 64 { break; }
                    }));

                    final_stmts.push(syn::Stmt::Expr(
                        syn::Expr::Loop(syn::ExprLoop {
                            attrs: Vec::new(),
                            label: None,
                            loop_token: Default::default(),
                            body: Block {
                                brace_token: Default::default(),
                                stmts: loop_stmts,
                            },
                        }),
                        None,
                    ));
                } else {
                    final_stmts.extend(parse_batch(quote! {
                        #(#bitmask_statements)*
                        #(#bitmask_clone_statements)*
                    }));

                    final_stmts.append(&mut original_block.stmts);
                }

                final_stmts.extend(parse_batch(quote! {
                    // #dyn_lists_end
                    #(#post_tokens)*
                    #widgets_gen_snapshot
                }));

                block.stmts = final_stmts;
            }
        }

        // Очистка локальных данных билдера
        self.builder.function_end();
    }

    /// Генерирует набор полей для вставки по ссылке на fields и возвращает вектор сырых
    /// полей (Имя, тип)
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all, fields(id = ?id, fields = ?fields)))]
    fn generate_fields(
        &self,
        id: u128,
        fields: &mut Vec<Field>,
        span: Span,
    ) -> Vec<(String, String)> {
        // Вектор полей структуры, хранит кортежи (имя, тип). Они собраны
        // анализатором для имени структуры ApplicationUiBlockStruct{id}
        let default = Vec::new();
        let fields_data = self
            .ir
            .screen_structs
            .get(&format!("ApplicationUiBlockStruct{}", id))
            .unwrap_or(&default);

        // Проход по всем сырым полям чтобы сгенерировать field через quote
        // с сохранением спана (для ошибок)
        for (field_name_raw, field_type_raw) in fields_data {
            // Имя и тип поля
            let field_name = format_ident!("{}", field_name_raw);
            let field_type_tokens: TokenStream = field_type_raw
                .parse()
                .expect("Failed to parse field_type to tokens");

            let field_type: Type = syn::parse2(quote_spanned! { span=>
                core::option::Option<#field_type_tokens>
            })
            .expect("IE: Failed to parse field type");

            // Кодогенерация поля
            let field = Field {
                attrs: Vec::new(),
                vis: Visibility::Inherited,
                mutability: FieldMutability::None,
                ident: Some(field_name),
                colon_token: Some(<Token![:]>::default()),
                ty: field_type,
            };

            fields.push(field);
        }

        fields_data.to_vec()
    }

    /// Проверяет нужно ли геенрировать структуру для этой функции
    fn should_generate_struct(&self) -> bool {
        match self.flags.compile_type {
            // В Shared режиме структура нужна только первой функции
            CompileType::Shared => self.functions_count == 1,

            // В компонентах это вообще не нужно так как сам компонент это и есть структура
            // и поля будут там
            CompileType::Component => false,

            // В обычном режиме структура нужная каждой функции так как каждая функция
            // это отдельный экран
            _ => true,
        }
    }

    /// проверяет возвращаемоет значение у функции
    fn check_function_has_return(&self, sig: &Signature) -> bool {
        match &sig.output {
            ReturnType::Default => false,
            ReturnType::Type(_, ty) => match ty.as_ref() {
                Type::Tuple(tuple) if tuple.elems.is_empty() => false,
                Type::Never(_) => false,
                _ => true,
            },
        }
    }

    /// Генерирует код для функции Build в Shared режиме и записывает новую функцию в
    /// new_item по мутабельной ссылке
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all, fields(id = ?id)))]
    fn generate_build(
        &self,
        new_items: &mut Vec<Item>,
        instance_item: Item,
        fields: &Vec<Field>,
        id: u128,
        span: Span,
    ) {
        // Только для shared
        let build_name = format_ident!("_fwc_fn_build{}", id);

        let struct_name = format_ident!("ApplicationUiBlockStruct{}", id);

        let mut fields_punctuated = syn::punctuated::Punctuated::<Field, token::Comma>::new();
        for field in fields {
            fields_punctuated.push(field.clone());
        }

        // HACK: Создание DelimSpan из Span для struct_duf
        let mut dummy_group = Group::new(proc_macro2::Delimiter::Brace, TokenStream::new());
        dummy_group.set_span(span);

        // Структура экрана где хранится состояние, компоненты и виджеты. Используется
        // создание вручную что позволяет достичь более высокой скорости компиляции
        let struct_def = Item::Struct(syn::ItemStruct {
            attrs: Vec::new(),
            vis: Visibility::Inherited,
            struct_token: token::Struct { span },
            ident: struct_name,
            generics: Generics::default(),
            fields: Fields::Named(syn::FieldsNamed {
                brace_token: token::Brace {
                    span: dummy_group.delim_span(),
                },
                named: fields_punctuated,
            }),
            semi_token: None,
        });

        // Проверка можно ли генерировать структуру сейчас, в Shared режиме
        // компиляции нужна только одна структура так как состояние глобальное
        // поэтому после первой генерации в Shared режиме генерировать структуру
        // и экземпляр больше нельзя
        if self.should_generate_struct() {
            new_items.push(struct_def);
            new_items.push(instance_item);

            if matches!(self.flags.compile_type, CompileType::Shared) {
                let (build_statements, build_check) = self.generate_shared_build(id);

                let tokens = quote! {
                    let mut _fwc_build = false;

                    #build_check

                    if _fwc_build {
                        #(#build_statements)*
                    }
                };

                // В обычном режиме просто вставка токенов в функцию
                #[cfg(not(feature = "safety-multithread"))]
                let tokens = quote! {
                    fn #build_name () {
                        #tokens
                    }
                };

                // Чтобы не было дедлока в эффектах необходимо в безопасном
                // режиме генерировать проверку на то что экземпляр не был
                // инициализирован (None)
                #[cfg(feature = "safety-multithread")]
                let tokens = quote! {
                    fn #build_name () {
                        if #_instance_ident.get().is_none() {
                            #tokens
                        }
                    }
                };

                if let Ok(item) = syn::parse2::<Item>(tokens) {
                    new_items.push(item);
                }
            }
        }
    }

    /// Генерирует инициализацию полей которые генерирует компилятор в конструкторе компонента
    fn extend_new(&self, item_impl: &mut ItemImpl, struct_name: &str) {
        if let Some(fields) = self.ir.component_structs.get(struct_name) {
            let fields_data: Vec<(String, String)> = fields
                .iter()
                .map(|(name, _type)| (name.clone(), _type.clone()))
                .collect();

            for item in &mut item_impl.items {
                if let ImplItem::Fn(method) = item
                    && method.sig.ident == "new"
                {
                    let mut visitor = SelfFieldAdder::new(fields_data.clone());
                    visitor.visit_block_mut(&mut method.block);
                }
            }
        }
    }
}
