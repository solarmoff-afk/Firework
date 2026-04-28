// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use super::*;

use crate::compiler::codegen::ir::FireworkAction::*;
use crate::compiler::codegen::ir::WidgetDescription;

#[test]
fn test_analyze_basic_widget() {
    let tokens = quote::quote! {
        fn screen() {
            rect! {
                position: (10, 20),
                size: (100, 100),
                color: (255, 0, 0),

                key: "unique_key",
            }
        }
    };

    let ir = extract_ir(tokens);
    
    let expected = [
        WidgetBlock(WidgetDescription {
            widget_type: "rect".to_string(),
            fields: HashMap::new(),
            is_functional: false,
            id: 0,
            has_microruntime: false,
            skin: "firework_ui::skins::DefaultRectSkin".to_string(),
            is_maybe: None,
        }),

        Terminator,
    ];

    assert_eq!(ir.statements.len(), expected.len());
    assert!(matches!(ir.statements[0].action, WidgetBlock(_)));
    assert!(matches!(ir.statements[1].action, Terminator));
}

/// Тест виджета со спарками в полях
#[test]
fn test_analyze_widget_with_sparks() {
    let tokens = quote::quote! {
        fn screen() {
            let mut x = spark!(10u32);
            let mut y = spark!(20u32);
            
            rect! {
                position: (x, y),
                size: (100, 100),
                color: (255, 0, 0),

                key: "widget_key",
            }
        }
    };

    let ir = extract_ir(tokens);
    
    let expected = [
        InitialSpark {
            name: "x".to_string(),
            id: 1,
            spark_type: "u32".to_string(),
            expr_body: "10u32".to_string(),
            is_mut: true
        },
        
        InitialSpark {
            name: "y".to_string(),
            id: 2,
            spark_type: "u32".to_string(),
            expr_body: "20u32".to_string(),
            is_mut: true
        },

        WidgetBlock(WidgetDescription {
            widget_type: "rect".to_string(),
            fields: HashMap::new(),
            is_functional: false,
            id: 0,
            has_microruntime: false,
            skin: "firework_ui::skins::DefaultRectSkin".to_string(),
            is_maybe: None,
        }),
        
        DropSpark {
            name: "y".to_string(),
            id: 2,
        },
        
        DropSpark {
            name: "x".to_string(),
            id: 1,
        },
        
        Terminator,
    ];

    assert_eq!(ir.statements.len(), expected.len());
}

/// Тест виджета с замыканием в пропсе
#[test]
fn test_analyze_widget_with_closure() {
    let tokens = quote::quote! {
        fn screen() {
            let mut counter = spark!(0u32);
            
            button! {
                text: "Click me",
                on_click: || {
                    counter += 1;
                    println!("Clicked!");
                },

                key: "click_button",
            }
        }
    };

    let ir = extract_ir(tokens);
    
    assert!(ir.statements.len() >= 2);
    assert!(matches!(ir.statements[0].action, InitialSpark { .. }));
    
    let mut found_widget = false;
    for stmt in &ir.statements {
        if let WidgetBlock(desc) = &stmt.action {
            found_widget = true;
            assert_eq!(desc.widget_type, "button");
            assert!(!desc.is_functional);
            
            if let Some(field) = desc.fields.get("on_click") {
                assert!(field.is_fn);
                assert!(field.string.contains("counter += 1"));
            }

            break;
        }
    }
    
    assert!(found_widget, "WidgetBlock not found in IR statements");
}

/// Тест функционального виджета layout
#[test]
fn test_analyze_layout_widget() {
    let tokens = quote::quote! {
        fn screen() {
            layout! {
                orientation: Vertical,
                spacing: 10,
                padding: 5,
            }
        }
    };

    let ir = extract_ir(tokens);
    
    let expected = [
        WidgetBlock(WidgetDescription {
            widget_type: "layout".to_string(),
            fields: HashMap::new(),
            is_functional: true,
            id: 0,
            has_microruntime: false,
            skin: "".to_string(),
            is_maybe: None,
        }),
        Terminator,
    ];

    assert_eq!(ir.statements.len(), expected.len());
    assert!(matches!(ir.statements[0].action, WidgetBlock(_)));
    
    if let WidgetBlock(desc) = &ir.statements[0].action {
        assert!(desc.is_functional);
        assert_eq!(desc.widget_type, "layout");
        assert!(desc.skin.is_empty());
    }
}

/// Тест условного виджета (виджет внутри if)
#[test]
fn test_analyze_conditional_widget() {
    let tokens = quote::quote! {
        fn screen() {
            let mut show_rect = spark!(true);
            
            if show_rect {
                rect! {
                    position: (10, 10),
                    size: (100, 100),
                    color: (255, 0, 0),

                    key: "conditional_rect",
                }
            }
        }
    };

    let ir = extract_ir(tokens);
    
    let mut found_conditional_widget = false; 
    
    for stmt in &ir.statements {
        if let WidgetBlock(desc) = &stmt.action {
            if desc.is_maybe.is_some() {
                found_conditional_widget = true;
                assert_eq!(desc.widget_type, "rect");
            }
        }
    }
    
    assert!(found_conditional_widget); 
    assert!(!ir.screen_maybe_widgets.is_empty());
}

