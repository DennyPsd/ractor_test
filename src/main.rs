use ractor::{Actor, ActorProcessingErr, ActorRef};
use std::{io, time::Duration};


// Сообщения для актора
#[derive(Debug)]
pub enum PipeMessage {
    Start,
    Number(i32),
    Stop,
}

pub struct PipeState {
    calculator_ref: Option<ActorRef<PipeMessage>>,
}

// Структура актора считывателя из терминала
pub struct TextReader;

impl Actor for TextReader {
    type Msg = PipeMessage;
    type State = PipeState;
    type Arguments = ActorRef<PipeMessage>;

    //Вывод сообщений перед запуском
    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        calculator_ref: ActorRef<PipeMessage>,
    ) -> Result<Self::State, ActorProcessingErr> {
        println!("Актор считывания запущен");
        Ok(PipeState { calculator_ref: (Some(calculator_ref)) })
    }

    //Действия актора
    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            PipeMessage::Start => {
                println!("Введите число в консоль...");

                if let Some(n) = read_content() {
                    // Отправляем число в Calculator
                    if let Some(ref calc_ref) = state.calculator_ref {
                        calc_ref.send_message(PipeMessage::Number(n))?;
                    }
                }

                // Повторяем через 1 секунду
                tokio::time::sleep(Duration::from_secs(1)).await;
                myself.send_message(PipeMessage::Start)?;

        }
        PipeMessage::Number(n) => {
            println!("Получено число: {}", n);
        }
        PipeMessage::Stop => {
            println!("Остановка актора...");
            myself.stop(None);
        }
            }
    Ok(())
        }
    }


   //Сктруктура Актора калькулятора 
pub struct Calculator;

impl Actor for Calculator {
    type Msg = PipeMessage;
    type State = PipeState;
    type Arguments = ();

    //Вывод сообщений перед запуском
    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        _: (),
    ) -> Result<Self::State, ActorProcessingErr> {
        println!("Актор калькуляции запущен");
        Ok(PipeState { calculator_ref: (None) })
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        _state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            PipeMessage::Start => {
        }
        PipeMessage::Number(n) => {
            println!("Калькулятор обработал число: {}", n*2);
        }
        PipeMessage::Stop => {
            println!("Остановка актора...");
            myself.stop(None);
        }
            }
    Ok(())
        }
}



//Чтение из консоли
fn read_content () -> Option<i32> {

    let mut number_string = String::new();
    
    io::stdin()
        .read_line(&mut number_string)
        .expect("Ошибка чтения строки");

let number_int = number_string.trim();
    match number_int.parse::<i32>() {
        Ok(i) => {
        println!("Получено число:{}",number_int);
        Some(i)
        }

        Err(..) => {
            println!("Вы ввели не число: {}", number_int);
            None
        }
    }
}


//Что делает это tokio? Везде его используют. А без него не работает))
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Мониторинг консоли ===");


    let (calculator, calculator_handle) = Calculator::spawn(None, Calculator, ())
    .await
    .expect("Не удалось создать калькулятор");

    // Создаем актор. Решение из документации.
    let (text_reader, text_reader_handle) = TextReader::spawn(None, TextReader, calculator.clone())
        .await
        .expect("Не удалось создать актор");

    text_reader.send_message(PipeMessage::Start)?;

    text_reader_handle.await?;
    calculator_handle.await?;
    Ok(())
}