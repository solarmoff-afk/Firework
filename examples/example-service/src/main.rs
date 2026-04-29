use firework_ui::{shared, effect};

shared! {
    state! {
        #[read] #[write]
        theme: u8 = 0,
    }

    
    #[effect(theme)]
    fn on_update_effect() {
        println!("Update!");
    }
}

fn main() {
    set_theme(10);
    println!("{}", get_theme());
}
