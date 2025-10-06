pub mod modules {

    pub fn test_fn() {
        println!("Подключили первый сторонний модуль");
    }

    pub fn test_fn2() {
        println!("Подключили второй сторонний модуль");
        
        //Проверка жизненного цикла строки
        let s1 = String::from("Hello, ");
        let s2 = String::from("world!");
        let s3 = s1 + &s2;
        println!("Сторонний модуль: {}",s3);
    }

}