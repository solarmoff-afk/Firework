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
                        None,
                    );

                    new_items.push(Item::Fn(item_fn));
                }

                Item::Struct(mut item_struct) => {
                    // Только если это компонент
                    if matches!(self.flags.compile_type, CompileType::Component) {
                        let struct_name = item_struct.ident.clone();

                        self.generate_component_setters(&mut item_struct, &mut new_items);
                        self.codegen_item_struct(&mut item_struct);
                        new_items.push(Item::Struct(item_struct.clone()));

                        let component_declaration =
                            self.ir.component_structs.get(&struct_name.to_string());
                        let (
                            get_z_size_tokens,
                            set_z_tokens,
                            is_dirty_check,
                            visible_tokens,
                            unmount_tokens,
                            event_tokens,
                            generics,
                            type_generics,
                        ) = if let Some(declaration) = component_declaration {
                            (
                                self.gen_get_z_size_tokens(declaration),
                                self.gen_set_s_tokens(declaration),
                                self.gen_is_dirty_check(declaration),
                                self.gen_visible_tokens(declaration),
                                self.gen_unmount_tokens(declaration),
                                self.gen_event_tokens(declaration),
                                declaration.tcomponents.clone(),
                                declaration.struct_generics.clone(),
                            )
                        } else {
                            (
                                quote! {},
                                quote! {},
                                quote! {},
                                quote! {},
                                quote! {},
                                quote! {},
                                Generics::default(),
                                Generics::default(),
                            )
                        };

                        // Методы компонента
                        let component_funcs = parse_quote! {
                            impl #generics #struct_name #type_generics {
                                // &mut self нельзя так как трейт Widget требует position с
                                // &self, а __set_position используется в position, а он
                                // используется в position для реализации трейта Widget
                                // для компонента, а без трейта Widget компонент не может
                                // использоваться в динамических списках и так далее
                                fn __set_position(&self, position: (i32, i32)) {
                                    if let Some(_fwc_component) = &self._fwc__fwc_component {
                                        _fwc_component.set_position(position);
                                    }
                                }

                                fn __set_width(&mut self, width: i32) {
                                    if let Some(_fwc_component) = &mut self._fwc__fwc_component {
                                        _fwc_component.size.0 = width;
                                        _fwc_component.set_size(_fwc_component.size);
                                    }
                                }

                                fn __set_height(&mut self, height: i32) {
                                    if let Some(_fwc_component) = &mut self._fwc__fwc_component {
                                        _fwc_component.size.1 = height;
                                        _fwc_component.set_size(_fwc_component.size);
                                    }
                                }

                                fn __set_size(&mut self, size: (i32, i32)) {
                                    if let Some(_fwc_component) = &mut self._fwc__fwc_component {
                                        _fwc_component.size = size;
                                        _fwc_component.set_size(size);
                                    }
                                }

                                pub fn position(self, position: (i32, i32)) -> Self {
                                    self.__set_position(position);
                                    self
                                }

                                pub fn width(mut self, width: i32) -> Self {
                                    self.__set_width(width);
                                    self
                                }

                                pub fn height(mut self, height: i32) -> Self {
                                    self.__set_height(height);
                                    self
                                }

                                pub fn z(self, z: i16) -> Self {
                                    <Self as firework_ui::std_widgets::widget::Widget>::set_z(&self, z);
                                    self
                                }

                                pub fn size<S: firework_ui::IntoSise>(mut self, size: S) -> Self {
                                    self.__set_size(size.into_size());
                                    self
                                }

                                pub fn __set_vcanvas(self, handle: usize) -> Self {
                                    <Self as firework_ui::std_widgets::widget::Widget>::set_vcanvas(&self, handle);
                                    self
                                }

                                pub fn __id(&self) -> usize {
                                    if let Some(_fwc_component) = &self._fwc__fwc_component {
                                        _fwc_component.substrate
                                    } else {
                                        0
                                    }
                                }

                                // Метод, который вызывается в случае, если у component! {}
                                // декларации есть ивент
                                pub fn __set_event(self) -> Self {
                                    if let Some(_fwc_component) = &self._fwc__fwc_component {
                                        firework_ui::adapter_command(
                                            firework_ui::AdapterCommand::SetHitGroup(
                                                _fwc_component.substrate,
                                                firework_ui::TOUCH_HIT_GROUP,
                                            )
                                        );
                                    }

                                    self
                                }

                                pub fn __is_dirty(&mut self) -> bool {
                                    #is_dirty_check
                                }

                                pub fn __fwc_event(&mut self, context: firework_ui::BuildContext) {
                                    <Self as firework_ui::std_widgets::widget::Widget>::__event(self, context);
                                }
                            }
                        };

                        // Реализация виджета
                        let widget_impl = parse_quote! {
                            impl #generics firework_ui::std_widgets::widget::Widget for #struct_name #type_generics {
                                fn position(&self, position: (i32, i32)) {
                                    self.__set_position(position);
                                }

                                fn visible(&self, visible: bool) {
                                    #visible_tokens
                                }

                                fn unmount(self) {
                                    #unmount_tokens
                                }

                                fn layout(
                                    &mut self,
                                    constraints: firework_ui::layout::Constraints
                                ) -> firework_ui::layout::Size {
                                    // TODO: Заменить на реальный расчёт размера
                                    firework_ui::layout::Size {
                                        width: 0,
                                        height: 0,
                                    }
                                }

                                fn get_z_size(&self) -> i16 {
                                    // По умолчанию 1 из-за подложки которая занимает 1 z
                                    let mut _fwc_z: i16 = 1;
                                    #get_z_size_tokens

                                    _fwc_z
                                }

                                fn set_z(&self, z: i16) {
                                    let mut _fwc_z: i16 = z;
                                    #set_z_tokens

                                    // У подложки самый высокий z, так как она должна хватать
                                    // on_click события в случае, если на компонент навешен
                                    // on_clock ивент
                                    if let Some(component_data) = &self._fwc__fwc_component {
                                        firework_ui::adapter_command(
                                            firework_ui::AdapterCommand::SetZ(
                                                component_data.substrate, _fwc_z + 1
                                            )
                                        );
                                    }
                                }

                                fn set_vcanvas(&self, vcanvas_handle: usize) {
                                    if let Some(component_data) = &self._fwc__fwc_component {
                                        firework_ui::adapter_command(
                                            firework_ui::AdapterCommand::SharedVCanvas(
                                                component_data.substrate, vcanvas_handle
                                            )
                                        );
                                    }
                                }

                                fn __event(&mut self, context: firework_ui::BuildContext) {
                                    #event_tokens
                                }
                            }
                        };

                        new_items.push(component_funcs);
                        new_items.push(widget_impl);
                    }
                }

                Item::Impl(mut item_impl) => {
                    let component_name = if let Type::Path(type_path) = &*item_impl.self_ty
                        && let Some(segment) = type_path.path.segments.last()
                    {
                        let struct_name = segment.ident.to_string();
                        self.extend_new(&mut item_impl, &struct_name);

                        Some(struct_name)
                    } else {
                        None
                    };

                    for item in &mut item_impl.items {
                        if let ImplItem::Fn(method) = item
                            && method.sig.ident == "flash"
                        {
                            self.transform_ui_function(
                                &mut method.sig,
                                &mut method.block,
                                &mut new_items,
                                component_name.clone(),
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
        component_name: Option<String>,
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

        // Генерация tick функции для экранов. Она нужна для асинхронности и в будущем
        // анимаций. Вызывается каждый кадр
        if matches!(self.flags.compile_type, CompileType::Screen) {
            let tick_name = format_ident!("_fwc_tick_{}", self.ui_id.unwrap_or(0));
            new_items.push(parse_quote! {
                fn #tick_name() {
                    // println!("Hello world");
                }
            });
        }

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

            let default_wmask_count = 0;
            let widget_mask_count = self
                .widget_mask_count
                .get(&id)
                .unwrap_or(&default_wmask_count);

            for mask_index in 0..*widget_mask_count {
                // Имя локальной маски и имя поле идентично
                let field_name = format!("_fwc__fwc_widget_bitmask{}", mask_index + 1);
                let copy_name = format!("_fwc_widget_bitmask{}.get()", mask_index + 1);
                let set_field_str =
                    static_gen::set_field(&struct_name_raw, &field_name, &copy_name);

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

                // Для компонента мы берём не Navigate, а фазу цикла из контекста родителя
                let (init_event_statement, init_depth) = if is_component {
                    // SAFETY: Так как для компонента обязательно должен существовать метод
                    // flash и аргумент контекста, тут всегда будет Some
                    let context_arg_name =
                        format_ident!("{}", self.last_context_arg_name.as_ref().expect("IE:13"));

                    (
                        quote! { let mut _fwc_event = #context_arg_name.cycle; },
                        quote! { let _fwc_depth = #context_arg_name.depth + 1; },
                    )
                } else {
                    (
                        quote! { let mut _fwc_event = firework_ui::LifeCycle::Navigate; },
                        quote! { let _fwc_depth = 0; },
                    )
                };

                // Для компонентов собираем код, который подтягивает изменения из глобальной
                // битмаски пропсов в локальную битмаску спарков
                let global_to_local = if let Some(ref name) = component_name
                    && let Some(declaration) = self.ir.component_structs.get(name)
                {
                    let mut code = quote! {};

                    for prop in &declaration.props {
                        // Локальная маска на уровне флеша
                        let local_bit =
                            if let Some(local) = declaration.component_prop_id.get(&prop.name) {
                                local
                            } else {
                                &0
                            };
                        let local_bitmask = get_spark_mask(*local_bit);
                        let local_bitmask_name = format_ident!("_fwc_bitmask{}", local_bitmask);
                        let local_id: u8 = normalize_bit_index(*local_bit);

                        // Глобальная маска на уровне компонента
                        let global_bitmask = get_spark_mask(prop.bit) - 1;
                        let global_bitmask_name =
                            format_ident!("_fwc_component_bitmask_{}", global_bitmask);
                        let global_bit: u8 = normalize_bit_index(prop.bit);

                        code.extend(quote! {
                            let _fwc_global_bit = ((self.#global_bitmask_name >> #global_bit) & 1) as u64;
                            #local_bitmask_name.set(#local_bitmask_name.get() | (_fwc_global_bit << #local_id));
                        });
                    }

                    code
                } else {
                    quote! {}
                };

                // Мёртвый код для shared режима, но так как он весь завёрнут в _{name}
                // (с _) то предупреждений не будет, а компилятор раста просто вырежет
                // этот код в релизной сборке как мёртвый
                final_stmts.extend(parse_batch(quote! {
                    let mut _fwc_z: i16 = 0;
                    let mut _fwc_z_dirty = false;
                    #init_event_statement
                    #init_code
                    let mut _fwc_guard: u8 = 0;
                    #(#bitmask_statements)*
                    #(#widget_bitmask_statement)*

                    if firework_ui::tiny_matches!(_fwc_event, firework_ui::LifeCycle::Build) || firework_ui::tiny_matches!(_fwc_event, firework_ui::LifeCycle::Navigate) {
                        _fwc_z_dirty = true;
                    }

                    #init_depth
                    #global_to_local
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

                let default = Vec::new();
                let widget = if is_component
                    && let Some(name) = component_name.clone()
                    && let Some(component) = self.ir.component_structs.get(&name)
                {
                    &component.widgets.widgets
                } else if let Some(screen) = self.ir.screen_structs.get(&struct_name_raw) {
                    &screen.widgets.widgets
                } else {
                    &default
                };

                let mut z_compute_tokens = quote! {};
                let z_compute_widget_tokens = quote! {
                    firework_ui::std_widgets::widget::Widget::set_z(&_fwc_element, _fwc_z);

                    // +1 так как следующий виджет займёт именно _fwc_z позицию,
                    // поэтому выделяется место под следующий
                    _fwc_z += firework_ui::std_widgets::widget::Widget::get_z_size(&_fwc_element) + 1;
                };

                let z_compute_dynlist_tokens = quote! {
                    // Устаналиваем списку этот z, DynList сам распределит z по элементам.
                    // Первый получит актуальный z, второй z+1 и так далее
                    _fwc_element.set_z_range(_fwc_z);

                    // Добавляем к _fwc_z количество элементов в списке +1 для слуедующего
                    // элемента
                    _fwc_z += _fwc_element.len() + 1;
                };

                for i in widget {
                    let name = format_ident!("{}", i.0);
                    let is_microruntime = i.2;

                    if
                    /* has_z */
                    i.3 {
                        continue;
                    }

                    let z_compute_variant = if is_microruntime {
                        &z_compute_dynlist_tokens
                    } else {
                        &z_compute_widget_tokens
                    };

                    if is_component {
                        z_compute_tokens.extend(quote! {
                            if let Some(_fwc_element) = self.#name {
                                #z_compute_variant
                            }
                        });
                    } else {
                        #[cfg(not(feature = "safety"))]
                        z_compute_tokens.extend(quote! {
                            unsafe {
                                if let Some(_fwc_element) = (*::core::ptr::addr_of!(#_instance_ident)).#name {
                                    #z_compute_variant
                                }
                            }
                        });

                        #[cfg(feature = "safety")]
                        z_compute_tokens.extend(quote! {
                            #_instance_ident.with(|inst| {
                                let mut _fwc_inst = inst.borrow_mut();

                                // Ref mut так как реализация только для &mut Widget из-за
                                // метода layout
                                if let Some(ref mut _fwc_element) = _fwc_inst.#name {
                                    #z_compute_variant
                                }
                            });
                        });
                    }
                }

                final_stmts.extend(parse_batch(quote! {
                    // #dyn_lists_end
                    #(#post_tokens)*
                    #widgets_gen_snapshot

                    if _fwc_z_dirty {
                        #z_compute_tokens
                    }
                }));

                if is_component
                    && let Some(component) = self
                        .ir
                        .component_structs
                        .get(&component_name.unwrap_or("".to_string()))
                {
                    if component.props_counter != 0 {
                        for i in 0..get_spark_mask(component.props_counter) {
                            let name = format_ident!("_fwc_component_bitmask_{}", i);

                            final_stmts.extend(parse_batch(quote! {
                                self.#name = 0;
                            }));
                        }
                    }
                }

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
        let default = Vec::new();
        let screen_name = format!("ApplicationUiBlockStruct{}", id);

        // Вектор полей структуры, хранит кортежи (имя, тип). Они собраны
        // анализатором для имени структуры ApplicationUiBlockStruct{id}
        let fields_data = if let Some(screen) = self.ir.screen_structs.get(&screen_name) {
            &screen.fields
        } else {
            &default
        };

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
            .expect(format!("IE: Failed to parse field type: \"{}\"", field_type_tokens).as_str());

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

                let tokens = quote! {
                    fn #build_name () {
                        #tokens
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
                .fields
                .iter()
                .map(|(name, _type)| (name.clone(), _type.clone()))
                .collect();

            for item in &mut item_impl.items {
                if let ImplItem::Fn(method) = item
                    && method.sig.ident == "new"
                {
                    let mut visitor = SelfFieldAdder::new(fields_data.clone());
                    visitor.props_count = fields.props_counter;
                    visitor.visit_block_mut(&mut method.block);
                }
            }
        }
    }
}
