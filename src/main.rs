use ractor::{Actor, ActorProcessingErr, ActorRef};
use std::{io, time::Duration};
//use regex::Regex;
use chumsky::prelude::*;
use tracing::{info, warn};
use tracing_subscriber;

// Сообщения для акторов
#[derive(Debug)]
pub enum PipeMessage {
    Start,
    Message(String),
    Error(String),
    Result(String),
    Stop,
}

//Ссылки на акторы
pub struct PipeState {
    calculator_ref: Option<ActorRef<PipeMessage>>,
    err_handler_ref: Option<ActorRef<PipeMessage>>,
    out_actor_ref: Option<ActorRef<PipeMessage>>,
}

// Параметры выражения для калькулятора
#[derive(Clone)]
pub enum Expression {
    Numb(i32),
    Operator(Op, Box<Expression>, Box<Expression>),
}

#[derive(Clone)]
pub enum Op {
    Plus,
    Minus,
    Mul,
    Div,
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
        info!("Актор считывания запущен");
        Ok(PipeState { 
            calculator_ref: Some(calculator_ref),
            err_handler_ref: Some(err_handler_ref),
            out_actor_ref: None,
         })
    }

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
                        info!("Прочитано: {:?}", text);
                        if let Some(ref calc_ref) = state.calculator_ref {
                            calc_ref.send_message(PipeMessage::Message(text))?;
                        }
                    }
                    None => {
                        let error_msg = "Неверный формат выражения.".to_string();
                        warn!("{}", error_msg);
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
            info!("Остановка актора...");
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
    type Arguments = ActorRef<PipeMessage>;

    //Вывод сообщений перед запуском
    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        out_actor_ref: ActorRef<PipeMessage>,
    ) -> Result<Self::State, ActorProcessingErr> {
        info!("Актор калькуляции запущен");
        Ok(PipeState {
            calculator_ref:None,
            err_handler_ref: None,
            out_actor_ref: Some(out_actor_ref),
        })
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            PipeMessage::Start => {}
            PipeMessage::Message(parts) => {
                 //Парсим выражение EDITED
                 //Изменение на парсер chumsky
                let parsed = parser().parse(parts.chars().collect::<Vec<_>>());

                let result_msg = match parsed {
                    Ok(expr) => {
                        //Вычисляем дерево выражений
                        match op_action(&expr) {
                            Ok(val) => PipeMessage::Result(val.to_string()),
                            Err(e) => PipeMessage::Error(e),
                        }
                    }
                    Err(errors) => {
                        //Формируем сообщение об ошибке парсинга
                        let err_str = format!("Ошибка парсинга: {:?}", errors);
                        PipeMessage::Error(err_str)
                    }
                };
                // Отправляем результат или ошибку
                if let Some(ref out_ref) = state.out_actor_ref {
                    out_ref.send_message(result_msg)?;
                }
            }
            PipeMessage::Stop => {
                info!("Остановка актора...");
                myself.stop(None);
            }
        _ => {}
            }
    Ok(())
        }
}


//Структура Актора вывода сообщения
//Пока не удалял, думаю
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
        info!("Актор вывода сообщений запущен!");
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
                warn!("Ошибка: {}", msg);
            }
            PipeMessage::Result(result_str) => { 
                println!("Актор вывода выдал результат: {}", result_str);
            }
            PipeMessage::Stop => {
                info!("Остановка актора вывода...");
                myself.stop(None);
            }
            _ => {}
        }
        Ok(())
    }
}



//Чтение из консоли регулярным выражением EDITED
//Переделал на простое чтение строки
fn read_content () -> Option<String> {

    let mut input = String::new();
    
    io::stdin()
        .read_line(&mut input)
        .expect("Ошибка чтения строки");

    let input = input.trim().to_string();

    if input.is_empty() {
        None
    } else {
        Some(input)
    }

}

//Парсер chumsky
//Тут вопрос. Пробовал версию библы 0.11, но там не получилось инициализировать парсер 237 строчкой. 
//Откатился до 0.9 и все ок. Надо понять, что изменилось
fn parser() -> impl Parser<char, Expression, Error = Simple<char>> {
    recursive(|expr| {

        //Разбор обычного числа
        let int = text::int(10)
            .map(|s: String| s.parse::<i32>().unwrap_or(0))
            .map(Expression::Numb);

        //Разбор выражения в скобках с определением чисел внутри
        let skobki = int
            .or(expr.delimited_by(just('('), just(')')))
            .padded();

        //Разбор выражения с * и /
        let priority = skobki.clone()
            .then(
                just('*').to(Op::Mul)
                    .or(just('/').to(Op::Div))
                    .then(skobki)
                    .repeated(),
            )
            .foldl(|lhs, (op, rhs)| Expression::Operator(op, Box::new(lhs), Box::new(rhs)));

        //Разбор после приоритетного выражения + и -
        let sum = priority.clone()
            .then(
                just('+').to(Op::Plus)
                    .or(just('-').to(Op::Minus))
                    .then(priority)
                    .repeated(),
            )
            .foldl(|lhs, (op, rhs)| Expression::Operator(op, Box::new(lhs), Box::new(rhs)));

        sum
    })
}

//Функция вычисления дерева выражений
fn op_action(expr: &Expression) -> Result<i32, String> {
    match expr {
        Expression::Numb(n) => Ok(*n),
        Expression::Operator(op, l, r) => {
            let lv = op_action(l)?;
            let rv = op_action(r)?;
            match op {
                Op::Plus => Ok(lv + rv),
                Op::Minus => Ok(lv - rv),
                Op::Mul => Ok(lv * rv),
                Op::Div => {
                    if rv == 0 {
                        Err("Нельзя делить на 0".to_string())
                    } else {
                        Ok(lv / rv)
                    }
                }
            }
        }
    }
}


//Настройка компилятора и main fn
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    //Спавним акторы без ссылок
    let (text_writer, text_writer_handler) = OutActor::spawn(None,OutActor,())
    .await
    .expect("Не удалось создать актор вывода");

        let (calculator, calculator_handle) = Calculator::spawn(None, Calculator, text_writer.clone())
    .await
    .expect("Не удалось создать актор калькулятора");

    //Спавним актор чтения и передаем две ссылки в него
    let (text_reader, text_reader_handle) = TextReader::spawn(None, TextReader, (calculator.clone(), text_writer.clone()))
        .await
        .expect("Не удалось создать актор чтения");

    println!("=== Мониторинг консоли ===");

    text_reader.send_message(PipeMessage::Start)?;

    text_reader_handle.await?;
    calculator_handle.await?;
    text_writer_handler.await?;

    Ok(())

}