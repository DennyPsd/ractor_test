use ractor::{Actor, ActorProcessingErr, ActorRef};

// Сообщение для нашего актора
#[derive(Debug)]
pub enum HelloMessage {
    SayHello,
    SayHelloTo(String),
}

// Структура актора
pub struct HelloActor;

// Реализация обработки сообщений
#[cfg_attr(feature = "async-trait", crate::async_trait)]
impl Actor for HelloActor {
    type Msg = HelloMessage;
    type State = ();
    type Arguments = ();

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        println!("Актор запущен!");
        Ok(())
    }

    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        _state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            HelloMessage::SayHello => {
                println!("Hello world!");
            }
            HelloMessage::SayHelloTo(name) => {
                println!("Hello, {}!", name);
            }
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Запуск примера с актором...");

    // Создаем актор
    let (actor, handle) = Actor::spawn(None, HelloActor, ())
        .await
        .expect("Не удалось создать актор");

    // Отправляем сообщение
    actor.send_message(HelloMessage::SayHello)?;
    
    // Даем время на обработку
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Останавливаем актор
    actor.stop(Some("Завершение работы".to_string()));
    
    // Ждем завершения
    handle.await?;

    println!("Пример завершен!");
    Ok(())
}