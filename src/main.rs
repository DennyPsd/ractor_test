use ractor::{Actor, ActorProcessingErr, ActorRef};
use std::{io, time::Duration};
use regex::Regex;


// Сообщения для актора
#[derive(Debug)]
pub enum PipeMessage {
    Start,
    Message(Vec<String>),
    Error(String),
    Stop,
}

pub struct PipeState {
    calculator_ref: Option<ActorRef<PipeMessage>>,
    err_handler_ref: Option<ActorRef<PipeMessage>>,
}

// Структура актора считывателя из терминала
pub struct TextReader;

impl Actor for TextReader {
    type Msg = PipeMessage;
    type State = PipeState;
    type Arguments = (ActorRef<PipeMessage>, ActorRef<PipeMessage>);

    //Вывод сообщений перед запуском
    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        (calculator_ref, err_handler_ref): (ActorRef<PipeMessage>, ActorRef<PipeMessage>),
    ) -> Result<Self::State, ActorProcessingErr> {
        println!("Актор считывания запущен");
        Ok(PipeState { 
            calculator_ref: Some(calculator_ref),
            err_handler_ref: Some(err_handler_ref),
         })
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
                println!("Введите выражение в консоль (например 2+4)");

                match read_content() {
                    Some(text) => {
                        println!("Прочитано: {:?}", text);
                        if let Some(ref calc_ref) = state.calculator_ref {
                            calc_ref.send_message(PipeMessage::Message(text))?;
                        }
                    }
                    None => {
                        let error_msg = "Неверный формат выражения.".to_string();
                        println!("{}", error_msg);
                        if let Some(ref err_ref) = state.err_handler_ref {
                            err_ref.send_message(PipeMessage::Error(error_msg))?;
                        }
                    }
                    
                }

                // Повторяем через 1 секунду
                tokio::time::sleep(Duration::from_secs(1)).await;
                myself.send_message(PipeMessage::Start)?;

        }
        PipeMessage::Stop => {
            println!("Остановка актора...");
            myself.stop(None);
        }
        _=> {}
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
        Ok(PipeState {
            calculator_ref:None,
            err_handler_ref: None,
        })
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        _state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            PipeMessage::Start => {}
        PipeMessage::Stop => {
            println!("Остановка актора...");
            myself.stop(None);
        }
        _ => {}
            }
    Ok(())
        }
}


pub struct OutActor;

impl Actor for OutActor {
    type Msg = PipeMessage;
    type State = ();
    type Arguments = ();
    
    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        _:(),
    ) -> Result<Self::State, ActorProcessingErr> {
        println!("Актор вывода сообщений запущен!");
        Ok(())
    }

    async fn handle (
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        _state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message{
            PipeMessage::Error(msg) => {
                println!("Ошибка{}", msg);
            }
            PipeMessage::Stop => {
                println!("Остановка актора вывода...");
                myself.stop(None);
            }
            _ => {}
        }
        Ok(())
    }
}



//Чтение из консоли
fn read_content () -> Option<Vec<String>> {

    let mut input = String::new();
    
    io::stdin()
        .read_line(&mut input)
        .expect("Ошибка чтения строки");

let input = input.trim();
let re = Regex::new(r"^\s*([0-9]+)\s*([+\-*/])\s*([0-9]+)\s*$").unwrap();
    if let Some(cap) = re.captures(input) {
        let first = cap.get(1).unwrap().as_str().to_string();
        let mid= cap.get(2).unwrap().as_str().to_string();
        let right = cap.get(3).unwrap().as_str().to_string();

        println!("Получено выражение: {} {} {}", first, mid, right);
        Some(vec![first, mid, right])
    } else {
        println!("Ввели неверное выражение: {}", input);
        None
    }
}


//Что делает это tokio? Везде его используют. А без него не работает))
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Мониторинг консоли ===");

    //Спавним акторы без ссылок
    let (calculator, calculator_handle) = Calculator::spawn(None, Calculator, ())
    .await
    .expect("Не удалось создать актор калькулятора");

    let (err_handler, err_handler_handler) = OutActor::spawn(None,OutActor,())
    .await
    .expect("Не удалось создать актор вывода");

    //Спавним актор чтения и передаем две ссылки в него
    let (text_reader, text_reader_handle) = TextReader::spawn(None, TextReader, (calculator.clone(), err_handler.clone()))
        .await
        .expect("Не удалось создать актор чтения");

    text_reader.send_message(PipeMessage::Start)?;

    text_reader_handle.await?;
    calculator_handle.await?;
    Ok(())
}