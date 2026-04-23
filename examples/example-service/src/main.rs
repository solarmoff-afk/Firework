use firework_ui::{shared, effect};

shared! {
    state! {
        theme: u8 = 0,
    }

    pub fn get_theme() -> u8 {
        let theme = spark_ref!(theme);
        *theme
    }

    #[effect(theme)]
    fn on_update_effect() {
        println!("Update!");
    }

    fn set_theme(new_theme: u8) {
        let mut theme = spark_ref!(theme); 
        *theme = new_theme;
    }
}

fn main() {
    set_theme(10);
    println!("{}", get_theme());
}
