use async_trait::async_trait;
use ractor::{Actor, ActorProcessingErr, ActorRef};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::time::Duration;

// Сообщения для актора
#[derive(Debug)]
pub enum PipeMessage {
    CheckFile,
    Stop,
}

// Структура актора
pub struct FileMonitor;

//Без этого кфг не работало. Вопрос: Это включение асинхронности?
#[cfg_attr(feature = "async-trait", crate::async_trait)]

//Структура Актора
impl Actor for FileMonitor {
    type Msg = PipeMessage;
    type State = ();
    type Arguments = ();

    //Вывод сообщений перед запуском
    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        println!("Монитор файла pipe.txt запущен!");
        println!("Файл: src/pipe.txt");
        println!("Запишите в него число");
        Ok(())
    }

    //Действия актора
    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        _state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {

            PipeMessage::CheckFile => {
                // Путь к файлу
                let file_path = "src/pipe.txt";
                // Пытаемся прочитать файл
                match read_file_content(file_path) {
                    Ok(Some(content)) => {
                        println!("Прочитано из файла: {}", content);
                        // Обрабатываем содержимое отправляя в функцию умножения
                        process_content(&content);
                        // Очищаем файл после чтения
                        if let Err(e) = clear_file(file_path) {
                            eprintln!("Ошибка очистки файла: {}", e);
                        }
                    }
                    //Если ничего нет - ничего не выводим
                    Ok(None) => {
                        // Файл пустой или не существует
                    }
                    //Обработка ошибки на случай невозможности чтения файла
                    Err(e) => {
                        eprintln!("Ошибка чтения файла: {}", e);
                    }
                }
                // Повторная проверка через 1с. Почему в примерах именно tokio?
                tokio::time::sleep(Duration::from_secs(1)).await;
                myself.send_message(PipeMessage::CheckFile)?;
            }

            PipeMessage::Stop => {
                println!("Остановка монитора...");
                myself.stop(Some("Остановка".to_string()));
            }
        }
        Ok(())
    }
}


// Чтение содержимого файла. Это решение нашел
fn read_file_content(file_path: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
    if !Path::new(file_path).exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(file_path)?;
    let trimmed_content = content.trim();
    
    if trimmed_content.is_empty() {
        Ok(None)
    } else {
        Ok(Some(trimmed_content.to_string()))
    }
}


// Очистка файла
fn clear_file(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(file_path)?;
    
    file.write_all(b"")?;
    Ok(())
}

// Обработка содержимого файла. Умножение на два.
fn process_content(content: &str) {
    match content.parse::<f64>() {
        Ok(number) => {
            let result = number * 2.0;
            println!("Результат: {} * 2 = {}", number, result);
        }
        Err(_) => {
            println!("Не число: '{}'", content);
            println!("Введите число в файл src/pipe.txt");
        }
    }
}

//Что делает это tokio? Везде его используют. А без него не работает))
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Монитор файла pipe.txt ===");
    println!("Создайте файл src/pipe.txt и напишите в него число");
    println!("Программа будет проверять файл каждую секунду");
    println!("Для выхода введите 'stop'");

    // Создаем актор. Решение из документации.
    let (actor, handle) = FileMonitor::spawn(None, FileMonitor, ())
        .await
        .expect("Не удалось создать актор");

    // Запускаем проверку файла
    actor.send_message(PipeMessage::CheckFile)?;

    // Простой метод для остановки
    let mut input = String::new();
    while input.trim() != "stop" {
        input.clear();
        std::io::stdin().read_line(&mut input)?;
    }

    // Останавливаем актор
    actor.send_message(PipeMessage::Stop)?;
    handle.await?;

    println!("Процесс остановлен");
    Ok(())
}