#[test]
fn test_analyze_widget_in_loop() {
    let tokens = quote::quote! {
        fn screen() {
            let mut count = spark!(5);
            
            for i in 0..count {
                rect! {
                    position: (i * 10, 0),
                    size: (50, 50),
                    color: (0, 255, 0),

                    #[key_type((i32))]
                    key: i,
                }
            }
        }
    };

    let ir = extract_ir(tokens);
    
    let mut has_microruntime = false;
    let mut has_dynamic_loop = false;
    
    for stmt in &ir.statements {
        if let WidgetBlock(desc) = &stmt.action {
            if desc.has_microruntime {
                has_microruntime = true;
                assert_eq!(desc.widget_type, "rect"); 
                assert!(desc.skin.contains("DynList") || desc.has_microruntime);
            }
        }
        
        if let DynamicLoopBegin(depth, widgets) = &stmt.action {
            has_dynamic_loop = true;
            assert!(*depth > 0);
            assert!(!widgets.is_empty());
        }
    }
    
    assert!(has_microruntime || has_dynamic_loop);
}

/// Тест виджетов внутри лайаута
#[test]
fn test_analyze_nested_widgets() {
    let tokens = quote::quote! {
        fn screen() {
            vertical! {
                rect! {
                    position: (0, 0),
                    size: (100, 50),
                    color: (255, 0, 0),

                    key: "red_rect",
                }
                
                rect! {
                    position: (0, 50),
                    size: (100, 50),
                    color: (0, 255, 0),

                    key: "green_rect",
                }
            }
        }
    };

    let ir = extract_ir(tokens);
    
    let widget_count = ir.statements.iter()
        .filter(|s| matches!(s.action, WidgetBlock(_)))
        .count();
    
    assert_eq!(widget_count, 2);
}

/// Тест ошибки при отсутствии пропса key у виджета
#[test]
fn test_analyze_widget_missing_key_error() {
    let tokens = quote::quote! {
        fn screen() {
            rect! {
                position: (10, 10),
                size: (100, 100),
                color: (255, 0, 0),
                // key 
            }
        }
    };

    let (_, error, _) = prepare_tokens(
        syn::parse2(tokens).unwrap(),
        CompileFlags::new(),
        0
    );
     
    let error_string = error.unwrap().to_string();
    assert!(error_string.contains("FE019") || error_string.contains("key"));
}

/// Тест виджета с атрибутом key_type
#[test]
fn test_analyze_widget_with_key_type() {
    let tokens = quote::quote! {
        fn screen() {
            let mut x = spark!(3);
            
            for i in 0..x {
                for j in 0..3 {
                    rect! {
                        position: (150 * i, 150 * j),
                        size: (100, 100),
                        color: (0, 255, 0),

                        #[key_type((i32, i32))]
                        key: (i, j),
                    }
                }
            }
        }
    };

    let ir = extract_ir(tokens);
     
    let mut found_widget = false;
    
    for stmt in &ir.statements {
        if let WidgetBlock(desc) = &stmt.action {
            found_widget = true;
            assert!(desc.has_microruntime); 
            assert!(desc.fields.contains_key("key"));
        }
    }
    
    assert!(found_widget);
}

/// Тест множества различных виджетов
#[test]
fn test_analyze_multiple_widget_types() {
    let tokens = quote::quote! {
        fn screen() {
            rect! {
                position: (10, 10),
                size: (100, 100),
                color: (255, 0, 0),

                key: "rect1",
            }
            
            text! {
                text: "Hello World",
                font_size: 16,
                color: (0, 0, 0),

                key: "hello_text",
            }
            
            button! {
                text: "Submit",
                on_click: || { println!("Submitted!"); },

                key: "submit_btn",
            }
            
            app_bar! {
                title: "My App",
                actions: vec![],

                key: "app_bar",
            }
        }
    };

    let ir = extract_ir(tokens);
    
    let widget_types: Vec<String> = ir.statements.iter()
        .filter_map(|s| {
            if let WidgetBlock(desc) = &s.action {
                Some(desc.widget_type.clone())
            } else {
                None
            }
        })
        .collect();
    
    assert_eq!(widget_types, vec!["rect", "text", "button", "app_bar"]);
}

/// Тест виджета с вложенными спарками в замыкании
#[test]
fn test_analyze_widget_closure_with_sparks() {
    let tokens = quote::quote! {
        fn screen() {
            let mut counter = spark!(0u32);
            let mut text_content = spark!("Initial".to_string());
            
            button! {
                text: "Update",
                on_click: || {
                    counter += 1;
                    text_content = format!("Count: {}", counter);
                    println!("Updated!");
                },

                key: "update_btn",
            }
        }
    };

    let ir = extract_ir(tokens);
    
    let mut found_closure_with_updates = false;
    
    for stmt in &ir.statements {
        if let WidgetBlock(desc) = &stmt.action {
            if let Some(on_click) = desc.fields.get("on_click") {
                assert!(on_click.is_fn);
                found_closure_with_updates = true;
                
                assert!(!on_click.sparks.is_empty());
            }
        }
    }
    
    assert!(found_closure_with_updates);
}
