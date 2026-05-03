use firework_ui::{effect, shared};

shared! {
    state! {
        #[read] #[write]
        theme: u8 = 0,
    }

    fn my_get_theme() -> u8 {
        let theme_ref = spark_ref!(theme);
        *theme_ref
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
