// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

pub use super::super::*;

impl<'ast> Analyzer {
    /// Генерирует заглушки для функций чтобы компилятор не выдал ошибку "функция отсуствует"
    /// вероятно это временное решение. Также собирает сигнатуру функции для кодогенератора
    pub(crate) fn analyze_item_fn(&mut self, node: &'ast ItemFn) {
        self.item_scope = self.scope.clone();
        self.context.layouts_count = 0;

        let mut function_head = String::from("");
        for attr in &node.attrs {
            function_head.push_str(format!("{}\n", quote::quote! { #attr }).as_str());
        }
        
        let vis = &node.vis;
        let constness = &node.sig.constness;
        let asyncness = &node.sig.asyncness;
        let unsafety = &node.sig.unsafety;
        let abi = &node.sig.abi;
        let fn_token = &node.sig.fn_token;
        let ident = &node.sig.ident;
        let generics = &node.sig.generics;
        let inputs = &node.sig.inputs;
        let output = &node.sig.output;
        
        let signature = quote::quote! {
            #vis #constness #asyncness #unsafety #abi #fn_token #ident #generics (#inputs) #output
        };

        function_head.push_str(format!("{}", signature).as_str());

        let mut fn_stub = node.clone();
        fn_stub.block = syn::parse2(quote::quote! {
            {}
        }).expect("Failed to parse item"); 
        
        self.context.output.extend(quote::quote! {
            #fn_stub
        });

        // Добавление всех аргументов в область видимости как переменных
        for input in &node.sig.inputs {
            self.visit_fn_arg(input);
        }

        let function_name = node.sig.ident.to_string();
        self.function_name = Some(function_name.clone());
        self.context.ir.screens.push(
            (
                function_name.clone(),
                function_head,
                self.scope.screen_index
            )
        );
        self.context.statement.screen_name = function_name;

        // Любой код в функции реактивный
        self.context.statement.reactive_loop = true;

        syn::visit::visit_item_fn(self, node);

        // После парсинга функции нужно добавить стейтемент который уведомит
        // кодогенератор о завершении тела функции чтобы он перед этим сгенерировал
        // выход из цикла реактивности если этого ещё никто не сделал
        let mut statement = self.context.statement.clone();
        statement.action = FireworkAction::Terminator;

        // Больше не часть цикла реактивности
        statement.reactive_loop = false;

        self.context.ir.statements.push(statement);
        self.context.ir.screen_sparks.insert(self.scope.screen_index, self.context.spark_counter);

        // Нужно сгенерировать индекс после анализации функции чтобы id экземпляра
        // был синхронизирован внутри блоков ir для одного экрана
        self.scope.screen_index_generate();

        // Обнуление счётчика реактивных переменных чтобы можно было считать что индекс
        // реактивной переменной это бит в битовой маске
        self.context.spark_counter = 0; 
    }

    pub(crate) fn analyze_fn_arg(&mut self, i: &'ast FnArg) {
        if let FnArg::Typed(pat_type) = i {
            self.current_type = pat_type.ty.to_token_stream().to_string();
        
            // Переменные будут добавляться в pending_vars
            self.visit_pat(&pat_type.pat);
        
            for (name, mut var_data) in self.pending_vars.drain(..) {
                // Аргумент функции не может быть спарком
                var_data.is_spark = false;

                self.scope.variables.insert(name, var_data);
            }

            self.current_type = String::from(NO_TYPE);
        }
    }
}
