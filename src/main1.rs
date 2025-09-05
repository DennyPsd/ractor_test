use async_trait::async_trait;
use ractor::{Actor, ActorProcessingErr, ActorRef};
use std::io::{self, Write};
use std::process::Stdio;
use tokio::process::Command;

// Сообщения для актора
#[derive(Debug)]
pub enum ConsoleMessage {
    ReadInput,
    ProcessInput(String),
}

// Структура актора
pub struct ConsoleActor;

#[cfg_attr(feature = "async-trait", crate::async_trait)]
impl Actor for ConsoleActor {
    type Msg = ConsoleMessage;
    type State = ();
    type Arguments = ();

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        println!("Консольный актор запущен!");
        println!("Введите число и нажмите Enter (или 'quit' для выхода):");
        Ok(())
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        _state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            ConsoleMessage::ReadInput => {
                // Читаем ввод из консоли с помощью echo (имитация)
                match read_input_from_echo().await {
                    Ok(input) => {
                        // Отправляем сообщение для обработки ввода
                        myself.send_message(ConsoleMessage::ProcessInput(input))?;
                    }
                    Err(e) => {
                        eprintln!("Ошибка чтения ввода: {}", e);
                    }
                }
            }
            ConsoleMessage::ProcessInput(input) => {
                if input.trim().eq_ignore_ascii_case("quit") {
                    println!("Завершение работы...");
                    myself.stop(Some("Пользовательский выход".to_string()));
                    return Ok(());
                }

                // Пытаемся преобразовать в число и умножить на 2
                match input.trim().parse::<f64>() {
                    Ok(number) => {
                        let result = number * 2.0;
                        println!("Результат ({} * 2): {}", number, result);
                    }
                    Err(_) => {
                        println!("Неверный ввод: '{}'. Введите число или 'quit'", input.trim());
                    }
                }

                // Запрашиваем следующий ввод
                println!("Введите следующее число:");
                myself.send_message(ConsoleMessage::ReadInput)?;
            }
        }
        Ok(())
    }
}

// Функция для чтения ввода через echo (имитация)
async fn read_input_from_echo() -> Result<String, Box<dyn std::error::Error>> {
    // В реальном приложении здесь был бы вызов echo, но для простоты используем stdin
    let mut input = String::new();
    
    // Выводим приглашение и читаем ввод
    print!("> ");
    io::stdout().flush()?;
    
    // Чтение из stdin (в реальном случае было бы через echo)
    io::stdin().read_line(&mut input)?;
    
    Ok(input)
}

// Альтернативная версия с использованием команды echo (если нужно именно через echo)
async fn read_input_via_echo() -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("bash")
        .arg("-c")
        .arg("read -p '> ' input && echo $input")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await?;

    if output.status.success() {
        let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(result)
    } else {
        let error = String::from_utf8_lossy(&output.stderr).to_string();
        Err(error.into())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Запуск консольного актора...");

    // Создаем актор
    let (actor, handle) = ConsoleActor::spawn(None, ConsoleActor, ())
        .await
        .expect("Не удалось создать актор");

    // Запускаем цикл чтения ввода
    actor.send_message(ConsoleMessage::ReadInput)?;

    // Ждем завершения актора
    match handle.await {
        Ok(_) => {
            println!("Актор завершил работу");
        }
        Err(e) => {
            eprintln!("Ошибка при завершении актора: {}", e);
        }
    }

    println!("Программа завершена!");
    Ok(())
}