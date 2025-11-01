pub fn run<F>(app: F)
where
    F: Fn() -> String,
{
    let ui_content = app();
    println!("Firework started");
    println!("UI content {}", ui_content);
}

pub fn hello_firework() -> String {
    "Hello, firework!".to_string()
}