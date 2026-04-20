use firework_ui::shared;

shared! {
    state! {
        theme: u8 = 0,
    }

    pub fn get_theme() -> u8 {
        let theme = spark_ref!(theme);
        
        if *theme > 5 {
            println!("Hi");
        }

        *theme
    }

    fn set_theme(new_theme: u8) {
        let mut theme = spark_ref!(theme);
        *theme = new_theme;
    }
}

fn main() {
    set_theme(1);
    println!("{}", get_theme());
}
